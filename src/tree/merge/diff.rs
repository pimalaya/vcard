//! # Diff
//!
//! What one side changed relative to the base, at the finest granularity the
//! changed field allows: a whole property, a whole value, one component of a
//! structured value, one item of a list value, one parameter, one item of a
//! list parameter.
//!
//! Two values compare on their raw nodes, component by component, never
//! through the decoded model, which reads a non-structured value's first
//! `;`-component alone.
//!
//! List items merge as a set (both sides' additions and removals all apply),
//! so they never conflict, and the items of a `TYPE` or `PID` parameter
//! compare as one too, since RFC 6350 gives them no order.
//!
//! One parameter name may be written more than once
//! (`TEL;TYPE=work;TYPE=voice`, RFC 2426 section 4), and each occurrence is a
//! field of its own, so two sides rewriting two of them agree.
//!
//! A whole-value change reports what the two nodes say rather than what they
//! decode to, so a value the model reads truncated comes back as its raw
//! components instead of as a decoded value missing everything past its first
//! `;`-component.

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::{
    param::VcardParam,
    tree::{
        codec::VcardCodec,
        merge::{
            VcardMergeAction, VcardPropPath, compare::component_eq, instance::Instance,
            matching::Matching,
        },
        param::node::VcardParamNode,
        value::node::VcardValueNode,
    },
    value::{VcardValue, VcardValueKind, VcardValueUnknown},
};

/// Where a side's action lands, the merger's routing key: a matched (base,
/// side) instance pair, a base instance the side removed, or one it added.
pub(super) enum Target {
    Pair { base: usize, side: usize },
    Removed(usize),
    Added(usize),
}

/// One side of the merge, ready to be diffed against the base along its
/// matching.
pub(super) struct Diff<'d, 'a> {
    pub(super) base: &'d [Instance<'a>],
    pub(super) side: &'d [Instance<'a>],
    pub(super) matching: &'d Matching,
}

