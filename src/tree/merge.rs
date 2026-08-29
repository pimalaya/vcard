//! # Three-way merge
//!
//! Diff two divergent edits of a card against their common base and reconcile
//! them into one merged card.
//!
//! Given a base card and two cards derived from it (left and right), [`merge`]
//! reports every change each side made as a list of [`VcardMergeAction`]s and
//! builds the merged card, the reconciliation unit a synchronisation engine
//! needs. That card starts as a clone of the left one, so the left side's edits
//! are present byte for byte; the right side's actions are then replayed onto it
//! through the byte-preserving edit layer ([`crate::tree::value`]), so every
//! field the right side did not touch keeps its exact bytes.
//!
//! Property instances of the same name are matched across cards by `PID` first
//! (the RFC 6350 section 7 synchronisation identity), then by equality, then by
//! position. Two values compare on their raw nodes, component by component,
//! never through the decoded model, which reads a non-structured value's first
//! `;`-component alone; across versions they compare on the bytes, since two
//! cards escaping values differently share no decoding. Changes are diffed at
//! the finest granularity the value shape allows: whole property, whole value,
//! one component of a structured value, one item of a list value, one
//! parameter, one item of a list parameter. List items merge as a set (both
//! sides' additions and removals all apply), so they never conflict, and the
//! items of a `TYPE` or `PID` parameter compare as one too, since RFC 6350
//! gives them no order.
//!
//! Divergent changes to the same field are conflicts ([`VcardMergeConflict`]):
//! the left action wins in the merged card, except when a removal meets an
//! update, where the update wins at every granularity (data survives over
//! silent loss). A change both sides made is no conflict at all. Every conflict
//! is reported either way, so a caller can resolve differently.
//!
//! The merged card keeps the left card's `VERSION`; a version change is not
//! reconciled, but a value replayed from a card of another version is
//! re-encoded for the merged card's escaping mode, so it arrives meaning what
//! it meant. A `BEGIN` or `END` line is envelope rather than property, so it
//! is never diffed or replayed, and an addition lands among the outer card's
//! lines rather than inside a card embedded in a vCard 2.1 `AGENT`. Every line
//! of the merged card but its last carries a line ending, so the card a caller
//! serializes reads back as itself.

use core::mem;

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    param::VcardParam,
    prop::{VcardProp, VcardPropKind},
    tree::{
        codec::{mode::VcardEscaper, unescape::unescape},
        cst::VcardCst,
        leaf::VcardLeaf,
        line::VcardLine,
        param::node::VcardParamNode,
        prop::{cardinality::VcardPropCardinality, spec::prop_spec},
        value::node::VcardValueNode,
    },
    value::{VcardValue, VcardValueKind},
    version::VcardVersion,
};

/// Three-way merge `left` and `right` against their common `base`.
///
/// Returns the merged card along with each side's actions relative to the
/// base and the conflicts between them; see the module docs for the matching,
/// granularity and conflict rules.
pub fn merge<'a>(
    base: &'a VcardCst<'a>,
    left: &'a VcardCst<'a>,
    right: &'a VcardCst<'a>,
) -> VcardMergeReport<'a> {
    let base_insts = instances(base);
    let left_insts = instances(left);
    let right_insts = instances(right);

    let left_matching = matching(base, &base_insts, left, &left_insts);
    let right_matching = matching(base, &base_insts, right, &right_insts);

    let left_ops = diff(base, left, &base_insts, &left_insts, &left_matching);
    let right_ops = diff(base, right, &base_insts, &right_insts, &right_matching);

    let mut merger = Merger {
        escaper: VcardEscaper::for_version(left.version()),
        left,
        right,
        left_insts: &left_insts,
        right_insts: &right_insts,
        left_matching: &left_matching,
        left_ops: &left_ops,
        merged: left.clone(),
        conflicts: Vec::new(),
        removals: Vec::new(),
        additions: Vec::new(),
        readded: Vec::new(),
    };

    for (target, action) in &right_ops {
        merger.apply(target, action);
    }

    let (merged, conflicts) = merger.finish();

    VcardMergeReport {
        merged,
        left: left_ops.into_iter().map(|(_, action)| action).collect(),
        right: right_ops.into_iter().map(|(_, action)| action).collect(),
        conflicts,
    }
}

/// The outcome of a three-way [`merge`]: the merged card, each side's actions
/// relative to the base, and the conflicts between them.
#[derive(Clone, Debug)]
pub struct VcardMergeReport<'a> {
    /// The merged card: the left card with the right side's non-conflicting
    /// actions replayed onto it byte-preservingly.
    pub merged: VcardCst<'a>,
    /// What the left card changed relative to the base.
    pub left: Vec<VcardMergeAction<'a>>,
    /// What the right card changed relative to the base.
    pub right: Vec<VcardMergeAction<'a>>,
    /// The pairs of actions that collided on the same field.
    pub conflicts: Vec<VcardMergeConflict<'a>>,
}

/// Two actions that collided on the same field, one per side.
///
/// The merged card kept the left action's outcome, except for a removal
/// against an update, where the update's outcome was kept (whichever side it
/// came from).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VcardMergeConflict<'a> {
    /// The left card's action on the field.
    pub left: VcardMergeAction<'a>,
    /// The right card's action on the field.
    pub right: VcardMergeAction<'a>,
}

/// Addresses one property instance inside a diffed card pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VcardPropPath<'a> {
    /// The property's wire name as written (group prefix included), matched
    /// case-insensitively.
    pub name: Cow<'a, str>,
    /// The instance's position among the card's properties of that name: in
    /// the base card for every action but
    /// [`PropAdded`](VcardMergeAction::PropAdded), whose instance only exists
    /// in the changed card.
    pub index: usize,
}

