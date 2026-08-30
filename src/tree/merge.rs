//! # Three-way merge
//!
//! Diff two divergent edits of a card against their common base and reconcile
//! them into one merged card.
//!
//! Given a base card and two cards derived from it (left and right),
//! [`VcardMerge::merge`] reports every change each side made as a list of
//! [`VcardMergeAction`]s and builds the merged card, the reconciliation unit a
//! synchronisation engine needs.
//!
//! That card starts as a clone of the left one, so the left side's edits are
//! present byte for byte; the right side's actions are then replayed onto it
//! through the byte-preserving edit layer ([`crate::tree::value`]), so every
//! field the right side did not touch keeps its exact bytes.
//!
//! ## Ours and theirs
//!
//! The left side is git's `ours` and the right side is git's `theirs`. The
//! left side supplies the baseline, so its folding, its parameter casing and
//! its property order come out untouched, and it keeps its own value where
//! both sides wrote one into a single field.
//!
//! One side answers both questions on purpose. A caller reaches for a merge
//! holding the version it is merging into, and that version is the one it
//! would rather not churn and the one it means to keep.
//!
//! Every collision is reported either way, so a caller wanting the other value
//! puts it to somebody rather than asking the merge to guess.
//!
//! ## What is matched with what
//!
//! Property instances of the same name are matched down one ladder: `PID`, the
//! RFC 6350 section 7 synchronisation identity, then the natural identity of a
//! property whose value names a thing outside the card, then exact bytes and
//! equality, then position.
//!
//! `PID` sits above the natural identity because it is metadata: it survives a
//! value change, so a rename stays a rename, which an identity that is the
//! value cannot do.
//!
//! An identity is compared lowercased and written back exactly, so a URI
//! scheme meets the other case it was written in while the line keeps its own
//! bytes.
//!
//! An identity a same-named sibling repeats tells neither of them apart, so
//! both fall back to their positions, and an instance carrying an identity is
//! never matched with one carrying none.
//!
//! The position rung is safe because the base card is never mutated: an
//! ordinal counted there names the same instance whenever it is resolved.
//!
//! Two values compare on their raw nodes, component by component, never
//! through the decoded model, which reads a non-structured value's first
//! `;`-component alone; across versions they compare on the bytes, since two
//! cards escaping values differently share no decoding.
//!
//! Changes are diffed at the finest granularity the value shape allows: whole
//! property, whole value, one component of a structured value, one item of a
//! list value, one parameter, one item of a list parameter.
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
//!
//! Two sides agree on a change only where they wrote the same bytes. A decode
//! is not injective, so `\N` and `\n` read alike (RFC 6350 section 3.4) while
//! saying different things on the wire, and reading two such lines as one
//! change would drop the difference without a word.
//!
//! The right side's change is instead judged normally, meets the left side's,
//! and is reported.
//!
//! A change that only takes something away wrote no bytes, and what it names
//! lives in the base both sides share, so the change itself settles it. The one
//! exception is `TYPE` and `PID`, whose items are a set rather than a sequence.
//!
//! Divergent changes to the same field are conflicts ([`VcardMergeConflict`]):
//! the left side's action wins in the merged card, except when a removal meets
//! an update, where the update wins at every granularity and whichever side it
//! came from (data survives over silent loss).
//!
//! A change both sides made is no conflict at all. Every conflict is reported,
//! so a caller can resolve differently.
//!
//! The merged card keeps the left card's `VERSION`; a version change is not
//! reconciled, but a value replayed from a card of another version is
//! re-encoded for the merged card's escaping mode, so it arrives meaning what
//! it meant.
//!
//! A `BEGIN` or `END` line is envelope rather than property, so it is never
//! diffed or replayed, and an addition lands among the outer card's lines
//! rather than inside a card embedded in a vCard 2.1 `AGENT`.
//!
//! Every line of the merged card but its last carries a line ending, so the
//! card a caller serializes reads back as itself.

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
        codec::{VcardCodec, mode::VcardEscaper, unescape::unescape_param},
        cst::VcardCst,
        leaf::VcardLeaf,
        line::VcardLine,
        param::node::VcardParamNode,
        prop::{cardinality::VcardPropCardinality, spec::prop_spec},
        value::node::VcardValueNode,
    },
    value::{VcardValue, VcardValueKind, VcardValueUnknown},
    version::VcardVersion,
};

/// A three-way merge waiting to run.
///
/// See the module documentation for the matching, granularity and conflict
/// rules.
pub struct VcardMerge<'a> {
    /// The common ancestor both sides were derived from.
    pub base: &'a VcardCst<'a>,
    /// The side being merged into, git's `ours`. The merged card is built from
    /// its bytes, and a collision neither side settles keeps its value.
    pub left: &'a VcardCst<'a>,
    /// The side being merged in, git's `theirs`. Its changes are replayed onto
    /// the left's bytes.
    pub right: &'a VcardCst<'a>,
}

impl<'a> VcardMerge<'a> {
    /// Run the merge.
    pub fn merge(self) -> VcardMergeReport<'a> {
        let base_insts = instances(self.base);
        let left_insts = instances(self.left);
        let right_insts = instances(self.right);

        let left_matching = matching(self.base, &base_insts, self.left, &left_insts);
        let right_matching = matching(self.base, &base_insts, self.right, &right_insts);

        let left_ops = diff(
            self.base,
            self.left,
            &base_insts,
            &left_insts,
            &left_matching,
        );
        let right_ops = diff(
            self.base,
            self.right,
            &base_insts,
            &right_insts,
            &right_matching,
        );