impl<'a> Diff<'_, 'a> {
    /// One action per observed change, each paired with the instance it
    /// targets.
    pub(super) fn run(&self) -> Vec<(Target, VcardMergeAction<'a>)> {
        let mut ops = Vec::new();

        for &(b, s) in &self.matching.pairs {
            let mut actions = Vec::new();
            Self::pair(&self.base[b], &self.side[s], &mut actions);
            ops.extend(
                actions
                    .into_iter()
                    .map(|action| (Target::Pair { base: b, side: s }, action)),
            );
        }

        for &b in &self.matching.removed {
            let action = VcardMergeAction::PropRemoved {
                at: self.base[b].path(),
                prop: self.base[b].prop.clone(),
            };
            ops.push((Target::Removed(b), action));
        }

        for &s in &self.matching.added {
            let action = VcardMergeAction::PropAdded {
                at: self.side[s].path(),
                prop: self.side[s].prop.clone(),
            };
            ops.push((Target::Added(s), action));
        }

        ops
    }

    /// Diff one matched pair: its parameters, then its value at the finest
    /// granularity the value shape allows.
    fn pair(b: &Instance<'a>, s: &Instance<'a>, out: &mut Vec<VcardMergeAction<'a>>) {
        let at = b.path();

        Self::params(b, s, &at, out);

        let old_node = &b.node().value;
        let new_node = &s.node().value;

        if old_node.same_value_as(new_node) {
            return;
        }

        match (&b.prop.value, &s.prop.value) {
            // NOTE: a list value's items are the whole value split on its
            // commas, while an item action is replayed by splicing one leaf
            // of component zero, so the two only address the same thing while
            // the value is one component; past that the whole value changed.
            (VcardValue::TextList(old), VcardValue::TextList(new))
                if old_node.component_count() == 1 && new_node.component_count() == 1 =>
            {
                let (added, removed) = list_diff(&old.0, &new.0);
                for item in removed {
                    out.push(VcardMergeAction::ValueItemRemoved {
                        at: at.clone(),
                        item,
                    });
                }
                for item in added {
                    out.push(VcardMergeAction::ValueItemAdded {
                        at: at.clone(),
                        item,
                    });
                }
            }
            (old, new)
                if old.kind() == new.kind()
                    && old
                        .kind()
                        .is_some_and(VcardValueKind::is_component_structured) =>
            {
                let count = old_node.component_count().max(new_node.component_count());

                for component in 0..count {
                    let old = old_node.decode_component_list(component);
                    let new = new_node.decode_component_list(component);
                    if !component_eq(&old, &new) {
                        out.push(VcardMergeAction::ValueComponentChanged {
                            at: at.clone(),
                            component,
                            old,
                            new,
                        });
                    }
                }
            }
            (old, new) => out.push(VcardMergeAction::ValueChanged {
                at: at.clone(),
                old: whole(old, old_node),
                new: whole(new, new_node),
            }),
        }
    }

    /// Diff two matched properties' parameter lists, by name then by
    /// position, a single `TYPE` or `PID` on both sides per item instead.
    fn params(
        b: &Instance<'a>,
        s: &Instance<'a>,
        at: &VcardPropPath<'a>,
        out: &mut Vec<VcardMergeAction<'a>>,
    ) {
        let (old, new) = (&b.prop.params, &s.prop.params);
        let (old_nodes, new_nodes) = (&b.node().params, &s.node().params);

        let mut keys: Vec<String> = Vec::new();
        for param in old.iter().chain(new) {
            let key = param.key();
            if !keys.contains(&key) {
                keys.push(key);
            }
        }

        for key in keys {
            // NOTE: a decode maps the parameters one for one, so each decoded
            // parameter is paired with the raw node it came from and the
            // comparison can run on the node.
            let olds: Vec<ParamPair<'_, 'a>> = old
                .iter()
                .zip(old_nodes)
                .filter(|(param, _)| param.key() == key)
                .collect();
            let news: Vec<ParamPair<'_, 'a>> = new
                .iter()
                .zip(new_nodes)
                .filter(|(param, _)| param.key() == key)
                .collect();

            if olds.len() == news.len()
                && olds
                    .iter()
                    .zip(&news)
                    .all(|((_, old), (_, new))| old.same_param_as(new))
            {
                continue;
            }

            if let (&[(old, _)], &[(new, _)]) = (olds.as_slice(), news.as_slice()) {
                match (old, new) {
                    (VcardParam::Type(old), VcardParam::Type(new))
                    | (VcardParam::Pid(old), VcardParam::Pid(new)) => {
                        let (added, removed) = list_diff(old, new);
                        for item in removed {
                            out.push(VcardMergeAction::ParamItemRemoved {
                                at: at.clone(),
                                index: 0,
                                param: Cow::Owned(key.clone()),
                                item,
                            });
                        }
                        for item in added {
                            out.push(VcardMergeAction::ParamItemAdded {
                                at: at.clone(),
                                index: 0,
                                param: Cow::Owned(key.clone()),
                                item,
                            });
                        }
                        continue;
                    }
                    _ => {}
                }
            }

            let shared = olds.len().min(news.len());
            for i in 0..shared {
                if !olds[i].1.same_param_as(news[i].1) {
                    out.push(VcardMergeAction::ParamChanged {
                        at: at.clone(),
                        index: i,
                        old: olds[i].0.clone(),
                        new: news[i].0.clone(),
                    });
                }
            }
            for (i, (param, _)) in news.iter().enumerate().skip(shared) {
                out.push(VcardMergeAction::ParamAdded {
                    at: at.clone(),
                    index: i,
                    param: (*param).clone(),
                });
            }
            for (i, (param, _)) in olds.iter().enumerate().skip(shared) {
                out.push(VcardMergeAction::ParamRemoved {
                    at: at.clone(),
                    index: i,
                    param: (*param).clone(),
                });
            }
        }
    }
}

impl VcardValueKind {
    /// Whether the kind is structured into `;`-components that carry
    /// independent meaning, so its components diff and merge one by one.
    pub(super) fn is_component_structured(self) -> bool {
        matches!(
            self,
            Self::N | Self::Adr | Self::Gender | Self::Org | Self::ClientPidMap,
        )
    }
}

/// One decoded parameter alongside the raw node it was decoded from.
type ParamPair<'p, 'a> = (&'p VcardParam<'a>, &'p VcardParamNode<'a>);

/// What a value node says, reported whole.
///
/// A decoded value says what its node says only when it encodes back to it:
/// `NOTE:a;b` decodes to the text `a;b` but re-encodes with the semicolon
/// escaped. A value failing that round trip is reported as raw components.
fn whole<'a>(value: &VcardValue<'a>, node: &'a VcardValueNode<'a>) -> VcardValue<'a> {
    if value.encode(node.escaper).same_value_as(node) {
        return value.clone();
    }

    VcardValue::Unknown(VcardValueUnknown::decode(node))
}

/// Diff two value lists as multisets: the items `new` gained and the items it
/// lost, matching duplicates one for one.
fn list_diff<'a>(
    old: &[Cow<'a, str>],
    new: &[Cow<'a, str>],
) -> (Vec<Cow<'a, str>>, Vec<Cow<'a, str>>) {
    let mut removed: Vec<Option<&Cow<'a, str>>> = old.iter().map(Some).collect();
    let mut added = Vec::new();

    for item in new {
        let kept = removed
            .iter()
            .position(|old| old.is_some_and(|old| old.as_ref() == item.as_ref()));

        match kept {
            Some(i) => removed[i] = None,
            None => added.push(item.clone()),
        }
    }

    let removed = removed.into_iter().flatten().cloned().collect();

    (added, removed)
}