/// One observed change of a card relative to the base, at the finest
/// granularity the changed field allows.
// NOTE: inherits VcardValue's ADR-dominated size; actions are a transient
// diff report, so plain variants beat boxing.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardMergeAction<'a> {
    /// A property the card added (absent from the base).
    PropAdded {
        /// The added instance (indexed in the changed card).
        at: VcardPropPath<'a>,
        /// The added property, decoded.
        prop: VcardProp<'a>,
    },
    /// A property the card removed (present in the base).
    PropRemoved {
        /// The removed base instance.
        at: VcardPropPath<'a>,
        /// The removed property, decoded.
        prop: VcardProp<'a>,
    },
    /// A matched property whose value changed as a whole (the fallback when
    /// no finer granularity applies).
    ValueChanged {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The base value.
        old: VcardValue<'a>,
        /// The changed value.
        new: VcardValue<'a>,
    },
    /// One `;`-component of a structured value (`N`, `ADR`, `GENDER`, `ORG`,
    /// `CLIENTPIDMAP`) changed.
    ValueComponentChanged {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The component position (e.g. 0 is `N`'s family names).
        component: usize,
        /// The component's base values.
        old: Vec<Cow<'a, str>>,
        /// The component's changed values.
        new: Vec<Cow<'a, str>>,
    },
    /// One item joined a list value (`NICKNAME`, `CATEGORIES`).
    ValueItemAdded {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The added item.
        item: Cow<'a, str>,
    },
    /// One item left a list value.
    ValueItemRemoved {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The removed item.
        item: Cow<'a, str>,
    },
    /// A parameter the card added on a matched property.
    ParamAdded {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The added parameter, decoded.
        param: VcardParam<'a>,
    },
    /// A parameter the card removed from a matched property.
    ParamRemoved {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The removed parameter, decoded.
        param: VcardParam<'a>,
    },
    /// A parameter of a matched property changed as a whole.
    ParamChanged {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The base parameter.
        old: VcardParam<'a>,
        /// The changed parameter.
        new: VcardParam<'a>,
    },
    /// One item joined a list parameter (`TYPE`, `PID`).
    ParamItemAdded {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The parameter's canonical name.
        param: Cow<'a, str>,
        /// The added item.
        item: Cow<'a, str>,
    },
    /// One item left a list parameter.
    ParamItemRemoved {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The parameter's canonical name.
        param: Cow<'a, str>,
        /// The removed item.
        item: Cow<'a, str>,
    },
}

impl VcardMergeAction<'_> {
    /// The field the action occupies, the granularity at which two sides'
    /// actions collide.
    fn slot(&self) -> Slot {
        match self {
            Self::PropAdded { .. } | Self::PropRemoved { .. } => Slot::Prop,
            Self::ValueChanged { .. } => Slot::Value,
            Self::ValueComponentChanged { component, .. } => Slot::Component(*component),
            Self::ValueItemAdded { .. } | Self::ValueItemRemoved { .. } => Slot::Items,
            Self::ParamAdded { param, .. } | Self::ParamRemoved { param, .. } => {
                Slot::Param(param_key(param))
            }
            Self::ParamChanged { new, .. } => Slot::Param(param_key(new)),
            Self::ParamItemAdded { param, .. } | Self::ParamItemRemoved { param, .. } => {
                Slot::ParamItems(param.to_ascii_uppercase())
            }
        }
    }
}

/// The field of a property instance an action occupies.
#[derive(Debug, PartialEq, Eq)]
enum Slot {
    /// The whole property (added or removed).
    Prop,
    /// The whole value.
    Value,
    /// One component of a structured value.
    Component(usize),
    /// The items of a list value (item edits merge as a set, so they never
    /// collide).
    Items,
    /// One whole parameter, by key.
    Param(String),
    /// The items of a list parameter, by key.
    ParamItems(String),
}

impl Slot {
    /// Whether a left action on this slot collides with a right action on
    /// `right`. Two item edits never collide: they merge as a set. An item
    /// edit does collide with a whole-value change on the other side, either
    /// way round, since one of the two values has to go.
    fn collides_with(&self, right: &Slot) -> bool {
        match (self, right) {
            (Self::Value, Self::Value | Self::Component(_) | Self::Items) => true,
            (Self::Component(_) | Self::Items, Self::Value) => true,
            (Self::Component(left), Self::Component(right)) => left == right,
            (Self::Param(left) | Self::ParamItems(left), Self::Param(right)) => left == right,
            _ => false,
        }
    }
}

/// One decodable property line of a card, the unit the matching pairs up.
struct Instance<'a> {
    /// The line's index in its card's props.
    line: usize,
    /// The instance's position among the card's same-name properties.
    nth: usize,
    /// The uppercased (group-qualified) wire name, the matching key.
    key: String,
    /// The decoded property.
    prop: VcardProp<'a>,
}

/// Decode every property line of a card into a matchable instance, skipping
/// the indicator and envelope lines, which are not properties: `VERSION`, and
/// the `BEGIN` / `END` pair an embedded card or a bare RFC 2425 record may
/// carry among the lines.
fn instances<'a>(cst: &'a VcardCst<'a>) -> Vec<Instance<'a>> {
    let version = cst.version();
    let mut insts: Vec<Instance<'a>> = Vec::new();

    for (line, node) in cst.props.iter().enumerate() {
        let name = node.name.get();
        if is_envelope(name) {
            continue;
        }

        let key = name.to_ascii_uppercase();
        let nth = insts.iter().filter(|inst| inst.key == key).count();

        insts.push(Instance {
            line,
            nth,
            key,
            prop: node.decode(version),
        });
    }

    insts
}

/// Whether a line names the card's structure rather than a property: the
/// `VERSION` indicator, or one half of a `BEGIN` / `END` pair.
///
/// A vCard 2.1 `AGENT` embeds a whole `BEGIN:VCARD`..`END:VCARD` that the
/// parser keeps verbatim among the outer card's properties, and a bare RFC
/// 2425 record has no envelope, so a `BEGIN` or `END` line in one is an
/// ordinary line of the body. Neither is a property, so neither is diffed,
/// matched, moved or replayed: replaying an `END` would close the merged card
/// early and drop everything after it.
fn is_envelope(name: &str) -> bool {
    name.eq_ignore_ascii_case("VERSION")
        || name.eq_ignore_ascii_case("BEGIN")
        || name.eq_ignore_ascii_case("END")
}