        let mut merger = Merger {
            escaper: VcardEscaper::for_version(self.left.version()),
            left: self.left,
            right: self.right,
            base_insts: &base_insts,
            left_insts: &left_insts,
            right_insts: &right_insts,
            left_matching: &left_matching,
            right_matching: &right_matching,
            left_ops: &left_ops,
            merged: self.left.clone(),
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
}

/// The outcome of a three-way [`VcardMerge::merge`]: the merged card, each
/// side's actions relative to the base, and the conflicts between them.
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
/// The merged card kept the left side's outcome, except for a removal against
/// an update, where the update's outcome was kept (whichever side it came
/// from).
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
    /// The value that tells the instance from its same-named siblings.
    ///
    /// The address of an `EMAIL`, the URI of an `IMPP` or `PHOTO`, the entity
    /// a `MEMBER` names, lowercased since matching normalises and writing is
    /// exact. `None` where position, not a value, tells the siblings apart.
    pub identity: Option<Cow<'a, str>>,
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
        /// The base value, as [`Unknown`](VcardValue::Unknown) raw components
        /// where the model reads the node truncated.
        old: VcardValue<'a>,
        /// The changed value, as [`Unknown`](VcardValue::Unknown) raw
        /// components where the model reads the node truncated.
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
        /// The parameter's position among the property's parameters of that
        /// name, since one name may be written more than once.
        index: usize,
        /// The added parameter, decoded.
        param: VcardParam<'a>,
    },
    /// A parameter the card removed from a matched property.
    ParamRemoved {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The parameter's position among the property's parameters of that
        /// name, since one name may be written more than once.
        index: usize,
        /// The removed parameter, decoded.
        param: VcardParam<'a>,
    },
    /// A parameter of a matched property changed as a whole.
    ParamChanged {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The parameter's position among the property's parameters of that
        /// name, since one name may be written more than once.
        index: usize,
        /// The base parameter.
        old: VcardParam<'a>,
        /// The changed parameter.
        new: VcardParam<'a>,
    },
    /// One item joined a list parameter (`TYPE`, `PID`).
    ParamItemAdded {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The parameter's position among the property's parameters of that
        /// name, since one name may be written more than once.
        index: usize,
        /// The parameter's canonical name.
        param: Cow<'a, str>,
        /// The added item.
        item: Cow<'a, str>,
    },
    /// One item left a list parameter.
    ParamItemRemoved {
        /// The changed base instance.
        at: VcardPropPath<'a>,
        /// The parameter's position among the property's parameters of that
        /// name, since one name may be written more than once.
        index: usize,
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
            Self::ParamAdded { index, param, .. } | Self::ParamRemoved { index, param, .. } => {
                Slot::Param {
                    name: param_key(param),
                    at: *index,
                }
            }
            Self::ParamChanged { index, new, .. } => Slot::Param {
                name: param_key(new),
                at: *index,
            },
            Self::ParamItemAdded { index, param, .. }
            | Self::ParamItemRemoved { index, param, .. } => Slot::ParamItems {
                name: param.to_ascii_uppercase(),
                at: *index,
            },
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
    /// One whole parameter, by key and by its position among the property's
    /// parameters of that key.
    Param {
        /// The parameter key.
        name: String,
        /// The position among the property's parameters of that key.
        at: usize,
    },
    /// The items of a list parameter, by key and position.
    ParamItems {
        /// The parameter key.
        name: String,
        /// The position among the property's parameters of that key.
        at: usize,
    },
}

