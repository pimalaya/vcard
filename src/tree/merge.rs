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
//! It runs in four steps, one per submodule: every property line becomes an
//! instance, the base's instances are paired with each side's (`matching`),
//! each side is diffed against the base along its pairing (`diff`), and the
//! right side's actions replay onto a clone of the left card (`merger`).
//! `compare` holds the vocabulary all four decide sameness with, and `slot` the
//! granularity two actions collide at.
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
//! ## Conflicts
//!
//! Divergent changes to the same field are conflicts ([`VcardMergeConflict`]):
//! the left side's action wins in the merged card, except when a removal meets
//! an update, where the update wins at every granularity and whichever side it
//! came from (data survives over silent loss).
//!
//! A change both sides made is no conflict at all. Every conflict is reported,
//! so a caller can resolve differently.
//!
//! ## Envelope and version
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

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    param::VcardParam,
    prop::VcardProp,
    tree::{
        codec::mode::VcardEscaper,
        cst::VcardCst,
        merge::{diff::Diff, instance::Instance, matching::Matching, merger::Merger},
    },
    value::VcardValue,
};

mod compare;
mod diff;
mod instance;
mod matching;
mod merger;
mod slot;

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
        let base = Instance::all(self.base);
        let left = Instance::all(self.left);
        let right = Instance::all(self.right);

        let left_matching = Matching::new(&base, &left);
        let right_matching = Matching::new(&base, &right);

        let left_ops = Diff {
            base: &base,
            side: &left,
            matching: &left_matching,
        }
        .run();

        let right_ops = Diff {
            base: &base,
            side: &right,
            matching: &right_matching,
        }
        .run();

        let mut merger = Merger {
            escaper: VcardEscaper::for_version(self.left.version()),
            base_instances: &base,
            left_instances: &left,
            right_instances: &right,
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

    /// The lowercase, oddly-spelled n line proves byte preservation: neither
    /// side touches it, so it must survive verbatim.
    #[test]
    fn merges_disjoint_edits_byte_preservingly() {
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

    /// The update is restored where the left side removed what the right side
    /// updated, and survives symmetrically the other way round.
    #[test]
    fn an_update_wins_over_a_removal() {
        let base = card("FN:X\r\nNOTE:a\r\n");
        let removed = card("FN:X\r\n");
        let updated = card("FN:X\r\nNOTE:b\r\n");

        let base = VcardCst::parse(&base).unwrap();
        let removed = VcardCst::parse(&removed).unwrap();
        let updated = VcardCst::parse(&updated).unwrap();

        let report = merge(&base, &removed, &updated);
        assert!(report.merged.to_string().contains("NOTE:b\r\n"));
        assert_eq!(report.conflicts.len(), 1);

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

    /// The left card swapped the two EMAIL lines, so `PID` matching has to
    /// route the right side's edit onto the `PID=2` instance wherever the left
    /// card moved it.
    #[test]
    fn pid_identity_survives_a_reorder() {
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

    // NOTE: the three whole-parameter edits below are the right side's replay
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

    /// RFC 2426 section 4 writes a repeated parameter name as
    /// `TEL;TYPE=work;TYPE=voice`, so the side rewriting the first parameter
    /// contests nothing the side rewriting the second wrote.
    #[test]
    fn two_parameters_of_one_name_are_two_fields() {
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

    /// The left side replaced Ada with Bob while the right side set Ada's
    /// type. An address is the instance, so Ada's edit never lands on Bob: it
    /// brings Ada's line back, and the collision is reported.
    #[test]
    fn an_edit_is_never_recorded_against_a_replacement() {
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

    /// An identity a same-named sibling repeats is no identity, so both fall
    /// back to their positions and the edit still lands. Which of the two
    /// interchangeable copies carries it is not pinned, so the assertion is on
    /// the content rather than on the order.
    #[test]
    fn one_address_written_twice_tells_neither_instance_apart() {
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

    /// Matching normalises, so the two scheme spellings are one address and
    /// one instance comes out. Writing is exact, so what lands is the bytes a
    /// side wrote: the right side rewrote the scheme's case, a change like any
    /// other, and the merge invents no spelling.
    #[test]
    fn an_identity_meets_the_other_case_it_was_written_in() {
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

    /// `PID` sits above the natural identity, so a card carrying one reads a
    /// changed address as one instance edited rather than as one leaving and
    /// another arriving.
    #[test]
    fn a_pid_keeps_a_rename_a_rename() {
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
