//! # Three-way merge
//!
//! Diff two divergent edits of a card against their common base and
//! reconcile them into one merged card.
//!
//! [`merge`] is the reconciliation unit a synchronisation engine needs: given
//! a base card and two cards derived from it (left and right), it reports
//! every change each side made relative to the base as a list of
//! [`VcardMergeAction`]s, and builds a merged card. The merged card starts as
//! a clone of the left card, so the left side's edits are present byte for
//! byte; the right side's actions are then replayed onto it through the
//! byte-preserving edit layer ([`crate::tree::value`]), so every field the
//! right side did not touch keeps its exact bytes.
//!
//! Property instances of the same name are matched across cards by `PID`
//! first (the RFC 6350 section 7 synchronisation identity), then by equality,
//! then by position. Changes are diffed at the finest granularity the value
//! shape allows: whole property, whole value, one component of a structured
//! value, one item of a list value, one parameter, one item of a list
//! parameter. List items merge as a set (both sides' additions and removals
//! all apply), so they never conflict.
//!
//! Divergent changes to the same field are conflicts
//! ([`VcardMergeConflict`]): the left action wins in the merged card, except
//! when a removal meets an update, where the update wins (data survives over
//! silent loss). Every conflict is reported either way, so a caller can
//! resolve differently. The merged card keeps the left card's `VERSION`; a
//! version change is not reconciled.

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
        codec::unescape::unescape,
        cst::VcardCst,
        leaf::VcardLeaf,
        line::VcardLine,
        param::node::VcardParamNode,
        prop::{cardinality::VcardPropCardinality, spec::prop_spec},
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

    let left_matching = matching(&base_insts, &left_insts);
    let right_matching = matching(&base_insts, &right_insts);

    let left_ops = diff(base, left, &base_insts, &left_insts, &left_matching);
    let right_ops = diff(base, right, &base_insts, &right_insts, &right_matching);

    let mut merger = Merger {
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
    /// `right`. Right item edits never collide: they replay onto whatever the
    /// merged field holds, presence-guarded.
    fn collides_with(&self, right: &Slot) -> bool {
        match (self, right) {
            (Self::Value, Self::Value | Self::Component(_)) => true,
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
/// the `VERSION` indicator.
fn instances<'a>(cst: &'a VcardCst<'a>) -> Vec<Instance<'a>> {
    let version = cst.version();
    let mut insts: Vec<Instance<'a>> = Vec::new();

    for (line, node) in cst.props.iter().enumerate() {
        let name = node.name.get();
        if name.eq_ignore_ascii_case("VERSION") {
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

/// The instance pairing between the base card and one side.
struct Matching {
    /// The matched (base, side) instance index pairs.
    pairs: Vec<(usize, usize)>,
    /// The side instances with no base counterpart.
    added: Vec<usize>,
    /// The base instances with no side counterpart.
    removed: Vec<usize>,
}

/// Pair the base instances with one side's, per name: by `PID` first (the RFC
/// 6350 section 7 synchronisation identity), then by equality, then by
/// position. Leftovers are additions (side) and removals (base).
fn matching(base: &[Instance<'_>], side: &[Instance<'_>]) -> Matching {
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

        pair_by(&mut base_free, &mut side_free, &mut pairs, |b, s| {
            pids_overlap(&base[b].prop, &side[s].prop)
        });
        pair_by(&mut base_free, &mut side_free, &mut pairs, |b, s| {
            base[b].prop == side[s].prop
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

    if b.prop.value == s.prop.value {
        return;
    }

    match (&b.prop.value, &s.prop.value) {
        (VcardValue::TextList(old), VcardValue::TextList(new)) => {
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
            let old_node = &base.props[b.line].value;
            let new_node = &side.props[s.line].value;
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
                self.additions
                    .push(self.right.props[self.right_insts[r].line].clone());
                self.record(self.left_removed_action(b), action);
            }
            return;
        }

        let line = self.left_line(b);

        match action {
            VcardMergeAction::ValueChanged { .. } => {
                if let Some(colliding) = self.colliding(b, &Slot::Value) {
                    if colliding != action {
                        self.record(colliding.clone(), action);
                    }
                    return;
                }

                let value = self.right.props[self.right_insts[r].line].value.clone();
                self.merged.props[line].value = value;
            }

            VcardMergeAction::ValueComponentChanged { component, new, .. } => {
                if let Some(colliding) = self.colliding(b, &Slot::Component(*component)) {
                    if colliding != action {
                        self.record(colliding.clone(), action);
                    }
                    return;
                }

                self.merged.props[line].value.set_at(*component, new);
            }

            VcardMergeAction::ValueItemAdded { item, .. } => {
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
                if let Some(colliding) = self.colliding(b, &Slot::Param(param_key(param))) {
                    if colliding != action {
                        self.record(colliding.clone(), action);
                    }
                    return;
                }

                if let Some(node) = self.right_param_node(r, param) {
                    self.merged.props[line].params.push(node.clone());
                }
            }

            VcardMergeAction::ParamRemoved { param, .. } => {
                if let Some(colliding) = self.colliding(b, &Slot::Param(param_key(param))) {
                    if colliding != action {
                        self.record(colliding.clone(), action);
                    }
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
                if let Some(colliding) = self.colliding(b, &Slot::Param(param_key(new))) {
                    if colliding != action {
                        self.record(colliding.clone(), action);
                    }
                    return;
                }

                let position = self.merged.props[line]
                    .params
                    .iter()
                    .position(|node| node.decode() == *old);

                if let (Some(i), Some(node)) = (position, self.right_param_node(r, new)) {
                    self.merged.props[line].params[i] = node.clone();
                }
            }

            VcardMergeAction::ParamItemAdded { param, item, .. } => {
                let Some(node) = param_node_mut(&mut self.merged.props[line], param) else {
                    self.param_gone_conflict(b, param, action);
                    return;
                };

                let present = node
                    .values
                    .iter()
                    .any(|value| unescape(value.get()) == item.as_ref());

                if !present {
                    node.values.push(VcardLeaf::from(item.to_string()));
                }
            }

            VcardMergeAction::ParamItemRemoved { param, item, .. } => {
                let Some(node) = param_node_mut(&mut self.merged.props[line], param) else {
                    self.param_gone_conflict(b, param, action);
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
        let VcardMergeAction::PropAdded { prop, .. } = action else {
            return;
        };
        let inst = &self.right_insts[s];

        // NOTE: the left side added the same property: one copy is enough.
        let both_added = self.left_matching.added.iter().any(|&l| {
            let left = &self.left_insts[l];
            left.key == inst.key && left.prop == *prop
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

        self.additions.push(self.right.props[inst.line].clone());
    }

    /// Run the deferred structural changes and return the merged card with
    /// the recorded conflicts. Removals go first (on stable indices), then
    /// each addition lands after the last line sharing its name, or at the
    /// end.
    fn finish(mut self) -> (VcardCst<'a>, Vec<VcardMergeConflict<'a>>) {
        self.removals.sort_unstable();
        self.removals.dedup();
        for &line in self.removals.iter().rev() {
            self.merged.props.remove(line);
        }

        for line in mem::take(&mut self.additions) {
            let after = self
                .merged
                .props
                .iter()
                .rposition(|prop| prop.name.get().eq_ignore_ascii_case(line.name.get()));

            match after {
                Some(i) => self.merged.props.insert(i + 1, line),
                None => self.merged.props.push(line),
            }
        }

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

    /// The right card's raw node of a decoded parameter, for byte-faithful
    /// replay.
    fn right_param_node(&self, r: usize, param: &VcardParam<'_>) -> Option<&'a VcardParamNode<'a>> {
        self.right.props[self.right_insts[r].line]
            .params
            .iter()
            .find(|node| node.decode() == *param)
    }

    /// Record the conflict of a right item edit whose parameter the left side
    /// removed or replaced; without a left culprit (nothing to restore onto),
    /// the edit is silently dropped.
    fn param_gone_conflict(&mut self, b: usize, param: &str, action: &VcardMergeAction<'a>) {
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

        if let Some(culprit) = culprit {
            self.record(culprit, action);
        }
    }

    /// Record one conflict pair.
    fn record(&mut self, left: VcardMergeAction<'a>, right: &VcardMergeAction<'a>) {
        self.conflicts.push(VcardMergeConflict {
            left,
            right: right.clone(),
        });
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
