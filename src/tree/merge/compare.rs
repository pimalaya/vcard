//! # Sameness
//!
//! How the merge decides that two things are the same thing, hung on the
//! nodes it compares.
//!
//! Every comparison runs on the raw nodes rather than on the decoded model,
//! which reads its own kind's shape and hides what the kind cannot express:
//! two `data:` URIs differing in their payload alone decode alike, and a
//! single-valued parameter reads its first value alone, so two parameters
//! differing past their first `,` decode alike too.
//!
//! Two cards of different versions escape by different rules and share no
//! decoding to compare through, `http\://x` reading as itself in vCard 2.1
//! and as `http://x` later, so only identical bytes are then certainly the
//! same value.
//!
//! Agreement between the two sides is byte equality for the same reason. A
//! decode is not injective, so `\N` and `\n` both unescape to a line break
//! (RFC 6350 section 3.4) while saying different things on the wire, and
//! reading two such lines as one change would drop the difference without a
//! word.
//!
//! `TYPE` (RFC 6350 section 5.6) and `PID` (section 7) are the exception:
//! the specification gives their items no order, so they compare as a set and
//! two sides writing one set in two orders wrote one parameter.

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    param::VcardParam,
    tree::{
        codec::unescape::unescape_param, line::VcardLine, merge::VcardMergeAction,
        param::node::VcardParamNode, value::node::VcardValueNode,
    },
};

impl VcardValueNode<'_> {
    /// The node's raw bytes, as it would serialize.
    pub(super) fn raw_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write_bytes(&mut out);
        out
    }

    /// Whether two raw value nodes hold the same value, component by
    /// component, or the same bytes across two versions.
    pub(super) fn same_value_as(&self, other: &Self) -> bool {
        if self.escaper != other.escaper {
            return self.raw_bytes() == other.raw_bytes();
        }

        let count = self.component_count().max(other.component_count());

        (0..count).all(|i| {
            component_eq(
                &self.decode_component_list(i),
                &other.decode_component_list(i),
            )
        })
    }

    /// Whether two sides spelled one item of a list value the same way on the
    /// wire.
    pub(super) fn same_item_bytes_as(&self, other: &Self, item: &str) -> bool {
        let raw = |node: &Self| -> Option<Vec<u8>> {
            let at = node
                .decode_list()
                .iter()
                .position(|held| held.as_ref() == item)?;

            node.raw_list().into_iter().nth(at)
        };

        match (raw(self), raw(other)) {
            (Some(ours), Some(theirs)) => ours == theirs,
            _ => false,
        }
    }
}

impl VcardParamNode<'_> {
    /// The node's raw bytes, as it would serialize.
    pub(super) fn raw_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.write_bytes(&mut out);
        out
    }

    /// Whether two raw parameter nodes hold the same parameter, value by
    /// value, or the same bytes across two versions.
    pub(super) fn same_param_as(&self, other: &Self) -> bool {
        if self.escaper != other.escaper {
            return self.raw_bytes() == other.raw_bytes();
        }

        self.values.len() == other.values.len()
            && self.values.iter().zip(&other.values).all(|(ours, theirs)| {
                unescape_param(ours.get(), self.escaper)
                    == unescape_param(theirs.get(), other.escaper)
            })
    }
}

impl VcardParam<'_> {
    /// The dispatch key of a parameter: the canonical spelling of a known
    /// kind, or the uppercased name of an unknown one.
    pub(super) fn key(&self) -> String {
        if let Self::Unknown { name, .. } = self {
            return name.to_ascii_uppercase();
        }

        self.kind().expect("a known parameter kind").to_string()
    }

    /// Whether the parameter's items are a set rather than a sequence.
    pub(super) fn is_unordered(&self) -> bool {
        matches!(self, Self::Type(_) | Self::Pid(_))
    }

    /// Whether two parameters carry the same value, an unordered list's items
    /// compared as a set.
    pub(super) fn same_value_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Type(ours), Self::Type(theirs)) | (Self::Pid(ours), Self::Pid(theirs)) => {
                sorted(ours) == sorted(theirs)
            }
            (ours, theirs) => ours == theirs,
        }
    }
}