impl Slot {
    /// Whether a left action on this slot collides with a right one on `right`.
    ///
    /// Two item edits merge as a set and never collide; an item edit against a
    /// whole-value change does, either way round, since one value has to go.
    /// Two parameters of one name are two slots (RFC 2426 4), never rivals.
    fn collides_with(&self, right: &Slot) -> bool {
        match (self, right) {
            (Self::Value, Self::Value | Self::Component(_) | Self::Items) => true,
            (Self::Component(_) | Self::Items, Self::Value) => true,
            (Self::Component(left), Self::Component(right)) => left == right,
            (
                Self::Param {
                    name: left,
                    at: left_at,
                }
                | Self::ParamItems {
                    name: left,
                    at: left_at,
                },
                Self::Param {
                    name: right,
                    at: right_at,
                },
            ) => left == right && left_at == right_at,
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
    /// The value that tells the instance from its same-named siblings, where
    /// vCard gives it one, normalised for comparison. `None` for every other
    /// property, and for a value a same-named sibling repeats.
    identity: Option<String>,
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

        let identity = identity_of(&key, node);

        insts.push(Instance {
            line,
            nth,
            key,
            identity,
            prop: node.decode(version),
        });
    }

    // NOTE: an identity a same-named sibling repeats is no identity: a value
    // written twice names both instances, so both fall back to their
    // positions, and a sibling still alone with its value keeps its own.
    let repeated: Vec<usize> = insts
        .iter()
        .enumerate()
        .filter(|(at, inst)| {
            inst.identity.is_some()
                && insts.iter().enumerate().any(|(index, other)| {
                    index != *at && other.key == inst.key && other.identity == inst.identity
                })
        })
        .map(|(at, _)| at)
        .collect();

    for at in repeated {
        insts[at].identity = None;
    }

    insts
}

/// The value naming what a property is about, lowercased into a match key.
///
/// A repeatable property whose value names a thing outside the card is that
/// thing. Any other value is the datum edited, so keying on it would make
/// every edit a replacement. A grouped name has none: the group tells it apart.
fn identity_of(key: &str, line: &VcardLine<'_>) -> Option<String> {
    let identified = matches!(
        key.parse::<VcardPropKind>(),
        Ok(VcardPropKind::CalAdrUri
            | VcardPropKind::CalUri
            | VcardPropKind::Email
            | VcardPropKind::FbUrl
            | VcardPropKind::Impp
            | VcardPropKind::Key
            | VcardPropKind::Logo
            | VcardPropKind::Member
            | VcardPropKind::Photo
            | VcardPropKind::Related
            | VcardPropKind::SocialProfile
            | VcardPropKind::Sound
            | VcardPropKind::Source
            | VcardPropKind::Tel
            | VcardPropKind::Url)
    );

    identified.then(|| String::from_utf8_lossy(&raw_value(&line.value)).to_lowercase())
}

/// Whether a line names the card's structure: `VERSION`, `BEGIN` or `END`.
///
/// None of them is a property, so none is diffed, matched, moved or replayed,
/// the `BEGIN` / `END` of a card embedded in a vCard 2.1 `AGENT` included:
/// replaying an `END` would close the merged card early and drop the rest.
fn is_envelope(name: &str) -> bool {
    name.eq_ignore_ascii_case("VERSION")
        || name.eq_ignore_ascii_case("BEGIN")
        || name.eq_ignore_ascii_case("END")
}

/// Which lines belong to an embedded card, its `BEGIN` and `END` included.
///
/// They are diffed like any other property, a change inside an agent still
/// being a change, but an addition is never *placed* among them: it belongs to
/// the card that owns the property name it repeats.
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

/// Pair the base instances with one side's, per name, down the matching ladder.
///
/// The rungs are the `PID` synchronisation identity (RFC 6350 7), the natural
/// identity of a property whose value names a thing outside the card, exact
/// bytes, equality, then position. Leftovers are additions and removals.
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
        // NOTE: `PID` sits above the natural identity, and that order is what
        // keeps a rename a rename: `PID` is metadata, so it survives a value
        // change, while an identity that is the value cannot.
        pair_by(&mut base_free, &mut side_free, &mut pairs, |b, s| {
            base[b].identity.is_some() && base[b].identity == side[s].identity
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

        // NOTE: position only tells apart properties vCard gives no identity
        // of their own. An address that matched nothing names an entry that
        // left, never one the other side renamed.
        let mut b = 0;
        while b < base_free.len() {
            if base[base_free[b]].identity.is_some() {
                b += 1;
                continue;
            }

            match side_free.iter().position(|&s| side[s].identity.is_none()) {
                Some(s) => pairs.push((base_free.remove(b), side_free.remove(s))),
                None => break,
            }
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
        identity: inst.identity.clone().map(Cow::Owned),
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

    diff_params(
        &b.prop.params,
        &s.prop.params,
        &base.props[b.line].params,
        &side.props[s.line].params,
        &at,
        out,
    );

    let old_node = &base.props[b.line].value;
    let new_node = &side.props[s.line].value;

    if value_eq(old_node, new_node) {
        return;
    }

    match (&b.prop.value, &s.prop.value) {
        // NOTE: a list value's items are the whole value split on its commas,
        // while an item action is replayed by splicing one leaf of component
        // zero, so the two only address the same thing while the value is one
        // component; past that the whole value changed.
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
        (old, new) if old.kind() == new.kind() && is_component_structured(old.kind()) => {
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

/// What a value node says, reported whole.
///
/// A decoded value says what its node says only when it encodes back to it:
/// `NOTE:a;b` decodes to the text `a;b` but re-encodes with the semicolon
/// escaped. A value failing that round trip is reported as raw components.
fn whole<'a>(value: &VcardValue<'a>, node: &'a VcardValueNode<'a>) -> VcardValue<'a> {
    if value_eq(&value.encode(node.escaper), node) {
        return value.clone();
    }

    VcardValue::Unknown(VcardValueUnknown::decode(node))
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

/// Whether two instances hold the same property: the same decoded name, and
/// the same parameters and value on the raw nodes.
fn prop_eq<'a>(
    a_cst: &VcardCst<'a>,
    a: &Instance<'a>,
    b_cst: &VcardCst<'a>,
    b: &Instance<'a>,
) -> bool {
    let a_line = &a_cst.props[a.line];
    let b_line = &b_cst.props[b.line];

    a.prop.name == b.prop.name
        && a_line.params.len() == b_line.params.len()
        && a_line
            .params
            .iter()
            .zip(&b_line.params)
            .all(|(a, b)| a.name.get().eq_ignore_ascii_case(b.name.get()) && param_eq(a, b))
        && value_eq(&a_line.value, &b_line.value)
}

/// Whether two raw value nodes hold the same value, component by component.
///
/// The comparison runs on the node rather than on the decoded value, which
/// reads its own kind's shape and would hide a change the kind cannot express:
/// two `data:` URIs differing in their payload alone decode alike.
fn value_eq(old: &VcardValueNode<'_>, new: &VcardValueNode<'_>) -> bool {
    // NOTE: two cards of different versions escape values by different rules,
    // so they share no decoding to compare through: `http\://x` reads as
    // itself in 2.1 and as `http://x` later. Only identical bytes are then
    // certainly the same value.
    if old.escaper != new.escaper {
        return raw_value(old) == raw_value(new);
    }

    let count = old.component_count().max(new.component_count());

    (0..count).all(|i| component_eq(&old.decode_component_list(i), &new.decode_component_list(i)))
}

/// Whether two raw parameter nodes hold the same parameter, value by value.
///
/// On the node rather than the decoded parameter, for the reason [`value_eq`]
/// gives: a single-valued parameter reads its first value alone, so two
/// parameters differing past the first `,` decode alike.
fn param_eq(old: &VcardParamNode<'_>, new: &VcardParamNode<'_>) -> bool {
    // NOTE: two cards of different versions encode parameters by different
    // rules, so they share no decoding to compare through. Only identical
    // bytes are then certainly the same parameter.
    if old.escaper != new.escaper {
        return raw_param(old) == raw_param(new);
    }

    old.values.len() == new.values.len()
        && old
            .values
            .iter()
            .zip(&new.values)
            .all(|(old_value, value)| {
                unescape_param(old_value.get(), old.escaper)
                    == unescape_param(value.get(), new.escaper)
            })
}

/// A parameter node's raw bytes, as it would serialize.
fn raw_param(node: &VcardParamNode<'_>) -> Vec<u8> {
    let mut out = Vec::new();
    node.write_bytes(&mut out);
    out
}

/// A value node's raw bytes, as it would serialize.
fn raw_value(node: &VcardValueNode<'_>) -> Vec<u8> {
    let mut out = Vec::new();
    node.write_bytes(&mut out);
    out
}

/// One decoded parameter alongside the raw node it was decoded from.
type ParamPair<'p, 'a> = (&'p VcardParam<'a>, &'p VcardParamNode<'a>);

/// Diff two matched properties' parameter lists, by name then by position.
///
/// One name may be written more than once (RFC 2426 4 gives
/// `TEL;TYPE=work;TYPE=voice`), each occurrence a field of its own. A single
/// `TYPE` / `PID` on both sides diffs per item instead, those being sets.
fn diff_params<'a>(
    old: &[VcardParam<'a>],
    new: &[VcardParam<'a>],
    old_nodes: &[VcardParamNode<'a>],
    new_nodes: &[VcardParamNode<'a>],
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
        // NOTE: a decode maps the parameters one for one, so each decoded
        // parameter is paired with the raw node it came from and the
        // comparison can run on the node.
        let olds: Vec<ParamPair<'_, 'a>> = old
            .iter()
            .zip(old_nodes)
            .filter(|(param, _)| param_key(param) == key)
            .collect();
        let news: Vec<ParamPair<'_, 'a>> = new
            .iter()
            .zip(new_nodes)
            .filter(|(param, _)| param_key(param) == key)
            .collect();

        if olds.len() == news.len()
            && olds
                .iter()
                .zip(&news)
                .all(|((_, old), (_, new))| param_eq(old, new))
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
            if !param_eq(olds[i].1, news[i].1) {
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
        ) => left_at == right_at && left_index == right_index && same_param(left, right),

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
                && same_param(left_old, right_old)
                && same_param(left_new, right_new)
        }

        (left, right) => left == right,
    }
}

/// Whether two parameters carry the same value, a list parameter's items
/// compared as an unordered set: see [`unordered`].
fn same_param(left: &VcardParam<'_>, right: &VcardParam<'_>) -> bool {
    match (left, right) {
        (VcardParam::Type(left), VcardParam::Type(right))
        | (VcardParam::Pid(left), VcardParam::Pid(right)) => sorted(left) == sorted(right),
        (left, right) => left == right,
    }
}

/// Whether a parameter's items are a set rather than a sequence.
///
/// `TYPE` (RFC 6350 5.6) and `PID` (7) each hold a list the specification
/// gives no order, so two sides writing their items in two orders wrote one
/// parameter.
fn unordered(param: &VcardParam<'_>) -> bool {
    matches!(param, VcardParam::Type(_) | VcardParam::Pid(_))
}

/// A list parameter's items in a stable order, for comparing them as a set.
fn sorted<'v>(values: &'v [Cow<'_, str>]) -> Vec<&'v str> {
    let mut items: Vec<&str> = values.iter().map(Cow::as_ref).collect();
    items.sort_unstable();
    items
}

/// Whether two sides spelled one parameter the same way on the wire.
///
/// A list parameter the specification gives no order compares as a set of raw
/// items, for the reason [`unordered`] gives; every other parameter compares
/// whole, so how it is written is part of what it says.
fn param_alike(
    ours: &VcardLine<'_>,
    theirs: &VcardLine<'_>,
    param: &VcardParam<'_>,
    index: usize,
) -> bool {
    let key = param_key(param);

    let (Some(ours), Some(theirs)) = (
        param_node(ours, &key, index),
        param_node(theirs, &key, index),
    ) else {
        return false;
    };

    if !unordered(param) {
        return raw_param(ours) == raw_param(theirs);
    }

    let raw = |node: &VcardParamNode<'_>| {
        let mut items: Vec<String> = node
            .values
            .iter()
            .map(|leaf| leaf.get().to_string())
            .collect();
        items.sort_unstable();
        items
    };

    ours.name.get().eq_ignore_ascii_case(theirs.name.get()) && raw(ours) == raw(theirs)
}

/// The still-encoded form one item of a list parameter was written as.
fn raw_param_item<'l>(
    line: &'l VcardLine<'_>,
    param: &str,
    index: usize,
    item: &str,
) -> Option<&'l str> {
    let node = param_node(line, param, index)?;

    node.values
        .iter()
        .find(|value| unescape_param(value.get(), node.escaper) == item)
        .map(|leaf| leaf.get())
}

/// Whether two sides spelled one item of a list value the same way on the
/// wire.
fn item_alike(ours: &VcardValueNode<'_>, theirs: &VcardValueNode<'_>, item: &str) -> bool {
    let raw = |node: &VcardValueNode<'_>| -> Option<Vec<u8>> {
        let at = node
            .decode_list()
            .iter()
            .position(|held| held.as_ref() == item)?;

        node.raw_list().into_iter().nth(at)
    };

    match (raw(ours), raw(theirs)) {
        (Some(ours), Some(theirs)) => ours == theirs,
        _ => false,
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
    escaper: VcardEscaper,
    left: &'a VcardCst<'a>,
    right: &'a VcardCst<'a>,
    base_insts: &'o [Instance<'a>],
    left_insts: &'o [Instance<'a>],
    right_insts: &'o [Instance<'a>],
    left_matching: &'o Matching,
    right_matching: &'o Matching,
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
                if self.already_made(b, r, action) {
                    return;
                }