/// Which lines of a card belong to an embedded card rather than to the card
/// itself, `BEGIN` and `END` included.
///
/// An embedded card's lines are diffed like any other property, since a change
/// inside an agent is still a change, but an addition is never *placed* among
/// them: it belongs to the card that owns the property name it repeats.
fn embedded(props: &[VcardLine<'_>]) -> Vec<bool> {
    let mut out = Vec::with_capacity(props.len());
    let mut depth = 0usize;

    for line in props {
        let name = line.name.get();

        if name.eq_ignore_ascii_case("BEGIN") {
            depth += 1;
            out.push(true);
        } else if name.eq_ignore_ascii_case("END") {
            depth = depth.saturating_sub(1);
            out.push(true);
        } else {
            out.push(depth > 0);
        }
    }

    out
}

/// The instance pairing between the base card and one side.
struct Matching {
    /// The matched (base, side) instance index pairs.
    pairs: Vec<(usize, usize)>,
    /// The side instances with no base counterpart.
    added: Vec<usize>,
    /// The base instances with no side counterpart.
    removed: Vec<usize>,
}

/// Pair the base instances with one side's, per name: by `PID` and equality
/// together first, then by `PID` alone (the RFC 6350 section 7 synchronisation
/// identity), then by identical bytes, then by equality alone, then by
/// position. Leftovers are additions (side) and removals (base).
fn matching<'a>(
    base_cst: &VcardCst<'a>,
    base: &[Instance<'a>],
    side_cst: &VcardCst<'a>,
    side: &[Instance<'a>],
) -> Matching {
    let mut keys: Vec<&str> = Vec::new();
    for inst in base.iter().chain(side) {
        if !keys.contains(&inst.key.as_str()) {
            keys.push(&inst.key);
        }
    }

    let mut pairs = Vec::new();
    let mut added = Vec::new();
    let mut removed = Vec::new();

    for key in keys {
        let mut base_free: Vec<usize> = indices_of(base, key);
        let mut side_free: Vec<usize> = indices_of(side, key);

        // NOTE: a card may carry two instances of one name under one `PID`,
        // and pairing them in source order would then break an identical pair
        // and rewrite two lines nobody touched. Identity and equality
        // together come first, so an unchanged instance stays unchanged.
        pair_by(&mut base_free, &mut side_free, &mut pairs, |b, s| {
            pids_overlap(&base[b].prop, &side[s].prop)
                && prop_eq(base_cst, &base[b], side_cst, &side[s])
        });
        pair_by(&mut base_free, &mut side_free, &mut pairs, |b, s| {
            pids_overlap(&base[b].prop, &side[s].prop)
        });
        // NOTE: among instances that decode alike, the one written the same
        // way comes first, so a card carrying an interchangeable duplicate
        // loses the copy nobody else spells that way rather than a copy all
        // three copies carry byte for byte.
        pair_by(&mut base_free, &mut side_free, &mut pairs, |b, s| {
            line_eq(base_cst, &base[b], side_cst, &side[s])
        });
        pair_by(&mut base_free, &mut side_free, &mut pairs, |b, s| {
            prop_eq(base_cst, &base[b], side_cst, &side[s])
        });

        while !base_free.is_empty() && !side_free.is_empty() {
            pairs.push((base_free.remove(0), side_free.remove(0)));
        }

        removed.append(&mut base_free);
        added.append(&mut side_free);
    }

    Matching {
        pairs,
        added,
        removed,
    }
}

/// The indices of a card's instances carrying the given name key, in order.
fn indices_of(insts: &[Instance<'_>], key: &str) -> Vec<usize> {
    insts
        .iter()
        .enumerate()
        .filter(|(_, inst)| inst.key == key)
        .map(|(i, _)| i)
        .collect()
}

/// Greedily pair the free base and side instances that `matches`, removing
/// each formed pair from the free lists.
fn pair_by(
    base_free: &mut Vec<usize>,
    side_free: &mut Vec<usize>,
    pairs: &mut Vec<(usize, usize)>,
    matches: impl Fn(usize, usize) -> bool,
) {
    let mut b = 0;
    while b < base_free.len() {
        match side_free.iter().position(|&s| matches(base_free[b], s)) {
            Some(s) => pairs.push((base_free.remove(b), side_free.remove(s))),
            None => b += 1,
        }
    }
}

/// The values of a property's `PID` parameter, if it carries one.
fn pids<'p, 'a>(prop: &'p VcardProp<'a>) -> Option<&'p [Cow<'a, str>]> {
    prop.params.iter().find_map(|param| match param {
        VcardParam::Pid(values) => Some(values.as_slice()),
        _ => None,
    })
}

/// Whether two properties share at least one `PID` source identifier.
fn pids_overlap(a: &VcardProp<'_>, b: &VcardProp<'_>) -> bool {
    match (pids(a), pids(b)) {
        (Some(a), Some(b)) => a.iter().any(|pid| b.contains(pid)),
        _ => false,
    }
}

/// Where a side's action lands, the merger's routing key.
enum Target {
    /// A matched (base, side) instance pair.
    Pair { base: usize, side: usize },
    /// A base instance the side removed.
    Removed(usize),
    /// A side instance the side added.
    Added(usize),
}

/// Diff one side against the base along its matching: one action per observed
/// change, each paired with the instance it targets.
fn diff<'a>(
    base: &'a VcardCst<'a>,
    side: &'a VcardCst<'a>,
    base_insts: &[Instance<'a>],
    side_insts: &[Instance<'a>],
    matching: &Matching,
) -> Vec<(Target, VcardMergeAction<'a>)> {
    let mut ops = Vec::new();

    for &(b, s) in &matching.pairs {
        let target = || Target::Pair { base: b, side: s };
        let mut actions = Vec::new();
        diff_pair(base, side, &base_insts[b], &side_insts[s], &mut actions);
        ops.extend(actions.into_iter().map(|action| (target(), action)));
    }

    for &b in &matching.removed {
        let action = VcardMergeAction::PropRemoved {
            at: prop_path(base, &base_insts[b]),
            prop: base_insts[b].prop.clone(),
        };
        ops.push((Target::Removed(b), action));
    }

    for &s in &matching.added {
        let action = VcardMergeAction::PropAdded {
            at: prop_path(side, &side_insts[s]),
            prop: side_insts[s].prop.clone(),
        };
        ops.push((Target::Added(s), action));
    }

    ops
}

/// The path of an instance inside its own card.
fn prop_path<'a>(cst: &'a VcardCst<'a>, inst: &Instance<'a>) -> VcardPropPath<'a> {
    VcardPropPath {
        name: Cow::Borrowed(cst.props[inst.line].name.get()),
        index: inst.nth,
    }
}

/// Diff one matched pair: its parameters, then its value at the finest
/// granularity the value shape allows.
fn diff_pair<'a>(
    base: &'a VcardCst<'a>,
    side: &'a VcardCst<'a>,
    b: &Instance<'a>,
    s: &Instance<'a>,
    out: &mut Vec<VcardMergeAction<'a>>,
) {
    let at = prop_path(base, b);

    diff_params(&b.prop.params, &s.prop.params, &at, out);

    let old_node = &base.props[b.line].value;
    let new_node = &side.props[s.line].value;

    if value_eq(old_node, new_node) {
        return;
    }

    match (&b.prop.value, &s.prop.value) {
        // NOTE: a list value decodes its first component only, so it diffs
        // item by item only while the rest of the node agrees; otherwise the
        // whole value changed.
        (VcardValue::TextList(old), VcardValue::TextList(new))
            if components_eq(old_node, new_node, 1) =>
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
        (old, new) if old.kind() == new.kind() && is_component_structured(old.kind()) => {
            let count = old_node.component_count().max(new_node.component_count());

            for component in 0..count {
                let old = old_node.decode_at(component);
                let new = new_node.decode_at(component);
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
            old: old.clone(),
            new: new.clone(),
        }),
    }
}