impl<'a> VcardLine<'a> {
    /// The `at`th parameter node of the line whose name matches the key.
    pub(super) fn param_node(&self, key: &str, at: usize) -> Option<&VcardParamNode<'a>> {
        self.params
            .iter()
            .filter(|node| node.name.get().eq_ignore_ascii_case(key))
            .nth(at)
    }

    /// The same, mutably, for item-level replay.
    pub(super) fn param_node_mut(
        &mut self,
        key: &str,
        at: usize,
    ) -> Option<&mut VcardParamNode<'a>> {
        self.params
            .iter_mut()
            .filter(|node| node.name.get().eq_ignore_ascii_case(key))
            .nth(at)
    }

    /// The still-encoded form one item of a list parameter was written as.
    pub(super) fn raw_param_item(&self, param: &str, index: usize, item: &str) -> Option<&str> {
        let node = self.param_node(param, index)?;

        node.values
            .iter()
            .find(|value| unescape_param(value.get(), node.escaper) == item)
            .map(|leaf| leaf.get())
    }

    /// Whether two sides spelled one parameter the same way on the wire.
    ///
    /// An unordered parameter compares as a set of raw items; every other one
    /// compares whole, so how it is written is part of what it says.
    pub(super) fn same_param_bytes_as(
        &self,
        other: &Self,
        param: &VcardParam<'_>,
        index: usize,
    ) -> bool {
        let key = param.key();

        let (Some(ours), Some(theirs)) =
            (self.param_node(&key, index), other.param_node(&key, index))
        else {
            return false;
        };

        if !param.is_unordered() {
            return ours.raw_bytes() == theirs.raw_bytes();
        }

        let items = |node: &VcardParamNode<'_>| {
            let mut items: Vec<String> = node
                .values
                .iter()
                .map(|leaf| leaf.get().to_string())
                .collect();
            items.sort_unstable();
            items
        };

        ours.name.get().eq_ignore_ascii_case(theirs.name.get()) && items(ours) == items(theirs)
    }
}

impl VcardMergeAction<'_> {
    /// Whether two actions are the same change, so a side that already made
    /// it needs no replay and reports no disagreement.
    ///
    /// Equality is exact but for a parameter, whose items compare as a set
    /// where the specification gives them no order.
    pub(super) fn same_change_as(&self, other: &Self) -> bool {
        use VcardMergeAction::{ParamAdded, ParamChanged, ParamRemoved};

        match (self, other) {
            (
                ParamAdded {
                    at: left_at,
                    index: left_index,
                    param: left,
                },
                ParamAdded {
                    at: right_at,
                    index: right_index,
                    param: right,
                },
            )
            | (
                ParamRemoved {
                    at: left_at,
                    index: left_index,
                    param: left,
                },
                ParamRemoved {
                    at: right_at,
                    index: right_index,
                    param: right,
                },
            ) => left_at == right_at && left_index == right_index && left.same_value_as(right),

            (
                ParamChanged {
                    at: left_at,
                    index: left_index,
                    old: left_old,
                    new: left_new,
                },
                ParamChanged {
                    at: right_at,
                    index: right_index,
                    old: right_old,
                    new: right_new,
                },
            ) => {
                left_at == right_at
                    && left_index == right_index
                    && left_old.same_value_as(right_old)
                    && left_new.same_value_as(right_new)
            }

            (left, right) => left == right,
        }
    }
}

/// Whether two decoded component value lists are equal, treating an absent
/// component and an empty one alike (`N:Doe;John` and `N:Doe;John;;;` agree).
pub(super) fn component_eq(old: &[Cow<'_, str>], new: &[Cow<'_, str>]) -> bool {
    let eq = old.len() == new.len()
        && old
            .iter()
            .zip(new)
            .all(|(old, new)| old.as_ref() == new.as_ref());

    eq || (old.iter().all(|value| value.is_empty()) && new.iter().all(|value| value.is_empty()))
}

/// A list parameter's items in a stable order, for comparing them as a set.
fn sorted<'v>(values: &'v [Cow<'_, str>]) -> Vec<&'v str> {
    let mut items: Vec<&str> = values.iter().map(Cow::as_ref).collect();
    items.sort_unstable();
    items
}