                let right_value = &self.right.props[self.right_insts[r].line].value;

                if let Some(colliding) = self.colliding(b, &Slot::Value) {
                    let colliding = colliding.clone();
                    self.record(colliding, action);
                    return;
                }

                self.merged.props[line].value = transcode(right_value, self.escaper);
            }

            VcardMergeAction::ValueComponentChanged { component, new, .. } => {
                if self.already_made(b, r, action) {
                    return;
                }

                if let Some(colliding) = self.colliding(b, &Slot::Component(*component)) {
                    let colliding = colliding.clone();
                    self.record(colliding, action);
                    return;
                }

                self.merged.props[line].value.set_component(*component, new);
            }

            VcardMergeAction::ValueItemAdded { item, .. } => {
                if self.already_made(b, r, action) {
                    return;
                }

                if let Some(colliding) = self.colliding(b, &Slot::Items) {
                    let colliding = colliding.clone();
                    self.record(colliding, action);
                    return;
                }

                // NOTE: component zero is named on purpose here, and read the
                // same way it is written: an item is spliced as one leaf, so
                // the leaves it is looked up among must be the same ones. The
                // diff only raises an item action for a one-component value.
                let value = &mut self.merged.props[line].value;
                let present = value
                    .decode_component_list(0)
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
                if self.already_made(b, r, action) {
                    return;
                }

                if let Some(colliding) = self.colliding(b, &Slot::Items) {
                    let colliding = colliding.clone();
                    self.record(colliding, action);
                    return;
                }

                let value = &mut self.merged.props[line].value;
                let position = value
                    .decode_component_list(0)
                    .iter()
                    .position(|value| value.as_ref() == item.as_ref());

                if let Some(j) = position {
                    value.remove_value_at(0, j);
                }
            }

            VcardMergeAction::ParamAdded { index, param, .. } => {
                if self.already_made(b, r, action) {
                    return;
                }

                // NOTE: an update beats a removal here as it does at property
                // granularity, so the addition still lands when all the left
                // side did was remove the parameter.
                let slot = Slot::Param {
                    name: param_key(param),
                    at: *index,
                };

                if let Some(colliding) = self.colliding(b, &slot) {
                    let removed = matches!(colliding, VcardMergeAction::ParamRemoved { .. });
                    let colliding = colliding.clone();
                    self.record(colliding, action);

                    if !removed {
                        return;
                    }
                }

                let Some(node) = self.right_param_node(r, param, *index).cloned() else {
                    return;
                };

                self.merged.props[line].params.push(node);
            }

            VcardMergeAction::ParamRemoved { index, param, .. } => {
                if self.already_made(b, r, action) {
                    return;
                }

                let slot = Slot::Param {
                    name: param_key(param),
                    at: *index,
                };

                if let Some(colliding) = self.colliding(b, &slot) {
                    let colliding = colliding.clone();
                    self.record(colliding, action);
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

            VcardMergeAction::ParamChanged {
                index, old, new, ..
            } => {
                if self.already_made(b, r, action) {
                    return;
                }

                let mut restore = false;
                let slot = Slot::Param {
                    name: param_key(new),
                    at: *index,
                };

                if let Some(colliding) = self.colliding(b, &slot) {
                    let colliding = colliding.clone();
                    restore = matches!(colliding, VcardMergeAction::ParamRemoved { .. });
                    self.record(colliding, action);

                    if !restore {
                        return;
                    }
                }

                let position = self.merged.props[line]
                    .params
                    .iter()
                    .position(|node| node.decode() == *old);

                if let Some(node) = self.right_param_node(r, new, *index) {
                    match (position, restore) {
                        (Some(i), _) => self.merged.props[line].params[i] = node.clone(),
                        // NOTE: the left side removed the parameter this
                        // update rewrote, so the update brings it back.
                        (None, true) => self.merged.props[line].params.push(node.clone()),
                        (None, false) => {}
                    }
                }
            }

            VcardMergeAction::ParamItemAdded {
                index, param, item, ..
            } => {
                if self.already_made(b, r, action) {
                    return;
                }

                // NOTE: the item has to land as the right card spelled it: a
                // decoded item holds a real line break where the wire holds
                // `^n`, and written decoded it would cut the line in two.
                let leaf = self.right_param_item(r, param, *index, item);

                let Some(node) = param_node_mut(&mut self.merged.props[line], param, *index) else {
                    self.restore_param(b, r, param, *index, action);
                    return;
                };

                let present = node
                    .values
                    .iter()
                    .any(|value| unescape_param(value.get(), node.escaper) == item.as_ref());

                if let Some(leaf) = leaf
                    && !present
                {
                    node.values.push(leaf);
                }
            }

            VcardMergeAction::ParamItemRemoved {
                index, param, item, ..
            } => {
                // NOTE: as above, a parameter may hold one item twice, and
                // `TYPE=work,,` is exactly that.
                if self.already_made(b, r, action) {
                    return;
                }

                let Some(node) = param_node_mut(&mut self.merged.props[line], param, *index) else {
                    self.restore_param(b, r, param, *index, action);
                    return;
                };

                let position = node
                    .values
                    .iter()
                    .position(|value| unescape_param(value.get(), node.escaper) == item.as_ref());

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

    /// The left arrival standing over the same departed base instance as `s`.
    ///
    /// A property whose identity is its own value cannot be seen to change, so
    /// an edit is a departure plus an arrival: two arrivals over one departure
    /// both sides agreed on are one instance edited twice, which collides.
    fn paired_arrival(&self, key: &str, s: usize) -> Option<usize> {
        let gone = |matching: &Matching| {
            matching
                .removed
                .iter()
                .copied()
                .filter(|b| self.base_insts[*b].key == key)
                .collect::<Vec<_>>()
        };
        let new = |matching: &Matching, insts: &[Instance<'a>]| {
            matching
                .added
                .iter()
                .copied()
                .filter(|a| insts[*a].key == key)
                .collect::<Vec<_>>()
        };

        let rank = new(self.right_matching, self.right_insts)
            .iter()
            .position(|&a| a == s)?;
        let base = *gone(self.right_matching).get(rank)?;
        let ours = gone(self.left_matching).iter().position(|&b| b == base)?;

        new(self.left_matching, self.left_insts).get(ours).copied()
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

        // NOTE: both sides wrote over one slot: a name allowed at most once,
        // or an instance both sides took away and each replaced. The left
        // side's copy is the one the card keeps, and the loser does not join
        // it on the card.
        let contested = self.paired_arrival(&inst.key, s).or_else(|| {
            match at_most_one(&inst.key, self.merged.version()) {
                true => self
                    .left_matching
                    .added
                    .iter()
                    .copied()
                    .find(|l| self.left_insts[*l].key == inst.key),
                false => None,
            }
        });

        if let Some(l) = contested {
            let colliding = self
                .left_ops
                .iter()
                .find(|(target, _)| matches!(target, Target::Added(x) if *x == l))
                .map(|(_, action)| action.clone());

            if let Some(colliding) = colliding {
                self.record(colliding, action);
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
    fn already_made(&self, b: usize, r: usize, action: &VcardMergeAction<'a>) -> bool {
        self.left_ops_on(b)
            .any(|(_, left)| same_change(left, action))
            && self.wrote_alike(b, r, action)
    }

    /// Whether the two sides put the same bytes on the wire for one change.
    ///
    /// A decode is not injective, so two sides writing different bytes can
    /// decode alike, and calling that one change drops a difference without a
    /// word. A removal wrote nothing, so the change itself settles it.
    fn wrote_alike(&self, b: usize, r: usize, action: &VcardMergeAction<'a>) -> bool {
        let ours = &self.left.props[self.left_line(b)];
        let theirs = &self.right.props[self.right_insts[r].line];

        match action {
            VcardMergeAction::PropAdded { .. } | VcardMergeAction::PropRemoved { .. } => false,
            VcardMergeAction::ValueItemRemoved { .. }
            | VcardMergeAction::ParamRemoved { .. }
            | VcardMergeAction::ParamItemRemoved { .. } => true,
            VcardMergeAction::ValueChanged { .. } => {
                raw_value(&ours.value) == raw_value(&theirs.value)
            }
            VcardMergeAction::ValueComponentChanged { component, .. } => {
                ours.value.raw_component_list(*component)
                    == theirs.value.raw_component_list(*component)
            }
            VcardMergeAction::ValueItemAdded { item, .. } => {
                item_alike(&ours.value, &theirs.value, item)
            }
            VcardMergeAction::ParamAdded { index, param, .. }
            | VcardMergeAction::ParamChanged {
                index, new: param, ..
            } => param_alike(ours, theirs, param, *index),
            VcardMergeAction::ParamItemAdded {
                index, param, item, ..
            } => {
                let ours = raw_param_item(ours, param, *index, item);

                ours.is_some() && ours == raw_param_item(theirs, param, *index, item)
            }
        }
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

    /// The right card's raw leaf for one item of the `index`th list parameter.
    ///
    /// The decoded text is no wire form: an item holds a real line break where
    /// the wire holds `^n`, so writing it back decoded would cut the line in
    /// two. The item was decoded from that node, so the leaf is there.
    fn right_param_item(
        &self,
        r: usize,
        param: &str,
        index: usize,
        item: &str,
    ) -> Option<VcardLeaf<'a>> {
        let line = &self.right.props[self.right_insts[r].line];
        let node = param_node(line, param, index)?;

        node.values
            .iter()
            .find(|value| unescape_param(value.get(), node.escaper) == item)
            .cloned()
    }

    /// The right card's raw parameter node, for byte-faithful replay.
    ///
    /// Addressed by name and ordinal, which is how the action was raised: a
    /// decoded parameter is no key, two same-named ones differing past their
    /// first value decoding alike.
    fn right_param_node(
        &self,
        r: usize,
        param: &VcardParam<'_>,
        index: usize,
    ) -> Option<&'a VcardParamNode<'a>> {
        let key = param_key(param);

        self.right.props[self.right_insts[r].line]
            .params
            .iter()
            .filter(|node| node.name.get().eq_ignore_ascii_case(&key))
            .nth(index)
    }

    /// Replay a right item edit whose parameter the left side removed: the
    /// update beats the removal, so the right side's whole parameter comes
    /// back and the collision is reported. Without a left culprit there is
    /// nothing to restore over, and the edit is dropped.
    fn restore_param(
        &mut self,
        b: usize,
        r: usize,
        param: &str,
        index: usize,
        action: &VcardMergeAction<'a>,
    ) {
        let key = param.to_ascii_uppercase();
        let culprit = self
            .left_ops_on(b)
            .find(|(_, action)| {
                matches!(
                    action.slot(),
                    Slot::Param { name, at } | Slot::ParamItems { name, at }
                        if name == key && at == index,
                )
            })
            .map(|(_, action)| action.clone());

        let Some(culprit) = culprit else {
            return;
        };

        let node = self.right.props[self.right_insts[r].line]
            .params
            .iter()
            .filter(|node| node.name.get().eq_ignore_ascii_case(param))
            .nth(index)
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

/// Re-encode a value node for `escaper`, the escaping its new card reads.
///
/// vCard 2.1 escapes only `;` while the later versions also escape a
/// backslash, a comma and a newline, so copying across the bytes of a value
/// replayed from another version's card would change what it means.
fn transcode<'a>(node: &VcardValueNode<'a>, escaper: VcardEscaper) -> VcardValueNode<'a> {
    if node.escaper == escaper {
        return node.clone();
    }

    let mut out = VcardValueNode::from_components(Vec::new(), escaper);

    for i in 0..node.component_count() {
        out.set_component(i, &node.decode_component_list(i));
    }

    out
}

/// Give every line of a card but its last a line ending.
///
/// A card read without a trailing break leaves its final line with an empty
/// ending, right only while that line stays last: a line serializes with
/// nothing between its parts, so anything after it would land in its value.
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

/// The `at`th parameter node of a line whose name matches the given key.
fn param_node<'l, 'a>(
    line: &'l VcardLine<'a>,
    key: &str,
    at: usize,
) -> Option<&'l VcardParamNode<'a>> {
    line.params
        .iter()
        .filter(|node| node.name.get().eq_ignore_ascii_case(key))
        .nth(at)
}

/// The same, mutably, for item-level replay.
fn param_node_mut<'l, 'a>(
    line: &'l mut VcardLine<'a>,
    key: &str,
    at: usize,
) -> Option<&'l mut VcardParamNode<'a>> {
    line.params
        .iter_mut()
        .filter(|node| node.name.get().eq_ignore_ascii_case(key))
        .nth(at)
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
    use alloc::{borrow::Cow, string::ToString};

    use crate::{
        tree::{
            cst::VcardCst,
            merge::{VcardMerge, VcardMergeAction, VcardMergeReport},
        },
        value::{VcardValue, VcardValueUnknown},
    };

    fn card(props: &str) -> alloc::string::String {
        alloc::format!("BEGIN:VCARD\r\nVERSION:4.0\r\n{props}END:VCARD\r\n")
    }

    fn components(components: &[&[&str]]) -> VcardValue<'static> {
        let components = components
            .iter()
            .map(|values| {
                values
                    .iter()
                    .map(|value| Cow::Owned(value.to_string()))
                    .collect()
            })
            .collect();

        VcardValue::Unknown(VcardValueUnknown { components })
    }

    fn merge<'a>(
        base: &'a VcardCst<'a>,
        left: &'a VcardCst<'a>,
        right: &'a VcardCst<'a>,
    ) -> VcardMergeReport<'a> {
        VcardMerge { base, left, right }.merge()
    }

    #[test]
    fn merges_disjoint_edits_byte_preservingly() {
        // NOTE: the lowercase, oddly-spelled n line proves byte preservation:
        // neither side touches it, so it must survive verbatim.
        let base = card("FN:John Doe\r\nn;pid=1:Doe;John;;;\r\nTITLE:dev\r\nNOTE:hi\r\n");
        let left = card("FN:Jane Doe\r\nn;pid=1:Doe;John;;;\r\nTITLE:dev\r\nNOTE:hi\r\n");
        let right =
            card("FN:John Doe\r\nn;pid=1:Doe;John;;;\r\nTITLE:boss\r\nEMAIL:j@doe.example\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);

        assert_eq!(
            report.merged.to_string(),
            card("FN:Jane Doe\r\nn;pid=1:Doe;John;;;\r\nTITLE:boss\r\nEMAIL:j@doe.example\r\n"),
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
            VcardMergeAction::ValueChanged { at, .. } if at.name == "TITLE",
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
    fn a_value_change_past_the_first_component_reports_both_values() {
        let base = card("NOTE:a;b\r\n");
        let left = card("NOTE:a;b\r\n");
        let right = card("NOTE:a;CHANGED\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(merged, card("NOTE:a;CHANGED\r\n"), "got: {merged}");
        assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);

        let VcardMergeAction::ValueChanged { old, new, .. } = &report.right[0] else {
            panic!("expected a value change, got: {:?}", report.right[0]);
        };

        assert_eq!(old, &components(&[&["a"], &["b"]]));
        assert_eq!(new, &components(&[&["a"], &["CHANGED"]]));
    }

    #[test]
    fn an_update_wins_over_a_removal() {
        let base = card("FN:X\r\nNOTE:a\r\n");
        let removed = card("FN:X\r\n");
        let updated = card("FN:X\r\nNOTE:b\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let removed = VcardCst::parse(&removed).unwrap();
        let updated = VcardCst::parse(&updated).unwrap();

        // NOTE: the left removed what the right updated: the update is
        // restored.
        let report = merge(&base, &removed, &updated);
        assert!(report.merged.to_string().contains("NOTE:b\r\n"));
        assert_eq!(report.conflicts.len(), 1);

        // NOTE: and symmetrically, the left update survives the right
        // removal.
        let report = merge(&base, &updated, &removed);
        assert!(report.merged.to_string().contains("NOTE:b\r\n"));
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

    /// A repeatable property whose identity is its value cannot be seen to
    /// change, so the matching reads an edit as a departure and an arrival.
    /// Two arrivals over one agreed departure are one instance edited twice.
    #[test]
    fn divergent_edits_of_an_identity_keyed_property_conflict() {
        let base = card("FN:A\r\nTEL:+1\r\n");
        let left = card("FN:A\r\nTEL:+2\r\n");
        let right = card("FN:A\r\nTEL:+3\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(report.conflicts.len(), 1, "{:?}", report.conflicts);
        assert!(merged.contains("TEL:+2\r\n"), "got: {merged}");
        assert!(
            !merged.contains("TEL:+3\r\n"),
            "the loser does not join the winner: {merged}",
        );
    }

    /// Nothing departed, so the two arrivals are two additions and the set
    /// keeps both. This is the case the collision rule must not swallow.
    #[test]
    fn divergent_additions_beside_an_untouched_instance_merge_as_a_set() {
        let base = card("FN:A\r\nTEL:+1\r\n");
        let left = card("FN:A\r\nTEL:+1\r\nTEL:+2\r\n");
        let right = card("FN:A\r\nTEL:+1\r\nTEL:+3\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
        assert!(merged.contains("TEL:+2\r\n"), "got: {merged}");
        assert!(merged.contains("TEL:+3\r\n"), "got: {merged}");
    }

    /// One side edited the instance and the other left it alone, so there is
    /// no contest: the edit stands and the addition joins it.
    #[test]
    fn an_edit_on_one_side_alone_does_not_contest_the_other_s_addition() {
        let base = card("FN:A\r\nTEL:+1\r\n");
        let left = card("FN:A\r\nTEL:+2\r\n");
        let right = card("FN:A\r\nTEL:+1\r\nTEL:+3\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
        assert!(merged.contains("TEL:+3\r\n"), "got: {merged}");
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
        let base = card("FN:X\r\nTEL;TYPE=work:+1\r\n");
        let left = card("FN:Y\r\nTEL;TYPE=work:+1\r\n");
        let right = card("FN:X\r\nTEL;TYPE=work;PREF=1:+1\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(
            merged,
            card("FN:Y\r\nTEL;TYPE=work;PREF=1:+1\r\n"),
            "got: {merged}",
        );
        assert!(report.conflicts.is_empty());
        assert!(matches!(
            &report.right[0],
            VcardMergeAction::ParamAdded { param, .. } if param.kind().is_some(),
        ));
    }

    #[test]
    fn a_right_side_parameter_removal_takes_it_off_the_left_line() {
        let base = card("FN:X\r\nTEL;TYPE=work;PREF=1:+1\r\n");
        let left = card("FN:Y\r\nTEL;TYPE=work;PREF=1:+1\r\n");
        let right = card("FN:X\r\nTEL;TYPE=work:+1\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(
            merged,
            card("FN:Y\r\nTEL;TYPE=work:+1\r\n"),
            "got: {merged}"
        );
        assert!(report.conflicts.is_empty());
        assert!(matches!(
            &report.right[0],
            VcardMergeAction::ParamRemoved { .. },
        ));
    }

    #[test]
    fn a_right_side_parameter_change_replaces_it_on_the_left_line() {
        let base = card("FN:X\r\nTEL;PREF=1:+1\r\n");
        let left = card("FN:Y\r\nTEL;PREF=1:+1\r\n");
        let right = card("FN:X\r\nTEL;PREF=2:+1\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(merged, card("FN:Y\r\nTEL;PREF=2:+1\r\n"), "got: {merged}");
        assert!(report.conflicts.is_empty());
        assert!(matches!(
            &report.right[0],
            VcardMergeAction::ParamChanged { .. },
        ));
    }

    #[test]
    fn two_parameters_of_one_name_are_two_fields() {
        // NOTE: RFC 2426 section 4 writes a repeated parameter name as
        // `TEL;TYPE=work;TYPE=voice`, so the side rewriting the first
        // parameter contests nothing the side rewriting the second wrote.
        let base = card("TEL;TYPE=work;TYPE=voice:+1\r\n");
        let left = card("TEL;TYPE=home;TYPE=voice:+1\r\n");
        let right = card("TEL;TYPE=work;TYPE=fax:+1\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(
            merged,
            card("TEL;TYPE=home;TYPE=fax:+1\r\n"),
            "got: {merged}",
        );
        assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
        assert!(matches!(
            &report.right[0],
            VcardMergeAction::ParamChanged { index: 1, .. },
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

    #[test]
    fn a_side_that_reordered_and_replaced_an_address_is_the_merged_card() {
        let base = card("EMAIL;TYPE=work:ada@x.test\r\nEMAIL;TYPE=home:zoe@x.test\r\n");
        let right = card("EMAIL;TYPE=home:zoe@x.test\r\nEMAIL;TYPE=work:bob@x.test\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let held = base.to_string();
        let left = VcardCst::parse(&held).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);

        assert_eq!(report.merged.to_bytes(), right.to_bytes());
        assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
    }

    #[test]
    fn an_edit_is_never_recorded_against_a_replacement() {
        // NOTE: the left side replaced Ada with Bob while the right side set
        // Ada's type. An address is the instance, so Ada's edit never lands
        // on Bob: it brings Ada's line back, and the collision is reported.
        let base = card("EMAIL:ada@x.test\r\n");
        let left = card("EMAIL:bob@x.test\r\n");
        let right = card("EMAIL;TYPE=work:ada@x.test\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let merged = merge(&base, &left, &right).merged.to_string();

        assert!(merged.contains("EMAIL:bob@x.test\r\n"), "got: {merged}");
        assert!(
            merged.contains("EMAIL;TYPE=work:ada@x.test\r\n"),
            "got: {merged}",
        );
    }

    #[test]
    fn one_address_written_twice_tells_neither_instance_apart() {
        // NOTE: an identity a same-named sibling repeats is no identity, so
        // both fall back to their positions and the edit still lands.
        // NOTE: which of two interchangeable copies carries the edit is not
        // pinned, so the assertion is on the content rather than the order.
        let base = card("EMAIL:ada@x.test\r\nEMAIL:ada@x.test\r\n");
        let right = card("EMAIL;TYPE=work:ada@x.test\r\nEMAIL:ada@x.test\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let held = base.to_string();
        let left = VcardCst::parse(&held).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(merged.matches("EMAIL").count(), 2, "got: {merged}");
        assert_eq!(merged.matches("TYPE=work").count(), 1, "got: {merged}");
        assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
    }

    #[test]
    fn an_identity_meets_the_other_case_it_was_written_in() {
        // NOTE: matching normalises, so the two scheme spellings are one
        // address and one instance comes out. Writing is exact, so what lands
        // is the bytes a side wrote: the right side rewrote the scheme's case,
        // a change like any other, and the merge invents no spelling.
        let base = card("IMPP:XMPP:ada@x.test\r\n");
        let left = card("IMPP;TYPE=work:XMPP:ada@x.test\r\n");
        let right = card("IMPP;PREF=1:xmpp:ada@x.test\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(
            merged,
            card("IMPP;TYPE=work;PREF=1:xmpp:ada@x.test\r\n"),
            "got: {merged}",
        );
        assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
    }

    #[test]
    fn a_pid_keeps_a_rename_a_rename() {
        // NOTE: `PID` sits above the natural identity, so a card carrying one
        // reads a changed address as one instance edited rather than as one
        // leaving and another arriving.
        let base = card("EMAIL;PID=1:ada@x.test\r\n");
        let left = card("EMAIL;PID=1;TYPE=work:ada@x.test\r\n");
        let right = card("EMAIL;PID=1:bob@x.test\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);
        let merged = report.merged.to_string();

        assert_eq!(
            merged,
            card("EMAIL;PID=1;TYPE=work:bob@x.test\r\n"),
            "got: {merged}",
        );
        assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
    }

    /// A field only one side touched is taken from that side, whether or not
    /// another field of the same card collided.
    #[test]
    fn an_uncontested_field_keeps_the_change_its_one_side_made() {
        let base = card("FN:A\r\nNOTE:x\r\n");
        let left = card("FN:B\r\nNOTE:x\r\n");
        let right = card("FN:A\r\nNOTE:x\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let left = VcardCst::parse(&left).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &left, &right);

        assert_eq!(report.merged.to_string(), card("FN:B\r\nNOTE:x\r\n"));
        assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
    }

    /// Keeping data beats losing it silently, so the update wins from either
    /// side, which is the one rule the ours-and-theirs convention does not
    /// settle.
    #[test]
    fn an_update_still_beats_a_removal() {
        let base = card("FN:X\r\nNOTE:a\r\n");
        let removed = card("FN:X\r\n");
        let updated = card("FN:X\r\nNOTE:b\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let removed = VcardCst::parse(&removed).unwrap();
        let updated = VcardCst::parse(&updated).unwrap();

        let report = merge(&base, &removed, &updated);
        assert!(report.merged.to_string().contains("NOTE:b\r\n"));

        let report = merge(&base, &updated, &removed);
        assert!(report.merged.to_string().contains("NOTE:b\r\n"));
    }

    /// A list value carrying a semicolon is diffed whole, not item by item.
    ///
    /// A list's items are the value split on its commas, while an item action
    /// splices one leaf of component zero. Past one component the added item
    /// straddles them, escaping its semicolon and writing the tail twice.
    #[test]
    fn a_list_value_with_a_semicolon_is_diffed_whole() {
        let base = card("FN:A\r\nNICKNAME:a,b;x\r\n");
        let right = card("FN:A\r\nNICKNAME:a,b,c;x\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let right = VcardCst::parse(&right).unwrap();

        let report = merge(&base, &base, &right);
        let merged = report.merged.to_string();

        assert!(merged.contains("NICKNAME:a,b,c;x\r\n"), "got: {merged}");
    }
}