/// Whether a value kind is structured into `;`-components that carry
/// independent meaning, so its components diff and merge one by one.
fn is_component_structured(kind: Option<VcardValueKind>) -> bool {
    matches!(
        kind,
        Some(
            VcardValueKind::N
                | VcardValueKind::Adr
                | VcardValueKind::Gender
                | VcardValueKind::Org
                | VcardValueKind::ClientPidMap,
        ),
    )
}

/// Whether two decoded component value lists are equal, treating an absent
/// component and an empty one alike (`N:Doe;John` and `N:Doe;John;;;` agree).
fn component_eq(old: &[Cow<'_, str>], new: &[Cow<'_, str>]) -> bool {
    let eq = old.len() == new.len()
        && old
            .iter()
            .zip(new)
            .all(|(old, new)| old.as_ref() == new.as_ref());

    eq || (old.iter().all(|value| value.is_empty()) && new.iter().all(|value| value.is_empty()))
}

/// Whether two instances serialize to the same bytes, line ending included.
fn line_eq<'a>(
    a_cst: &VcardCst<'a>,
    a: &Instance<'a>,
    b_cst: &VcardCst<'a>,
    b: &Instance<'a>,
) -> bool {
    let (mut left, mut right) = (Vec::new(), Vec::new());

    a_cst.props[a.line].write_bytes(&mut left);
    b_cst.props[b.line].write_bytes(&mut right);

    left == right
}

/// Whether two instances hold the same property: the same decoded name and
/// parameters, and the same value on the raw node.
fn prop_eq<'a>(
    a_cst: &VcardCst<'a>,
    a: &Instance<'a>,
    b_cst: &VcardCst<'a>,
    b: &Instance<'a>,
) -> bool {
    a.prop.name == b.prop.name
        && a.prop.params == b.prop.params
        && value_eq(&a_cst.props[a.line].value, &b_cst.props[b.line].value)
}

/// Whether two raw value nodes hold the same value.
///
/// The comparison runs on the node rather than on the decoded value, which
/// reads only a value's first `;`-component and would make a change past it
/// invisible: two `data:` URIs differing in their payload alone decode alike.
fn value_eq(old: &VcardValueNode<'_>, new: &VcardValueNode<'_>) -> bool {
    components_eq(old, new, 0)
}

/// Whether two raw value nodes agree on every component from `from` onwards.
fn components_eq(old: &VcardValueNode<'_>, new: &VcardValueNode<'_>, from: usize) -> bool {
    // NOTE: two cards of different versions escape values by different rules,
    // so they share no decoding to compare through: `http\://x` reads as
    // itself in 2.1 and as `http://x` later. Only identical bytes are then
    // certainly the same value.
    if old.escaper != new.escaper {
        return raw_value(old) == raw_value(new);
    }

    let count = old.component_count().max(new.component_count());

    (from..count).all(|i| component_eq(&old.decode_at(i), &new.decode_at(i)))
}

/// A value node's raw bytes, as it would serialize.
fn raw_value(node: &VcardValueNode<'_>) -> Vec<u8> {
    let mut out = Vec::new();
    node.write_bytes(&mut out);
    out
}

/// Diff two matched properties' parameter lists, keyed by parameter name. A
/// single `TYPE` / `PID` on both sides diffs per item (they are sets);
/// everything else diffs as whole parameters.
fn diff_params<'a>(
    old: &[VcardParam<'a>],
    new: &[VcardParam<'a>],
    at: &VcardPropPath<'a>,
    out: &mut Vec<VcardMergeAction<'a>>,
) {
    let mut keys: Vec<String> = Vec::new();
    for param in old.iter().chain(new) {
        let key = param_key(param);
        if !keys.contains(&key) {
            keys.push(key);
        }
    }

    for key in keys {
        let olds: Vec<&VcardParam<'a>> = old.iter().filter(|p| param_key(p) == key).collect();
        let news: Vec<&VcardParam<'a>> = new.iter().filter(|p| param_key(p) == key).collect();

        if olds.len() == news.len() && olds.iter().zip(&news).all(|(old, new)| old == new) {
            continue;
        }

        if let (&[old], &[new]) = (olds.as_slice(), news.as_slice()) {
            match (old, new) {
                (VcardParam::Type(old), VcardParam::Type(new))
                | (VcardParam::Pid(old), VcardParam::Pid(new)) => {
                    let (added, removed) = list_diff(old, new);
                    for item in removed {
                        out.push(VcardMergeAction::ParamItemRemoved {
                            at: at.clone(),
                            param: Cow::Owned(key.clone()),
                            item,
                        });
                    }
                    for item in added {
                        out.push(VcardMergeAction::ParamItemAdded {
                            at: at.clone(),
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
            if olds[i] != news[i] {
                out.push(VcardMergeAction::ParamChanged {
                    at: at.clone(),
                    old: olds[i].clone(),
                    new: news[i].clone(),
                });
            }
        }
        for &param in &news[shared..] {
            out.push(VcardMergeAction::ParamAdded {
                at: at.clone(),
                param: param.clone(),
            });
        }
        for &param in &olds[shared..] {
            out.push(VcardMergeAction::ParamRemoved {
                at: at.clone(),
                param: param.clone(),
            });
        }
    }
}

/// Whether two actions are the same change, so a side that already made it
/// needs no replay and reports no disagreement.
///
/// Equality is exact but for a list parameter, whose items compare as a set:
/// RFC 6350 section 5.6 gives `TYPE` no ordering, and `PID` none either.
fn same_change(left: &VcardMergeAction<'_>, right: &VcardMergeAction<'_>) -> bool {
    use VcardMergeAction::{ParamAdded, ParamChanged, ParamRemoved};

    match (left, right) {
        (
            ParamAdded {
                at: left_at,
                param: left,
            },
            ParamAdded {
                at: right_at,
                param: right,
            },
        )
        | (
            ParamRemoved {
                at: left_at,
                param: left,
            },
            ParamRemoved {
                at: right_at,
                param: right,
            },
        ) => left_at == right_at && param_eq(left, right),

        (
            ParamChanged {
                at: left_at,
                old: left_old,
                new: left_new,
            },
            ParamChanged {
                at: right_at,
                old: right_old,
                new: right_new,
            },
        ) => left_at == right_at && param_eq(left_old, right_old) && param_eq(left_new, right_new),

        (left, right) => left == right,
    }
}

/// Whether two parameters carry the same value, a list parameter's items
/// compared as an unordered set.
fn param_eq(left: &VcardParam<'_>, right: &VcardParam<'_>) -> bool {
    match (left, right) {
        (VcardParam::Type(left), VcardParam::Type(right))
        | (VcardParam::Pid(left), VcardParam::Pid(right)) => sorted(left) == sorted(right),
        (left, right) => left == right,
    }
}

/// A list parameter's items in a stable order, for comparing them as a set.
fn sorted<'v>(values: &'v [Cow<'_, str>]) -> Vec<&'v str> {
    let mut items: Vec<&str> = values.iter().map(Cow::as_ref).collect();
    items.sort_unstable();
    items
}

/// The dispatch key of a parameter: the canonical spelling of a known kind,
/// or the uppercased name of an unknown one.
fn param_key(param: &VcardParam<'_>) -> String {
    if let VcardParam::Unknown { name, .. } = param {
        return name.to_ascii_uppercase();
    }

    // NOTE: every variant but Unknown has a kind.
    param.kind().expect("a known parameter kind").to_string()
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

/// The merge state while the right side's actions replay onto the left
/// clone: the merged card under edit, the recorded conflicts, and the
/// deferred structural changes (index-stable edits happen in place; removals
/// and additions are deferred to [`finish`](Self::finish)).
struct Merger<'o, 'a> {
    escaper: VcardEscaper,
    left: &'a VcardCst<'a>,
    right: &'a VcardCst<'a>,
    left_insts: &'o [Instance<'a>],
    right_insts: &'o [Instance<'a>],
    left_matching: &'o Matching,
    left_ops: &'o [(Target, VcardMergeAction<'a>)],
    merged: VcardCst<'a>,
    conflicts: Vec<VcardMergeConflict<'a>>,
    removals: Vec<usize>,
    additions: Vec<VcardLine<'a>>,
    readded: Vec<usize>,
}

impl<'a> Merger<'_, 'a> {
    /// Replay one right-side action onto the merged card, or record the
    /// conflict that prevents it.
    fn apply(&mut self, target: &Target, action: &VcardMergeAction<'a>) {
        match target {
            Target::Pair { base, side } => self.apply_pair(*base, *side, action),
            Target::Removed(base) => self.apply_removed(*base, action),
            Target::Added(side) => self.apply_added(*side, action),
        }
    }

    /// Replay a right-side edit of a matched property.
    fn apply_pair(&mut self, b: usize, r: usize, action: &VcardMergeAction<'a>) {
        // NOTE: the left side removed the property the right side edited: a
        // remove-update conflict, resolved for the update (data survives over
        // silent loss) by restoring the right side's whole line.
        if self.left_matching.removed.contains(&b) {
            if !self.readded.contains(&b) {
                self.readded.push(b);
                let line = self.right_line(self.right_insts[r].line);
                self.additions.push(line);
                self.record(self.left_removed_action(b), action);
            }
            return;
        }

        let line = self.left_line(b);

        match action {
            VcardMergeAction::ValueChanged { .. } => {
                // NOTE: the action's decoded payload cannot tell two values
                // apart past their first `;`-component, so the two sides are
                // compared on their nodes: same value, nothing to replay.
                let left_value = &self.left.props[line].value;
                let right_value = &self.right.props[self.right_insts[r].line].value;

                if value_eq(left_value, right_value) {
                    return;
                }

                if let Some(colliding) = self.colliding(b, &Slot::Value) {
                    self.record(colliding.clone(), action);
                    return;
                }

                self.merged.props[line].value = transcode(right_value, self.escaper);
            }

            VcardMergeAction::ValueComponentChanged { component, new, .. } => {
                if self.already_made(b, action) {
                    return;
                }

                if let Some(colliding) = self.colliding(b, &Slot::Component(*component)) {
                    self.record(colliding.clone(), action);
                    return;
                }

                self.merged.props[line].value.set_at(*component, new);
            }

            VcardMergeAction::ValueItemAdded { item, .. } => {
                if let Some(colliding) = self.colliding(b, &Slot::Items) {
                    self.record(colliding.clone(), action);
                    return;
                }

                let value = &mut self.merged.props[line].value;
                let present = value
                    .decode_at(0)
                    .iter()
                    .any(|value| value.as_ref() == item.as_ref());

                if !present {
                    value.push_value(0, item);
                }
            }

            VcardMergeAction::ValueItemRemoved { item, .. } => {
                // NOTE: a removal is not idempotent when the list holds the
                // item twice, so a side that already dropped it must not drop
                // a second copy neither side wrote off.
                if self.already_made(b, action) {
                    return;
                }

                if let Some(colliding) = self.colliding(b, &Slot::Items) {
                    self.record(colliding.clone(), action);
                    return;
                }

                let value = &mut self.merged.props[line].value;
                let position = value
                    .decode_at(0)
                    .iter()
                    .position(|value| value.as_ref() == item.as_ref());

                if let Some(j) = position {
                    value.remove_value_at(0, j);
                }
            }

            VcardMergeAction::ParamAdded { param, .. } => {
                if self.already_made(b, action) {
                    return;
                }

                // NOTE: an update beats a removal here as it does at property
                // granularity, so the addition still lands when all the left
                // side did was remove the parameter.
                if let Some(colliding) = self.colliding(b, &Slot::Param(param_key(param))) {
                    let removed = matches!(colliding, VcardMergeAction::ParamRemoved { .. });
                    self.record(colliding.clone(), action);

                    if !removed {
                        return;
                    }
                }

                if let Some(node) = self.right_param_node(r, param) {
                    self.merged.props[line].params.push(node.clone());
                }
            }

            VcardMergeAction::ParamRemoved { param, .. } => {
                if self.already_made(b, action) {
                    return;
                }

                if let Some(colliding) = self.colliding(b, &Slot::Param(param_key(param))) {
                    self.record(colliding.clone(), action);
                    return;
                }

                let position = self.merged.props[line]
                    .params
                    .iter()
                    .position(|node| node.decode() == *param);

                if let Some(i) = position {
                    self.merged.props[line].params.remove(i);
                }
            }

            VcardMergeAction::ParamChanged { old, new, .. } => {
                if self.already_made(b, action) {
                    return;
                }

                let mut restore = false;

                if let Some(colliding) = self.colliding(b, &Slot::Param(param_key(new))) {
                    restore = matches!(colliding, VcardMergeAction::ParamRemoved { .. });
                    self.record(colliding.clone(), action);

                    if !restore {
                        return;
                    }
                }

                let position = self.merged.props[line]
                    .params
                    .iter()
                    .position(|node| node.decode() == *old);

                if let Some(node) = self.right_param_node(r, new) {
                    match (position, restore) {
                        (Some(i), _) => self.merged.props[line].params[i] = node.clone(),
                        // NOTE: the left side removed the parameter this
                        // update rewrote, so the update brings it back.
                        (None, true) => self.merged.props[line].params.push(node.clone()),
                        (None, false) => {}
                    }
                }
            }

            VcardMergeAction::ParamItemAdded { param, item, .. } => {
                // NOTE: a parameter value is read through the value unescaper
                // and written back verbatim, so the item has to land as the
                // right card spelled it: written decoded, a `\n` would become
                // a real line break and cut the line in two.
                let leaf = self.right_param_item(r, param, item);

                let Some(node) = param_node_mut(&mut self.merged.props[line], param) else {
                    self.restore_param(b, r, param, action);
                    return;
                };

                let present = node
                    .values
                    .iter()
                    .any(|value| unescape(value.get()) == item.as_ref());

                if let Some(leaf) = leaf
                    && !present
                {
                    node.values.push(leaf);
                }
            }

            VcardMergeAction::ParamItemRemoved { param, item, .. } => {
                // NOTE: as above, a parameter may hold one item twice, and
                // `TYPE=work,,` is exactly that.
                if self.already_made(b, action) {
                    return;
                }

                let Some(node) = param_node_mut(&mut self.merged.props[line], param) else {
                    self.restore_param(b, r, param, action);
                    return;
                };

                let position = node
                    .values
                    .iter()
                    .position(|value| unescape(value.get()) == item.as_ref());

                if let Some(i) = position {
                    node.values.remove(i);
                }
            }

            // NOTE: prop-level actions carry Removed / Added targets.
            VcardMergeAction::PropAdded { .. } | VcardMergeAction::PropRemoved { .. } => {}
        }
    }

    /// Replay a right-side property removal.
    fn apply_removed(&mut self, b: usize, action: &VcardMergeAction<'a>) {
        // NOTE: both sides removed it: already gone from the left clone.
        if self.left_matching.removed.contains(&b) {
            return;
        }

        // NOTE: the left side edited what the right side removed: an
        // update-remove conflict, resolved for the update by keeping the left
        // line as edited.
        let colliding = self.left_ops_on(b).next().map(|(_, op)| op.clone());
        if let Some(colliding) = colliding {
            self.record(colliding, action);
            return;
        }

        let line = self.left_line(b);
        self.removals.push(line);
    }

    /// Replay a right-side property addition.
    fn apply_added(&mut self, s: usize, action: &VcardMergeAction<'a>) {
        let VcardMergeAction::PropAdded { .. } = action else {
            return;
        };
        let inst = &self.right_insts[s];

        // NOTE: the left side added the same property: one copy is enough.
        let both_added = self.left_matching.added.iter().any(|&l| {
            let left = &self.left_insts[l];
            left.key == inst.key && prop_eq(self.left, left, self.right, inst)
        });
        if both_added {
            return;
        }

        // NOTE: both sides added different content under a name allowed at
        // most once: keep the left one and report the collision.
        if at_most_one(&inst.key, self.merged.version()) {
            let colliding = self.left_ops.iter().find(|(target, _)| {
                matches!(target, Target::Added(l) if self.left_insts[*l].key == inst.key)
            });

            if let Some((_, colliding)) = colliding {
                self.record(colliding.clone(), action);
                return;
            }
        }

        self.additions.push(self.right_line(inst.line));
    }

    /// Run the deferred structural changes and return the merged card with
    /// the recorded conflicts. Removals go first (on stable indices), then
    /// each addition lands after the last line sharing its name, or at the
    /// end, and every line but the last is terminated.
    fn finish(mut self) -> (VcardCst<'a>, Vec<VcardMergeConflict<'a>>) {
        self.removals.sort_unstable();
        self.removals.dedup();
        for &line in self.removals.iter().rev() {
            self.merged.props.remove(line);
        }

        for line in mem::take(&mut self.additions) {
            // NOTE: only the outer card's lines count here: an embedded card
            // carries its own properties, and with an agent present the last
            // line of a given name is usually the agent's, which is where an
            // addition to the outer card used to land.
            let embedded = embedded(&self.merged.props);

            let after = self
                .merged
                .props
                .iter()
                .enumerate()
                .filter(|(i, _)| !embedded[*i])
                .rev()
                .find(|(_, prop)| prop.name.get().eq_ignore_ascii_case(line.name.get()))
                .map(|(i, _)| i);

            match after {
                Some(i) => self.merged.props.insert(i + 1, line),
                None => self.merged.props.push(line),
            }
        }

        terminate_lines(&mut self.merged);

        (self.merged, self.conflicts)
    }

    /// The merged (= left) line index of a base instance the left side kept.
    fn left_line(&self, b: usize) -> usize {
        let pair = self
            .left_matching
            .pairs
            .iter()
            .find(|(base, _)| *base == b)
            .expect("a left-matched base instance");

        self.left_insts[pair.1].line
    }

    /// The left-side actions targeting a base instance.
    fn left_ops_on(&self, b: usize) -> impl Iterator<Item = &(Target, VcardMergeAction<'a>)> {
        self.left_ops.iter().filter(move |(target, _)| {
            matches!(target, Target::Pair { base, .. } | Target::Removed(base) if *base == b)
        })
    }

    /// Whether the left side already made the very same change, so the merged
    /// card holds it and the right action needs neither a replay nor a report.
    fn already_made(&self, b: usize, action: &VcardMergeAction<'a>) -> bool {
        self.left_ops_on(b)
            .any(|(_, left)| same_change(left, action))
    }

    /// The left-side action whose slot collides with a right action's slot on
    /// the same base instance, if any.
    fn colliding(&self, b: usize, right: &Slot) -> Option<&VcardMergeAction<'a>> {
        self.left_ops_on(b)
            .find(|(_, action)| action.slot().collides_with(right))
            .map(|(_, action)| action)
    }

    /// The left side's removal action of a base instance.
    fn left_removed_action(&self, b: usize) -> VcardMergeAction<'a> {
        self.left_ops
            .iter()
            .find(|(target, _)| matches!(target, Target::Removed(base) if *base == b))
            .map(|(_, action)| action.clone())
            .expect("a left removal action")
    }

    /// One line of the right card, ready to land in the merged one: its value
    /// re-encoded for the merged card's escaping mode when the two differ.
    fn right_line(&self, line: usize) -> VcardLine<'a> {
        let mut line = self.right.props[line].clone();
        line.value = transcode(&line.value, self.escaper);
        line
    }

    /// The right card's raw leaf for one decoded item of a list parameter.
    ///
    /// The item was decoded from that very node, so the leaf is there; without
    /// it there is no wire form to write, and the decoded text is not one: a
    /// parameter value is unescaped on the way in and copied verbatim on the
    /// way out, so writing it back decoded can put a line break in the head.
    fn right_param_item(&self, r: usize, param: &str, item: &str) -> Option<VcardLeaf<'a>> {
        self.right.props[self.right_insts[r].line]
            .params
            .iter()
            .filter(|node| node.name.get().eq_ignore_ascii_case(param))
            .flat_map(|node| node.values.iter())
            .find(|value| unescape(value.get()) == item)
            .cloned()
    }

    /// The right card's raw node of a decoded parameter, for byte-faithful
    /// replay.
    fn right_param_node(&self, r: usize, param: &VcardParam<'_>) -> Option<&'a VcardParamNode<'a>> {
        self.right.props[self.right_insts[r].line]
            .params
            .iter()
            .find(|node| node.decode() == *param)
    }

    /// Replay a right item edit whose parameter the left side removed: the
    /// update beats the removal, so the right side's whole parameter comes
    /// back and the collision is reported. Without a left culprit there is
    /// nothing to restore over, and the edit is dropped.
    fn restore_param(&mut self, b: usize, r: usize, param: &str, action: &VcardMergeAction<'a>) {
        let key = param.to_ascii_uppercase();
        let culprit = self
            .left_ops_on(b)
            .find(|(_, action)| {
                matches!(
                    action.slot(),
                    Slot::Param(k) | Slot::ParamItems(k) if k == key,
                )
            })
            .map(|(_, action)| action.clone());

        let Some(culprit) = culprit else {
            return;
        };

        let node = self.right.props[self.right_insts[r].line]
            .params
            .iter()
            .find(|node| node.name.get().eq_ignore_ascii_case(param))
            .cloned();

        if let Some(node) = node {
            let line = self.left_line(b);
            self.merged.props[line].params.push(node);
        }

        self.record(culprit, action);
    }

    /// Record one conflict pair.
    fn record(&mut self, left: VcardMergeAction<'a>, right: &VcardMergeAction<'a>) {
        self.conflicts.push(VcardMergeConflict {
            left,
            right: right.clone(),
        });
    }
}

/// Re-encode a value node for `escaper`, so a value replayed from a card of
/// another version arrives with the escaping its new card reads.
///
/// vCard 2.1 escapes only `;`, while the later versions also escape a
/// backslash, a comma and a newline, so copying the bytes across would change
/// what the value means. A node already written for `escaper` is cloned
/// unchanged, which is every merge of one version's cards.
fn transcode<'a>(node: &VcardValueNode<'a>, escaper: VcardEscaper) -> VcardValueNode<'a> {
    if node.escaper == escaper {
        return node.clone();
    }

    let mut out = VcardValueNode::from_components(Vec::new(), escaper);

    for i in 0..node.component_count() {
        out.set_at(i, &node.decode_at(i));
    }

    out
}

/// Give every line of a card but its last a line ending.
///
/// A card read without a trailing break leaves its final line with an empty
/// ending, which is right only while that line stays last: a line serializes
/// as its name, parameters, value and ending with nothing in between, so
/// anything written after an unterminated line would land inside its value.
fn terminate_lines(cst: &mut VcardCst<'_>) {
    let mut lines: Vec<&mut VcardLine<'_>> = cst
        .begin
        .iter_mut()
        .chain(cst.props.iter_mut())
        .chain(cst.end.iter_mut())
        .collect();

    let Some((_, rest)) = lines.split_last_mut() else {
        return;
    };

    for line in rest {
        if line.eol.get().is_empty() {
            line.eol = VcardLeaf::from("\r\n");
        }
    }
}

/// The first parameter node of a line whose name matches the given key,
/// mutably, for item-level replay.
fn param_node_mut<'l, 'a>(
    line: &'l mut VcardLine<'a>,
    key: &str,
) -> Option<&'l mut VcardParamNode<'a>> {
    line.params
        .iter_mut()
        .find(|node| node.name.get().eq_ignore_ascii_case(key))
}

/// Whether a property name may appear at most once in a card of the given
/// version (an unknown or grouped name is taken as repeatable).
fn at_most_one(key: &str, version: VcardVersion) -> bool {
    let Ok(kind) = key.parse::<VcardPropKind>() else {
        return false;
    };

    matches!(
        (prop_spec(kind).cardinality)(version),
        VcardPropCardinality::ExactlyOne | VcardPropCardinality::AtMostOne,
    )
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::tree::{cst::VcardCst, merge::VcardMergeAction, merge::merge};

    fn card(props: &str) -> alloc::string::String {
        alloc::format!("BEGIN:VCARD\r\nVERSION:4.0\r\n{props}END:VCARD\r\n")
    }

    #[test]
    fn merges_disjoint_edits_byte_preservingly() {
        // NOTE: the lowercase, oddly-spelled n line proves byte preservation:
        // neither side touches it, so it must survive verbatim.
        let base = card("FN:John Doe\r\nn;pid=1:Doe;John;;;\r\nTEL:+1\r\nNOTE:hi\r\n");
        let left = card("FN:Jane Doe\r\nn;pid=1:Doe;John;;;\r\nTEL:+1\r\nNOTE:hi\r\n");
        let right = card("FN:John Doe\r\nn;pid=1:Doe;John;;;\r\nTEL:+2\r\nEMAIL:j@doe.example\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);

        assert_eq!(
            report.merged.to_string(),
            card("FN:Jane Doe\r\nn;pid=1:Doe;John;;;\r\nTEL:+2\r\nEMAIL:j@doe.example\r\n"),
        );
        assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);

        assert_eq!(report.left.len(), 1);
        assert!(matches!(
            &report.left[0],
            VcardMergeAction::ValueChanged { at, .. } if at.name == "FN",
        ));

        assert_eq!(report.right.len(), 3);
        assert!(matches!(
            &report.right[0],
            VcardMergeAction::ValueChanged { at, .. } if at.name == "TEL",
        ));
        assert!(matches!(
            &report.right[1],
            VcardMergeAction::PropRemoved { at, .. } if at.name == "NOTE",
        ));
        assert!(matches!(
            &report.right[2],
            VcardMergeAction::PropAdded { at, .. } if at.name == "EMAIL",
        ));
    }

    #[test]
    fn identical_edits_do_not_conflict() {
        let base = card("FN:A\r\n");
        let side = card("FN:B\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&side).unwrap();
        let right = VcardCst::parse(&side).unwrap();

        let report = merge(&base, &left, &right);

        assert!(report.merged.to_string().contains("FN:B\r\n"));
        assert!(report.conflicts.is_empty());
    }

    #[test]
    fn divergent_edits_conflict_and_the_left_wins() {
        let base = card("FN:A\r\n");
        let left = card("FN:B\r\n");
        let right = card("FN:C\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);

        assert!(report.merged.to_string().contains("FN:B\r\n"));
        assert_eq!(report.conflicts.len(), 1);
        assert!(matches!(
            &report.conflicts[0].right,
            VcardMergeAction::ValueChanged { at, .. } if at.name == "FN",
        ));
    }

    #[test]
    fn an_update_wins_over_a_removal() {
        let base = card("FN:X\r\nTEL:+1\r\n");
        let removed = card("FN:X\r\n");
        let updated = card("FN:X\r\nTEL:+2\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let removed = VcardCst::parse(&removed).unwrap();
        let updated = VcardCst::parse(&updated).unwrap();

        // NOTE: the left removed what the right updated: the update is
        // restored.
        let report = merge(&base, &removed, &updated);
        assert!(report.merged.to_string().contains("TEL:+2\r\n"));
        assert_eq!(report.conflicts.len(), 1);

        // NOTE: and symmetrically, the left update survives the right
        // removal.
        let report = merge(&base, &updated, &removed);
        assert!(report.merged.to_string().contains("TEL:+2\r\n"));
        assert_eq!(report.conflicts.len(), 1);
    }

    #[test]
    fn list_items_merge_as_a_set() {
        let base = card("NICKNAME:a,b\r\n");
        let left = card("NICKNAME:a,b,c\r\n");
        let right = card("NICKNAME:b\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);

        assert!(report.merged.to_string().contains("NICKNAME:b,c\r\n"));
        assert!(report.conflicts.is_empty());
        assert!(matches!(
            &report.left[0],
            VcardMergeAction::ValueItemAdded { item, .. } if item == "c",
        ));
        assert!(matches!(
            &report.right[0],
            VcardMergeAction::ValueItemRemoved { item, .. } if item == "a",
        ));
    }

    #[test]
    fn structured_components_merge_one_by_one() {
        let base = card("N:Doe;John;;;\r\n");
        let left = card("N:Doe;Johnny;;;\r\n");
        let right = card("N:Smith;John;;;\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);

        assert!(report.merged.to_string().contains("N:Smith;Johnny;;;\r\n"));
        assert!(report.conflicts.is_empty());
        assert!(matches!(
            &report.right[0],
            VcardMergeAction::ValueComponentChanged { component: 0, .. },
        ));
    }

    #[test]
    fn params_merge_by_key_and_list_params_per_item() {
        let base = card("TEL;TYPE=work:+1\r\n");
        let left = card("TEL;TYPE=work;PREF=1:+1\r\n");
        let right = card("TEL;TYPE=work,cell:+1\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);

        assert!(
            report
                .merged
                .to_string()
                .contains("TEL;TYPE=work,cell;PREF=1:+1\r\n"),
            "got: {}",
            report.merged,
        );
        assert!(report.conflicts.is_empty());
        assert!(matches!(
            &report.left[0],
            VcardMergeAction::ParamAdded { .. },
        ));
        assert!(matches!(
            &report.right[0],
            VcardMergeAction::ParamItemAdded { param, item, .. }
                if param == "TYPE" && item == "cell",
        ));
    }

    #[test]
    fn pid_identity_survives_a_reorder() {
        // NOTE: the left card swapped the two EMAIL lines; PID matching must
        // route the right side's edit onto the PID=2 instance, wherever the
        // left card moved it.
        let base = card("EMAIL;PID=1:a@x\r\nEMAIL;PID=2:b@x\r\n");
        let left = card("EMAIL;PID=2:b@x\r\nEMAIL;PID=1:a@x\r\n");
        let right = card("EMAIL;PID=1:a@x\r\nEMAIL;PID=2:c@x\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert!(merged.contains("EMAIL;PID=2:c@x\r\n"), "got: {merged}");
        assert!(merged.contains("EMAIL;PID=1:a@x\r\n"), "got: {merged}");
        assert!(report.conflicts.is_empty());
    }

    #[test]
    fn divergent_additions_of_a_single_valued_property_conflict() {
        let base = card("FN:X\r\n");
        let left = card("FN:X\r\nUID:urn:a\r\n");
        let right = card("FN:X\r\nUID:urn:b\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert!(merged.contains("UID:urn:a\r\n"), "got: {merged}");
        assert!(!merged.contains("UID:urn:b\r\n"), "got: {merged}");
        assert_eq!(report.conflicts.len(), 1);
    }

    #[test]
    fn an_identical_addition_lands_once() {
        let base = card("FN:X\r\n");
        let side = card("FN:X\r\nEMAIL:x@y\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&side).unwrap();
        let right = VcardCst::parse(&side).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(merged.matches("EMAIL:x@y\r\n").count(), 1, "got: {merged}");
        assert!(report.conflicts.is_empty());
    }

    // NOTE: The three whole-parameter edits below are the right side's replay
    // path: the left side edits the value so the two never meet, and what is
    // asserted is that the right card's parameter lands on the left card's
    // line. The list-item paths are covered by the TYPE test above.

    #[test]
    fn a_right_side_parameter_addition_lands_on_the_left_line() {
        let base = card("TEL;TYPE=work:+1\r\n");
        let left = card("TEL;TYPE=work:+2\r\n");
        let right = card("TEL;TYPE=work;PREF=1:+1\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(merged, card("TEL;TYPE=work;PREF=1:+2\r\n"), "got: {merged}");
        assert!(report.conflicts.is_empty());
        assert!(matches!(
            &report.right[0],
            VcardMergeAction::ParamAdded { param, .. } if param.kind().is_some(),
        ));
    }

    #[test]
    fn a_right_side_parameter_removal_takes_it_off_the_left_line() {
        let base = card("TEL;TYPE=work;PREF=1:+1\r\n");
        let left = card("TEL;TYPE=work;PREF=1:+2\r\n");
        let right = card("TEL;TYPE=work:+1\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(merged, card("TEL;TYPE=work:+2\r\n"), "got: {merged}");
        assert!(report.conflicts.is_empty());
        assert!(matches!(
            &report.right[0],
            VcardMergeAction::ParamRemoved { .. },
        ));
    }

    #[test]
    fn a_right_side_parameter_change_replaces_it_on_the_left_line() {
        let base = card("TEL;PREF=1:+1\r\n");
        let left = card("TEL;PREF=1:+2\r\n");
        let right = card("TEL;PREF=2:+1\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(merged, card("TEL;PREF=2:+2\r\n"), "got: {merged}");
        assert!(report.conflicts.is_empty());
        assert!(matches!(
            &report.right[0],
            VcardMergeAction::ParamChanged { .. },
        ));
    }

    #[test]
    fn divergent_parameter_changes_conflict_and_the_left_wins() {
        let base = card("TEL;PREF=1:+1\r\n");
        let left = card("TEL;PREF=2:+1\r\n");
        let right = card("TEL;PREF=3:+1\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(merged, card("TEL;PREF=2:+1\r\n"), "got: {merged}");
        assert_eq!(report.conflicts.len(), 1);
    }
}
