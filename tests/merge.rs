#![cfg(feature = "parser")]
//! # Three-way merge: algebraic laws, completeness, and a differential
//! reference
//!
//! Property-based and differential coverage of
//! [`vcard::tree::merge::merge`](vcard::tree::merge::merge), the
//! byte-preserving three-way merge. Three layers, all driven by the same
//! generator and the same plain-data model of a card, run over both generated
//! cards and the corpus fixtures. The libFuzzer twin lives in
//! fuzz/fuzz_targets/merge.rs.
//!
//! The **laws** are the algebraic identities a three-way merge owes its caller:
//! an untouched side contributes nothing, two identical edits are not a
//! disagreement, neither the collided fields nor their number depends on which
//! side is called left, the merged card reparses to a fixpoint, a line all
//! three copies carry keeps its bytes (bar the line ending a line gains when
//! it stops being last), and re-merging the merged card changes nothing.
//!
//! The **completeness law** is the one that catches silent loss: every change
//! either lands in the merged card or is named in the report's conflicts, and
//! nothing appears that neither side wrote. It is stated field by field over
//! [`Field`], the granularity the merge itself diffs at.
//!
//! The **differential** compares the real merge against [`reference_merge`], a
//! deliberately naive second implementation that models a card as plain field
//! maps, diffs each side against the base with set operations, and reconciles
//! by the rules the merge module documents. It knows nothing about byte
//! preservation and keeps no ordering, so the two are compared on normalised
//! content ([`canon`]) and on conflict keys.
//!
//! `PROPTEST_CASES` raises the case count and `MERGE_CORPUS_ROUNDS` the number
//! of edit pairs each fixture is merged against, so the same tests run as a
//! long campaign. Regression seeds land in tests/merge.proptest-regressions
//! and are meant to be committed.
//!
//! ## What is deliberately excluded
//!
//! Instance matching by value equality and by position (the second and third
//! passes of the merge's matching) is out of the generator's reach by
//! construction: every generated card either has unique property names or
//! carries a distinct `PID` on every instance, so the reference can match by
//! identity without reimplementing the matching. `PID` matching gets its own
//! law in [`pid_matching_survives_a_reorder`], and equality matching stays
//! covered by the unit tests in the merge module itself.
//!
//! List values are generated with distinct items only, because the merge diffs
//! list items as a multiset but replays them as a set; the duplicate-item test
//! near the bottom pins the current behaviour rather than a law.
//!
//! Corpus fixtures drive the laws unconditionally, and the completeness and
//! differential layers only for the fixtures whose property names are all
//! distinct, since only there is instance identity unambiguous without a `PID`.
//!
//! Every defect this suite found has since been repaired, one Cairn change
//! each, and the exclusions they forced are gone: their reproductions now run
//! alongside the laws.

mod common;

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

use proptest::{prelude::*, strategy::ValueTree, test_runner::TestRunner};
use vcard::{
    param::VcardParam,
    tree::{
        cst::VcardCst,
        leaf::VcardLeaf,
        line::VcardLine,
        merge::{VcardMergeAction, VcardMergeReport, merge},
        param::node::VcardParamNode,
        value::node::VcardValueNode,
    },
    value::VcardValueKind,
};

/// How the merge diffs a property's value, read off its decoded value kind.
/// It decides the field granularity every law below is stated at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Shape {
    /// One opaque value: the whole value is a single field.
    Scalar,
    /// Comma-separated items, reconciled as a set.
    List,
    /// Semicolon components, each an independent field.
    Structured,
}

/// The shape the merge will diff a value of this kind at, mirroring
/// `is_component_structured` and the `TextList` arm of `diff_pair`.
fn shape_of(kind: Option<VcardValueKind>) -> Shape {
    match kind {
        Some(VcardValueKind::TextList) => Shape::List,
        Some(
            VcardValueKind::N
            | VcardValueKind::Adr
            | VcardValueKind::Gender
            | VcardValueKind::Org
            | VcardValueKind::ClientPidMap,
        ) => Shape::Structured,
        _ => Shape::Scalar,
    }
}

/// One property instance as plain data: the form the laws and the reference
/// merge reason about, with no bytes, no escaping and no ordering promises.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Prop {
    /// The uppercased wire name, group prefix included.
    name: String,
    /// The value shape, from the decoded value kind.
    shape: Shape,
    /// The parameters in source order, each an uppercased key and its values.
    params: Vec<(String, Vec<String>)>,
    /// The `;`-components, each a list of `,`-separated decoded values.
    comps: Vec<Vec<String>>,
}

impl Prop {
    /// The instance identity the laws and the reference match on: the property
    /// name, qualified by the `PID` parameter when the card carries one.
    fn id(&self) -> String {
        match self.param("PID") {
            Some(values) => format!("{}#{}", self.name, values.join(",")),
            None => self.name.clone(),
        }
    }

    /// The values of the first parameter carrying `key`.
    fn param(&self, key: &str) -> Option<&Vec<String>> {
        self.params.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

/// A field of one property instance: the granularity at which the merge
/// diffs, the laws reconcile, and conflicts are reported.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Field {
    /// The property itself, added or removed.
    Presence,
    /// One `;`-component of a scalar or structured value.
    Comp(usize),
    /// The item set of a list value.
    Items,
    /// One whole parameter, by uppercased key.
    Param(String),
    /// The item set of a list parameter (`TYPE`, `PID`), by uppercased key.
    ParamItems(String),
    /// A whole-value change on a property the merge did not diff component by
    /// component; it subsumes every value field of its instance.
    Whole,
}

/// A field's value: absent (missing, empty or removed) or the values it holds.
type Value = Option<Vec<String>>;

/// The conflict key a merge report and the reference are compared on: an
/// instance id (or a bare property name, for prop-level conflicts) and a field.
type Key = (String, Field);

/// The list parameters the merge diffs item by item.
const LIST_PARAMS: [&str; 2] = ["TYPE", "PID"];

/// The 4.0 property names that may appear at most once, so two divergent
/// additions of one collide. Mirrors the `AtMostOne` / `ExactlyOne` arms of
/// the crate's per-property cardinality, which is not reachable from an
/// integration test.
const AT_MOST_ONE: [&str; 8] = [
    "KIND",
    "N",
    "BDAY",
    "ANNIVERSARY",
    "GENDER",
    "PRODID",
    "REV",
    "UID",
];

/// Whether a property name may appear at most once in a 4.0 card.
fn at_most_one(name: &str) -> bool {
    AT_MOST_ONE.contains(&name)
}

/// Project a parsed card onto the plain model, skipping the `VERSION`
/// indicator exactly as the merge's own instance list does.
fn model_of(cst: &VcardCst<'_>) -> Vec<Prop> {
    let version = cst.version();
    let mut props = Vec::new();

    for line in &cst.props {
        let name = line.name.get().to_ascii_uppercase();
        if name == "VERSION" {
            continue;
        }

        let params = line
            .params
            .iter()
            .map(|node| {
                let key = node.name.get().to_ascii_uppercase();
                let values = node.values.iter().map(|v| unquote(v.get())).collect();
                (key, values)
            })
            .collect();

        let shape = shape_of(line.decode(version).value.kind());

        let comps = match shape {
            // NOTE: the merge diffs a non-structured value whole, on its raw
            // node, so the model carries the whole node as one canonical
            // string rather than the decoded projection, which truncates a
            // value at its first `;` or `,`.
            Shape::Scalar => vec![vec![whole_value(&line.value)]],
            Shape::List => vec![
                line.value
                    .decode_at(0)
                    .iter()
                    .map(|v| v.to_string())
                    .collect(),
            ],
            Shape::Structured => (0..line.value.component_count())
                .map(|i| {
                    line.value
                        .decode_at(i)
                        .iter()
                        .map(|v| v.to_string())
                        .collect()
                })
                .collect(),
        };

        props.push(Prop {
            name,
            shape,
            params,
            comps,
        });
    }

    props
}

/// A non-structured value as one canonical string: its components joined by
/// `;` and their items by `,`, each item re-escaped so a literal separator
/// stays distinguishable from a structural one, and trailing empty components
/// dropped. This is the granularity the merge compares such a value at.
fn whole_value(node: &VcardValueNode<'_>) -> String {
    let escape = |value: &str| {
        value
            .replace('\\', "\\\\")
            .replace(',', "\\,")
            .replace(';', "\\;")
    };

    let mut comps: Vec<String> = (0..node.component_count())
        .map(|i| {
            let values = node.decode_at(i);
            match values.iter().all(|value| value.is_empty()) {
                true => String::new(),
                false => values
                    .iter()
                    .map(|value| escape(value))
                    .collect::<Vec<_>>()
                    .join(","),
            }
        })
        .collect();

    while comps.last().is_some_and(String::is_empty) {
        comps.pop();
    }

    comps.join(";")
}

/// Strip the double quotes a parameter value may be wrapped in, so a quoted
/// and an unquoted spelling of the same value compare equal.
fn unquote(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(raw)
        .to_string()
}

/// Index a card's instances by identity. `None` when an identity repeats,
/// which is where the identity-based layers cannot run.
fn index(props: &[Prop]) -> Option<BTreeMap<String, &Prop>> {
    let mut out = BTreeMap::new();

    for prop in props {
        if out.insert(prop.id(), prop).is_some() {
            return None;
        }
    }

    Some(out)
}

/// A component's value, treating an absent component and an all-empty one
/// alike, as the merge's `component_eq` does.
fn norm_comp(comp: &[String]) -> Value {
    if comp.iter().all(String::is_empty) {
        return None;
    }

    Some(comp.to_vec())
}

/// The sorted, deduplicated items of a set-valued field.
fn sorted_set<'i>(items: impl Iterator<Item = &'i String>) -> Vec<String> {
    let set: BTreeSet<&String> = items.collect();
    set.into_iter().cloned().collect()
}

/// Every field of one instance, with the value it holds.
fn fields_of(prop: &Prop) -> BTreeMap<Field, Value> {
    let mut out = BTreeMap::new();

    match prop.shape {
        Shape::List => {
            out.insert(Field::Items, Some(sorted_set(prop.comps.iter().flatten())));
        }
        // NOTE: a scalar value is one field however many semicolons it holds,
        // since the merge diffs it whole, as its `Slot::Value` does; the model
        // already carries the whole node as one canonical string.
        Shape::Scalar => {
            out.insert(Field::Whole, norm_comp(&prop.comps.concat()));
        }
        Shape::Structured => {
            for (i, comp) in prop.comps.iter().enumerate() {
                out.insert(Field::Comp(i), norm_comp(comp));
            }
        }
    }

    let mut keys: Vec<&String> = Vec::new();
    for (key, _) in &prop.params {
        if !keys.contains(&key) {
            keys.push(key);
        }
    }

    for key in keys {
        let values: Vec<String> = prop
            .params
            .iter()
            .filter(|(k, _)| k == key)
            .flat_map(|(_, v)| v.iter().cloned())
            .collect();

        // NOTE: a list parameter's items carry no order (RFC 6350 section
        // 5.6), and the merge compares them as a set whether or not the base
        // carries the parameter, so the model holds them sorted.
        if LIST_PARAMS.contains(&key.as_str()) {
            out.insert(
                Field::ParamItems(key.clone()),
                Some(sorted_set(values.iter())),
            );
        } else {
            out.insert(Field::Param(key.clone()), Some(values));
        }
    }

    out
}

/// Write a reconciled field back onto an instance.
fn set_field(prop: &mut Prop, field: &Field, value: Value) {
    match field {
        Field::Items | Field::Whole => prop.comps = vec![value.unwrap_or_default()],
        Field::Comp(i) => {
            while prop.comps.len() <= *i {
                prop.comps.push(vec![String::new()]);
            }
            prop.comps[*i] = value.unwrap_or_else(|| vec![String::new()]);
        }
        Field::Param(key) | Field::ParamItems(key) => {
            prop.params.retain(|(k, _)| k != key);
            if let Some(values) = value {
                prop.params.push((key.clone(), values));
            }
        }
        Field::Presence => {}
    }
}

/// Reconcile a set-valued field: both sides' additions and removals all apply,
/// so it never conflicts.
fn set_reconcile(base: &Value, left: &Value, right: &Value) -> Vec<String> {
    let set = |value: &Value| -> BTreeSet<String> {
        value.clone().unwrap_or_default().into_iter().collect()
    };

    let (base, left, right) = (set(base), set(left), set(right));
    let mut out = base.clone();

    for item in left.difference(&base).chain(right.difference(&base)) {
        out.insert(item.clone());
    }
    for item in base.difference(&left).chain(base.difference(&right)) {
        out.remove(item);
    }

    out.into_iter().collect()
}

/// A canonical, order-free rendering of one instance, for comparing merged
/// content without depending on line or parameter order.
fn canon_prop(prop: &Prop) -> String {
    let mut params: Vec<String> = Vec::new();
    let mut seen: Vec<&String> = Vec::new();

    for (key, _) in &prop.params {
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);

        let mut values: Vec<String> = prop
            .params
            .iter()
            .filter(|(k, _)| k == key)
            .flat_map(|(_, v)| v.iter().cloned())
            .collect();

        if LIST_PARAMS.contains(&key.as_str()) {
            values = sorted_set(values.iter());
        }

        params.push(format!("{key}={}", values.join(",")));
    }

    params.sort();

    let value = match prop.shape {
        Shape::List => sorted_set(prop.comps.iter().flatten()).join(","),
        Shape::Scalar | Shape::Structured => {
            let mut comps: Vec<&Vec<String>> = prop.comps.iter().collect();
            while comps.last().is_some_and(|c| c.iter().all(String::is_empty)) {
                comps.pop();
            }
            comps
                .iter()
                .map(|c| c.join(","))
                .collect::<Vec<_>>()
                .join(";")
        }
    };

    format!("{}[{}]={value}", prop.name, params.join(";"))
}

/// A whole card as a sorted list of canonical instances.
fn canon(props: &[Prop]) -> Vec<String> {
    let mut out: Vec<String> = props.iter().map(canon_prop).collect();
    out.sort();
    out
}

/// The conflict keys a merge report names: one per action of each reported
/// pair, so a field either side lost is covered.
fn conflict_keys(report: &VcardMergeReport<'_>, base: &[Prop]) -> BTreeSet<Key> {
    let mut keys = BTreeSet::new();

    for conflict in &report.conflicts {
        let actions = [&conflict.left, &conflict.right];

        // NOTE: a pair where either side acted on the whole property is one
        // prop-level collision, keyed by name; the other action's finer field
        // key would double-count it.
        let prop_level = actions.iter().any(|action| {
            matches!(
                action,
                VcardMergeAction::PropAdded { .. } | VcardMergeAction::PropRemoved { .. },
            )
        });

        for action in actions {
            let key = action_key(action, base);
            if prop_level && key.1 != Field::Presence {
                continue;
            }
            keys.insert(key);
        }
    }

    keys
}

/// The conflict key of one action: the base instance it targets and the field
/// it occupies. A prop-level action is keyed by bare name, since an addition's
/// path indexes the changed card rather than the base.
fn action_key(action: &VcardMergeAction<'_>, base: &[Prop]) -> Key {
    let at = match action {
        VcardMergeAction::PropAdded { at, .. } | VcardMergeAction::PropRemoved { at, .. } => {
            return (at.name.to_ascii_uppercase(), Field::Presence);
        }
        VcardMergeAction::ValueChanged { at, .. }
        | VcardMergeAction::ValueComponentChanged { at, .. }
        | VcardMergeAction::ValueItemAdded { at, .. }
        | VcardMergeAction::ValueItemRemoved { at, .. }
        | VcardMergeAction::ParamAdded { at, .. }
        | VcardMergeAction::ParamRemoved { at, .. }
        | VcardMergeAction::ParamChanged { at, .. }
        | VcardMergeAction::ParamItemAdded { at, .. }
        | VcardMergeAction::ParamItemRemoved { at, .. } => at,
    };

    let name = at.name.to_ascii_uppercase();
    let target = base.iter().filter(|prop| prop.name == name).nth(at.index);
    let id = target.map(Prop::id).unwrap_or_else(|| name.clone());

    let field = match action {
        VcardMergeAction::ValueChanged { .. } => Field::Whole,
        VcardMergeAction::ValueComponentChanged { component, .. } => Field::Comp(*component),
        VcardMergeAction::ValueItemAdded { .. } | VcardMergeAction::ValueItemRemoved { .. } => {
            Field::Items
        }
        VcardMergeAction::ParamAdded { param, .. }
        | VcardMergeAction::ParamRemoved { param, .. } => Field::Param(param_key(param)),
        VcardMergeAction::ParamChanged { new, .. } => Field::Param(param_key(new)),
        VcardMergeAction::ParamItemAdded { param, .. }
        | VcardMergeAction::ParamItemRemoved { param, .. } => {
            Field::Param(param.to_ascii_uppercase())
        }
        VcardMergeAction::PropAdded { .. } | VcardMergeAction::PropRemoved { .. } => {
            Field::Presence
        }
    };

    (id, field)
}

/// The dispatch key of a decoded parameter, mirroring the merge's own
/// `param_key`: the canonical spelling of a known kind, the uppercased name of
/// an unknown one.
fn param_key(param: &VcardParam<'_>) -> String {
    match param {
        VcardParam::Unknown { name, .. } => name.to_ascii_uppercase(),
        param => param
            .kind()
            .map(|kind| kind.to_string())
            .unwrap_or_default(),
    }
}

/// Whether a field is covered by a reported conflict, directly or through a
/// coarser one on the same instance.
fn reported(keys: &BTreeSet<Key>, id: &str, name: &str, field: &Field) -> bool {
    let coarse = match field {
        Field::ParamItems(key) => Field::Param(key.clone()),
        Field::Comp(_) | Field::Items => Field::Whole,
        field => field.clone(),
    };

    keys.contains(&(id.to_string(), field.clone()))
        || keys.contains(&(id.to_string(), coarse))
        || keys.contains(&(name.to_string(), Field::Presence))
}

/// The completeness law: every change either lands or is reported.
///
/// For every field of every instance the merged card either holds what one
/// side made it (and that side changed it), or what the base held (and neither
/// side changed it). A side's change that did not land must be named in the
/// report's conflicts, and nothing may appear that neither side wrote.
///
/// Set-valued fields (list values, list parameters present on all three) are
/// stated differently: they carry both sides' additions and removals, so the
/// law is an equality against [`set_reconcile`] rather than a choice of side.
///
/// Returns the first violation, or `Ok(())`.
fn check_completeness(
    base: &[Prop],
    left: &[Prop],
    right: &[Prop],
    merged: &[Prop],
    keys: &BTreeSet<Key>,
) -> Result<(), String> {
    let (Some(b), Some(l), Some(r), Some(m)) =
        (index(base), index(left), index(right), index(merged))
    else {
        return Ok(());
    };

    let mut ids: Vec<&String> = Vec::new();
    for id in b.keys().chain(l.keys()).chain(r.keys()).chain(m.keys()) {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }

    for id in ids {
        let (bp, lp, rp, mp) = (b.get(id), l.get(id), r.get(id), m.get(id));
        let name = [bp, lp, rp, mp]
            .into_iter()
            .flatten()
            .next()
            .map(|prop| prop.name.clone())
            .expect("an instance in at least one card");
        let presence = reported(keys, id, &name, &Field::Presence);
        let fail = |what: &str| Err(format!("{id}: {what}"));

        match (bp, lp, rp) {
            // The instance is in the base and both sides kept it: reconcile
            // field by field below.
            (Some(bp), Some(lp), Some(rp)) => {
                if mp.is_none() {
                    return fail("dropped although both sides kept it");
                }
                if bp.shape != lp.shape || bp.shape != rp.shape {
                    continue;
                }
                let mp = mp.expect("a merged instance");
                check_fields(id, &name, bp, lp, rp, mp, keys)?;
            }
            // The left side removed it.
            (Some(bp), None, Some(rp)) => {
                let touched = canon_prop(rp) != canon_prop(bp);
                match (touched, mp) {
                    (false, Some(_)) => return fail("resurrected by an untouched side"),
                    (true, None) => return fail("removed although the right side updated it"),
                    (true, Some(mp)) if canon_prop(mp) != canon_prop(rp) => {
                        return fail("restored as neither side wrote it");
                    }
                    (true, _) if !presence => return fail("remove against update went unreported"),
                    _ => {}
                }
            }
            // The right side removed it.
            (Some(bp), Some(lp), None) => {
                let touched = canon_prop(lp) != canon_prop(bp);
                match (touched, mp) {
                    (false, Some(_)) => return fail("resurrected by an untouched side"),
                    (true, None) => return fail("removed although the left side updated it"),
                    (true, Some(mp)) if canon_prop(mp) != canon_prop(lp) => {
                        return fail("kept as neither side wrote it");
                    }
                    (true, _) if !presence => return fail("update against remove went unreported"),
                    _ => {}
                }
            }
            // Both sides removed it.
            (Some(_), None, None) => {
                if mp.is_some() {
                    return fail("resurrected although both sides removed it");
                }
            }
            // One or both sides added it.
            (None, lp, rp) => {
                let added = lp.or(rp).expect("an added instance");
                match mp {
                    None if !presence => return fail("an addition was dropped unreported"),
                    Some(mp) => {
                        let landed = [lp, rp]
                            .into_iter()
                            .flatten()
                            .any(|side| canon_prop(side) == canon_prop(mp));
                        if !landed {
                            return fail("an addition landed as neither side wrote it");
                        }
                        // The right side lost its content to the left one.
                        if let (Some(lp), Some(rp)) = (lp, rp)
                            && canon_prop(lp) != canon_prop(rp)
                            && canon_prop(mp) != canon_prop(rp)
                            && !presence
                        {
                            return fail("a divergent addition was dropped unreported");
                        }
                    }
                    None => {
                        let _ = added;
                    }
                }
            }
        }
    }

    Ok(())
}

/// The field-level half of the completeness law, for an instance both sides
/// kept.
fn check_fields(
    id: &str,
    name: &str,
    bp: &Prop,
    lp: &Prop,
    rp: &Prop,
    mp: &Prop,
    keys: &BTreeSet<Key>,
) -> Result<(), String> {
    let (bf, lf, rf, mf) = (fields_of(bp), fields_of(lp), fields_of(rp), fields_of(mp));

    let mut fields: BTreeSet<Field> = BTreeSet::new();
    fields.extend(bf.keys().cloned());
    fields.extend(lf.keys().cloned());
    fields.extend(rf.keys().cloned());
    fields.extend(mf.keys().cloned());

    for field in fields {
        let get = |map: &BTreeMap<Field, Value>| map.get(&field).cloned().flatten();
        let (bv, lv, rv, mv) = (get(&bf), get(&lf), get(&rf), get(&mf));

        let set_valued = matches!(field, Field::Items)
            || (matches!(field, Field::ParamItems(_))
                && bv.is_some()
                && lv.is_some()
                && rv.is_some());

        let fail = |what: &str| {
            Err(format!(
                "{id} {field:?}: {what} (base {bv:?}, left {lv:?}, right {rv:?}, merged {mv:?})"
            ))
        };

        if set_valued {
            let held = sorted_set(mv.clone().unwrap_or_default().iter());
            if held != set_reconcile(&bv, &lv, &rv) {
                return fail("set reconciliation lost or invented an item");
            }
            continue;
        }

        if lv == bv && rv == bv {
            if mv != bv {
                return fail("changed although neither side touched it");
            }
        } else if lv == bv {
            if mv != rv && !reported(keys, id, name, &field) {
                return fail("the right side's change neither landed nor was reported");
            }
            if mv != rv && mv != bv {
                return fail("holds what neither side wrote");
            }
        } else if rv == bv {
            if mv != lv {
                return fail("the left side's change was overwritten by an untouched side");
            }
        } else if lv == rv {
            if mv != lv {
                return fail("both sides agreed and the agreement did not land");
            }
        } else {
            if mv != lv && mv != rv {
                return fail("holds what neither side wrote");
            }
            if !reported(keys, id, name, &field) {
                return fail("a divergent change went unreported");
            }
        }
    }

    Ok(())
}

/// The outcome of the naive reference merge.
struct Reference {
    /// The merged instances, in no meaningful order.
    props: Vec<Prop>,
    /// The conflict keys the reference reports.
    conflicts: BTreeSet<Key>,
}

/// A deliberately naive three-way merge over the plain model.
///
/// Instances are matched by identity, each side is diffed against the base
/// field by field as plain set and value comparisons, and the two diffs are
/// reconciled by the rules the merge module documents: the left action wins,
/// except that an update beats a removal, set-valued fields carry both sides'
/// additions and removals, and two divergent additions of an at-most-once
/// property collide. It makes no attempt at byte preservation, ordering or
/// clever matching.
fn reference_merge(base: &[Prop], left: &[Prop], right: &[Prop]) -> Reference {
    let (Some(b), Some(l), Some(r)) = (index(base), index(left), index(right)) else {
        return Reference {
            props: Vec::new(),
            conflicts: BTreeSet::new(),
        };
    };

    let mut props = Vec::new();
    let mut conflicts = BTreeSet::new();

    for bp in base {
        let id = bp.id();

        match (l.get(&id), r.get(&id)) {
            (Some(lp), Some(rp)) => {
                props.push(reconcile(&id, bp, lp, rp, &mut conflicts));
            }
            (None, Some(rp)) => {
                if canon_prop(rp) != canon_prop(bp) {
                    conflicts.insert((bp.name.clone(), Field::Presence));
                    props.push((*rp).clone());
                }
            }
            (Some(lp), None) => {
                if canon_prop(lp) != canon_prop(bp) {
                    conflicts.insert((bp.name.clone(), Field::Presence));
                    props.push((*lp).clone());
                }
            }
            (None, None) => {}
        }
    }

    let added = |side: &[Prop]| -> Vec<Prop> {
        side.iter()
            .filter(|prop| !b.contains_key(&prop.id()))
            .cloned()
            .collect()
    };

    let (left_added, right_added) = (added(left), added(right));

    props.extend(left_added.iter().cloned());

    for rp in &right_added {
        if left_added.iter().any(|lp| canon_prop(lp) == canon_prop(rp)) {
            continue;
        }
        if at_most_one(&rp.name) && left_added.iter().any(|lp| lp.name == rp.name) {
            conflicts.insert((rp.name.clone(), Field::Presence));
            continue;
        }
        props.push(rp.clone());
    }

    Reference { props, conflicts }
}

/// Reconcile one instance both sides kept, field by field.
fn reconcile(id: &str, bp: &Prop, lp: &Prop, rp: &Prop, conflicts: &mut BTreeSet<Key>) -> Prop {
    let mut out = bp.clone();
    let (bf, lf, rf) = (fields_of(bp), fields_of(lp), fields_of(rp));

    let mut fields: BTreeSet<Field> = BTreeSet::new();
    fields.extend(bf.keys().cloned());
    fields.extend(lf.keys().cloned());
    fields.extend(rf.keys().cloned());

    for field in fields {
        let get = |map: &BTreeMap<Field, Value>| map.get(&field).cloned().flatten();
        let (bv, lv, rv) = (get(&bf), get(&lf), get(&rf));

        let set_valued = matches!(field, Field::Items)
            || (matches!(field, Field::ParamItems(_))
                && bv.is_some()
                && lv.is_some()
                && rv.is_some());

        let merged = if set_valued {
            Some(set_reconcile(&bv, &lv, &rv))
        } else if lv == bv {
            rv
        } else if rv == bv || rv == lv {
            lv
        } else {
            let key = match &field {
                Field::ParamItems(key) => Field::Param(key.clone()),
                field => field.clone(),
            };
            conflicts.insert((id.to_string(), key));

            // NOTE: an update beats a removal, so a side that dropped the
            // parameter loses to the side that rewrote it. Clearing a value
            // component is an update rather than a removal, so the exception
            // is a parameter's alone.
            let structural = matches!(field, Field::Param(_) | Field::ParamItems(_));

            match (structural, lv.is_none(), rv.is_none()) {
                (true, true, false) => rv,
                _ => lv,
            }
        };

        set_field(&mut out, &field, merged);
    }

    out
}

/// One synthetic edit a side applies to its copy of the base card.
#[derive(Clone, Copy, Debug)]
enum Edit {
    /// Set one component of a scalar or structured value, or add one item to a
    /// list value.
    Value {
        /// The target instance, taken modulo the editable count.
        prop: usize,
        /// The target component, taken modulo the component count.
        comp: usize,
        /// The word to write, from [`WORDS`].
        word: usize,
    },
    /// Clear one component, or drop one item of a list value.
    Clear {
        /// The target instance.
        prop: usize,
        /// The target component or item.
        comp: usize,
    },
    /// Set a whole parameter, or add one item to `TYPE`.
    Param {
        /// The target instance.
        prop: usize,
        /// The parameter, from [`PARAM_KEYS`].
        key: usize,
        /// The word to write.
        word: usize,
    },
    /// Drop a whole parameter.
    DropParam {
        /// The target instance.
        prop: usize,
        /// The parameter to drop.
        key: usize,
    },
    /// Drop a whole property.
    DropProp {
        /// The target instance.
        prop: usize,
    },
    /// Add a property from [`ADD_POOL`].
    AddProp {
        /// The pool entry.
        seed: usize,
    },
}

impl Edit {
    /// The instance an edit targets, or `None` for an addition.
    fn prop(&self) -> Option<usize> {
        match self {
            Self::Value { prop, .. }
            | Self::Clear { prop, .. }
            | Self::Param { prop, .. }
            | Self::DropParam { prop, .. }
            | Self::DropProp { prop } => Some(*prop),
            Self::AddProp { .. } => None,
        }
    }
}

/// The small word pool every generated value is drawn from, so the two sides
/// frequently write the same word (an agreement) and frequently different ones
/// (a collision).
const WORDS: [&str; 3] = ["alpha", "beta", "gamma"];

/// The parameters the generator edits. `PID` is excluded: it carries instance
/// identity.
const PARAM_KEYS: [&str; 4] = ["TYPE", "PREF", "LANGUAGE", "X-TAG"];

/// The properties an addition may draw from. The two `UID` entries share an
/// at-most-once name, so two divergent additions of it collide; every other
/// name appears once, so two sides adding the same entry add the same
/// property.
const ADD_POOL: [(&str, Shape, &[&str]); 6] = [
    ("UID", Shape::Scalar, &["urn:uuid:a"]),
    ("UID", Shape::Scalar, &["urn:uuid:b"]),
    ("TITLE", Shape::Scalar, &["boss"]),
    ("ROLE", Shape::Scalar, &["dev"]),
    ("CATEGORIES", Shape::List, &["x", "y"]),
    ("ORG", Shape::Structured, &["Acme", "Widgets"]),
];

/// Lines no edit ever targets, carrying a redundant escape, a fold and a
/// quoted parameter, so the byte-preservation law has something to bite on.
const NOISE: &str = concat!(
    "X-NOISE:a\\;b\\,c\\\\d\r\n",
    "X-FOLD:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\n",
    " bbbbbbbbbbbbbbbbbbbb\r\n",
    "x-lower;Q=\"a;b\":kept\r\n",
);

/// Build one instance for the base card or the addition pool.
fn prop(name: &str, shape: Shape, params: &[(&str, &[&str])], comps: Vec<Vec<String>>) -> Prop {
    Prop {
        name: name.to_string(),
        shape,
        params: params
            .iter()
            .map(|(key, values)| {
                (
                    key.to_string(),
                    values.iter().map(|v| v.to_string()).collect(),
                )
            })
            .collect(),
        comps,
    }
}

/// The base card the two sides diverge from: one instance of every shape, with
/// and without parameters. In `pid` mode it gains a second `TEL` and an
/// `EMAIL` and every instance carries a distinct `PID`, so property names
/// repeat while instance identity stays unambiguous.
fn base_model(pid: bool, seed: [usize; 4]) -> Vec<Prop> {
    let word = |i: usize| WORDS[i % WORDS.len()].to_string();
    let one = |text: &str| vec![text.to_string()];

    let mut props = vec![
        prop(
            "FN",
            Shape::Scalar,
            &[("TYPE", &["work"]), ("LANGUAGE", &["en"])],
            vec![one(&word(seed[0]))],
        ),
        prop(
            "N",
            Shape::Structured,
            &[("LANGUAGE", &["en"])],
            vec![one("Doe"), one("John"), one(""), one(""), one("")],
        ),
        prop(
            "NICKNAME",
            Shape::List,
            &[("PREF", &["1"])],
            vec![vec!["nick".to_string(), word(seed[1])]],
        ),
        prop(
            "TEL",
            Shape::Scalar,
            &[("TYPE", &["work"]), ("PREF", &["1"])],
            vec![one(&format!("+155501{}", seed[2]))],
        ),
        prop(
            "NOTE",
            Shape::Scalar,
            &[("LANGUAGE", &["en"])],
            vec![one(&word(seed[3]))],
        ),
    ];

    if pid {
        props.push(prop(
            "TEL",
            Shape::Scalar,
            &[("TYPE", &["home"])],
            vec![one("+15550999")],
        ));
        props.push(prop("EMAIL", Shape::Scalar, &[], vec![one("a@b.test")]));

        for (i, prop) in props.iter_mut().enumerate() {
            prop.params
                .push(("PID".to_string(), vec![(i + 1).to_string()]));
        }
    }

    props
}

/// The instance an addition adds, identical for both sides drawing the same
/// pool entry so the both-added path is reachable.
fn added_prop(seed: usize, pid: bool) -> Prop {
    let (name, shape, values) = ADD_POOL[seed % ADD_POOL.len()];

    let comps = match shape {
        Shape::List => vec![values.iter().map(|v| v.to_string()).collect()],
        Shape::Scalar | Shape::Structured => values.iter().map(|v| vec![v.to_string()]).collect(),
    };

    let mut prop = prop(name, shape, &[], comps);

    if pid {
        prop.params
            .push(("PID".to_string(), vec![(90 + seed).to_string()]));
    }

    prop
}

/// Apply one side's edits to a copy of the base model. Field edits run first
/// on stable indices, then removals, then additions, so both sides address the
/// same instances.
fn apply_edits(base: &[Prop], edits: &[Edit], pid: bool) -> Vec<Prop> {
    let editable = base.len();
    let mut props = base.to_vec();

    for edit in edits {
        let Some(target) = edit.prop() else { continue };
        let target = target % editable;

        match *edit {
            Edit::Value { comp, word, .. } => {
                let text = WORDS[word % WORDS.len()].to_string();
                let prop = &mut props[target];
                match prop.shape {
                    Shape::List => {
                        if !prop.comps[0].contains(&text) {
                            prop.comps[0].push(text);
                        }
                    }
                    Shape::Scalar => prop.comps[0] = vec![text],
                    Shape::Structured => {
                        let i = comp % prop.comps.len();
                        prop.comps[i] = vec![text];
                    }
                }
            }
            Edit::Clear { comp, .. } => {
                let prop = &mut props[target];
                match prop.shape {
                    // NOTE: a list is never emptied: an empty value decodes to
                    // one empty item, which the merge sees as an item rather
                    // than as an absence.
                    Shape::List => {
                        if prop.comps[0].len() > 1 {
                            let i = comp % prop.comps[0].len();
                            prop.comps[0].remove(i);
                        }
                    }
                    Shape::Scalar => prop.comps[0] = vec![String::new()],
                    Shape::Structured => {
                        let i = comp % prop.comps.len();
                        prop.comps[i] = vec![String::new()];
                    }
                }
            }
            Edit::Param { key, word, .. } => {
                let key = PARAM_KEYS[key % PARAM_KEYS.len()].to_string();
                let text = WORDS[word % WORDS.len()].to_string();
                let prop = &mut props[target];

                match prop.params.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, values)) if key == "TYPE" => {
                        if !values.contains(&text) {
                            values.push(text);
                        }
                    }
                    Some((_, values)) => *values = vec![text],
                    None => prop.params.push((key, vec![text])),
                }
            }
            Edit::DropParam { key, .. } => {
                let key = PARAM_KEYS[key % PARAM_KEYS.len()];
                props[target].params.retain(|(k, _)| k != key);
            }
            Edit::DropProp { .. } | Edit::AddProp { .. } => {}
        }
    }

    let mut dropped: Vec<usize> = edits
        .iter()
        .filter_map(|edit| match edit {
            Edit::DropProp { prop } => Some(prop % editable),
            _ => None,
        })
        .collect();
    dropped.sort_unstable();
    dropped.dedup();

    for &i in dropped.iter().rev() {
        props.remove(i);
    }

    for edit in edits {
        let Edit::AddProp { seed } = edit else {
            continue;
        };
        let added = added_prop(*seed, pid);
        // NOTE: one instance per name per side, so an identity stays unique
        // inside each card and the merged card can be indexed by it.
        if props.iter().all(|prop| prop.name != added.name) {
            props.push(added);
        }
    }

    props
}

/// Serialize a model into a 4.0 card. Values are drawn from an alphabet with
/// no separator or backslash in it, so no escaping is needed and a parse of
/// the output projects back onto the same model.
fn render(props: &[Prop]) -> String {
    let mut out = String::from("BEGIN:VCARD\r\nVERSION:4.0\r\n");

    for prop in props {
        out.push_str(&prop.name);

        for (key, values) in &prop.params {
            out.push(';');
            out.push_str(key);
            if !values.is_empty() {
                out.push('=');
                out.push_str(&values.join(","));
            }
        }

        out.push(':');
        let comps: Vec<String> = prop.comps.iter().map(|c| c.join(",")).collect();
        out.push_str(&comps.join(";"));
        out.push_str("\r\n");
    }

    out.push_str(NOISE);
    out.push_str("END:VCARD\r\n");
    out
}

/// One generated case: the base card seed, whether instances carry a `PID`,
/// and each side's edits.
#[derive(Clone, Debug)]
struct Case {
    /// Whether every instance carries a distinct `PID`, which also repeats
    /// property names in the base card.
    pid: bool,
    /// The words the base card's values are seeded from.
    seed: [usize; 4],
    /// The left side's edits.
    left: Vec<Edit>,
    /// The right side's edits.
    right: Vec<Edit>,
}

impl Case {
    /// The three cards of the case, as raw text.
    fn cards(&self) -> (String, String, String) {
        let base = base_model(self.pid, self.seed);
        let left = apply_edits(&base, &self.left, self.pid);
        let right = apply_edits(&base, &self.right, self.pid);

        (render(&base), render(&left), render(&right))
    }
}

/// The instance an edit targets, biased onto the first few so the two sides
/// frequently pick the same one.
fn arb_target() -> impl Strategy<Value = usize> {
    prop_oneof![5 => 0usize..2, 1 => 2usize..7]
}

/// The component or item an edit targets, biased the same way.
fn arb_comp() -> impl Strategy<Value = usize> {
    prop_oneof![4 => 0usize..2, 1 => 2usize..5]
}

/// One edit, biased towards the field edits that actually collide.
fn arb_edit() -> impl Strategy<Value = Edit> {
    prop_oneof![
        8 => (arb_target(), arb_comp(), 0usize..3)
            .prop_map(|(prop, comp, word)| Edit::Value { prop, comp, word }),
        3 => (arb_target(), arb_comp()).prop_map(|(prop, comp)| Edit::Clear { prop, comp }),
        5 => (arb_target(), 0usize..4, 0usize..3)
            .prop_map(|(prop, key, word)| Edit::Param { prop, key, word }),
        3 => (arb_target(), 0usize..4).prop_map(|(prop, key)| Edit::DropParam { prop, key }),
        2 => arb_target().prop_map(|prop| Edit::DropProp { prop }),
        3 => (0usize..6).prop_map(|seed| Edit::AddProp { seed }),
    ]
}

/// A base card plus two derived edit scripts. Both scripts draw from the same
/// small target space, so the two sides frequently touch the same field.
fn arb_case() -> impl Strategy<Value = Case> {
    (
        any::<bool>(),
        proptest::array::uniform4(0usize..3),
        proptest::collection::vec(arb_edit(), 2..5),
        proptest::collection::vec(arb_edit(), 2..5),
    )
        .prop_map(|(pid, seed, left, right)| Case {
            pid,
            seed,
            left,
            right,
        })
}

/// The logical lines of a card, as the exact bytes each one serializes to.
fn lines(cst: &VcardCst<'_>) -> Vec<String> {
    cst.props.iter().map(VcardLine::to_string).collect()
}

/// How many times each item occurs.
fn counts(items: &[String]) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for item in items {
        *out.entry(item.clone()).or_insert(0) += 1;
    }
    out
}

/// Parse the three cards, merge them, and run every law that does not need
/// instance identity. Returns the merged card's text and the conflict keys.
fn check_laws(base: &str, left: &str, right: &str) -> Result<(String, usize), String> {
    let base_cst = VcardCst::parse(base).map_err(|e| format!("parse base: {e}"))?;
    let left_cst = VcardCst::parse(left).map_err(|e| format!("parse left: {e}"))?;
    let right_cst = VcardCst::parse(right).map_err(|e| format!("parse right: {e}"))?;

    let report = merge(&base_cst, &left_cst, &right_cst);
    let merged = report.merged.to_string();

    // The merged card always parses again, unless the right side removed
    // every line there was: nothing is not a card.
    if merged.is_empty() {
        return Ok((merged, report.conflicts.len()));
    }

    let reparsed = VcardCst::parse(&merged).map_err(|e| format!("reparse merged: {e}"))?;
    if reparsed.to_string() != merged {
        return Err("the merged card is not a serialization fixpoint".to_string());
    }

    // A line neither side touched keeps its bytes, folds included.
    let (base_lines, left_lines, right_lines, merged_lines) = (
        lines(&base_cst),
        lines(&left_cst),
        lines(&right_cst),
        lines(&report.merged),
    );
    let (bc, lc, rc, mc) = (
        counts(&base_lines),
        counts(&left_lines),
        counts(&right_lines),
        counts(&merged_lines),
    );

    for (line, &count) in &bc {
        let kept = count
            .min(lc.get(line).copied().unwrap_or(0))
            .min(rc.get(line).copied().unwrap_or(0));

        let mut held = mc.get(line).copied().unwrap_or(0);

        // NOTE: a line a source file left unterminated gains the default
        // ending when it stops being last, which is framing rather than
        // content, and is the one gain the law allows.
        if !line.ends_with('\n') {
            held += mc.get(&format!("{line}\r\n")).copied().unwrap_or(0);
        }

        if held < kept {
            return Err(format!("an untouched line lost its bytes: {line:?}"));
        }
    }

    // The identity laws, on the cards as they are.
    let twin = merge(&base_cst, &left_cst, &left_cst);
    if twin.merged.to_bytes() != left_cst.to_bytes() || !twin.conflicts.is_empty() {
        return Err(format!(
            "two identical edits disagreed: {:?}",
            twin.conflicts,
        ));
    }

    let untouched = merge(&base_cst, &left_cst, &base_cst);
    if untouched.merged.to_bytes() != left_cst.to_bytes() || !untouched.conflicts.is_empty() {
        return Err(format!(
            "an untouched right side contributed something: {:?}",
            untouched.conflicts,
        ));
    }

    // Every reported path addresses a real base instance. Only an addition's
    // path indexes the changed card, so it is the one exception.
    let base_model = model_of(&base_cst);

    for action in report.left.iter().chain(&report.right) {
        if matches!(action, VcardMergeAction::PropAdded { .. }) {
            continue;
        }

        let (name, index) = action_path(action);
        let resolved = base_model
            .iter()
            .filter(|prop| prop.name == name.to_ascii_uppercase())
            .nth(index);

        if resolved.is_none() {
            return Err(format!("an action addresses no base instance: {action:?}"));
        }
    }

    // Conflict symmetry: swapping the sides reports the same collided fields,
    // and as many pairs. The merged bytes legitimately differ, since the left
    // action wins.
    let swapped = merge(&base_cst, &right_cst, &left_cst);
    let (forward, backward) = (
        conflict_keys(&report, &base_model),
        conflict_keys(&swapped, &base_model),
    );

    if forward != backward {
        return Err(format!(
            "conflict fields are not symmetric:\nleft-first  {forward:#?}\nright-first {backward:#?}",
        ));
    }

    if report.conflicts.len() != swapped.conflicts.len() {
        return Err(format!(
            "conflict counts are not symmetric:\nleft-first  {:#?}\nright-first {:#?}",
            report.conflicts, swapped.conflicts,
        ));
    }

    // Idempotence: replaying the right side onto the merged card changes
    // nothing.
    let again = merge(&base_cst, &reparsed, &right_cst);
    if canon(&model_of(&again.merged)) != canon(&model_of(&reparsed)) {
        return Err(format!(
            "merging the merged card again thrashed:\n{}\n{}",
            merged, again.merged,
        ));
    }

    Ok((merged, report.conflicts.len()))
}

/// Run the identity laws, the completeness law and the differential on one
/// triple. Returns whether the merge reported a conflict.
fn check_case(base: &str, left: &str, right: &str) -> Result<bool, String> {
    let base_cst = VcardCst::parse(base).map_err(|e| format!("parse base: {e}"))?;
    let left_cst = VcardCst::parse(left).map_err(|e| format!("parse left: {e}"))?;
    let right_cst = VcardCst::parse(right).map_err(|e| format!("parse right: {e}"))?;

    let report = merge(&base_cst, &left_cst, &right_cst);

    let base_model = model_of(&base_cst);
    let left_model = model_of(&left_cst);
    let right_model = model_of(&right_cst);
    let merged_model = model_of(&report.merged);

    // The identity-keyed layers cannot run on an ambiguous card, and both
    // would then pass or fail for the wrong reason. The callers only feed them
    // cards whose instances are identifiable, so an ambiguity here is a defect
    // in the harness rather than a case to skip quietly.
    let identifiable = [&base_model, &left_model, &right_model, &merged_model]
        .into_iter()
        .all(|model| index(model).is_some());

    if !identifiable {
        return Err("instance identity repeats: the layers cannot run".to_string());
    }

    let keys = conflict_keys(&report, &base_model);

    check_completeness(&base_model, &left_model, &right_model, &merged_model, &keys)
        .map_err(|e| format!("completeness: {e}"))?;

    let reference = reference_merge(&base_model, &left_model, &right_model);

    if canon(&reference.props) != canon(&merged_model) {
        return Err(format!(
            "content differs from the reference:\nreal      {:#?}\nreference {:#?}",
            canon(&merged_model),
            canon(&reference.props),
        ));
    }

    if reference.conflicts != keys {
        return Err(format!(
            "conflicts differ from the reference:\nreal      {keys:#?}\nreference {:#?}",
            reference.conflicts,
        ));
    }

    Ok(!report.conflicts.is_empty())
}

/// The proptest configuration every law below runs with. `PROPTEST_CASES`
/// overrides the count, so the same suite doubles as a long fuzzing run.
/// Regression seeds land in tests/merge.proptest-regressions and are meant
/// to be committed.
fn config(cases: u32) -> ProptestConfig {
    let default = ProptestConfig::default();

    ProptestConfig {
        cases: match std::env::var("PROPTEST_CASES") {
            Ok(_) => default.cases,
            Err(_) => cases,
        },
        ..default
    }
}

#[test]
fn the_generator_produces_collisions_often_enough() {
    // NOTE: the generator quality is the test quality: merging two unrelated
    // random cards exercises no reconciliation at all. This measures the
    // fraction of generated triples where the two sides actually collide, and
    // fails if the generator drifts into producing disjoint edits.
    let mut runner = TestRunner::deterministic();
    let strategy = arb_case();
    let (mut total, mut conflicting) = (0usize, 0usize);

    for _ in 0..2000 {
        let case = strategy
            .new_tree(&mut runner)
            .expect("a generated case")
            .current();
        let (base, left, right) = case.cards();

        let base_cst = VcardCst::parse(&base).expect("a generated base card");
        let left_cst = VcardCst::parse(&left).expect("a generated left card");
        let right_cst = VcardCst::parse(&right).expect("a generated right card");

        let report = merge(&base_cst, &left_cst, &right_cst);

        total += 1;
        if !report.conflicts.is_empty() {
            conflicting += 1;
        }
    }

    let rate = conflicting as f64 / total as f64;
    println!("collision rate: {conflicting}/{total} = {rate:.3}");

    assert!(
        rate > 0.25,
        "only {conflicting}/{total} generated triples collide; the generator is not \
         exercising reconciliation",
    );
}

proptest! {
    #![proptest_config(config(512))]

    /// Two identical edits are not a disagreement: the merged card is the
    /// side, byte for byte, and nothing is reported.
    #[test]
    fn merging_a_side_with_itself_yields_that_side(case in arb_case()) {
        let (base, side, _) = case.cards();

        let base_cst = VcardCst::parse(&base).unwrap();
        let side_cst = VcardCst::parse(&side).unwrap();

        let report = merge(&base_cst, &side_cst, &side_cst);

        prop_assert_eq!(report.merged.to_bytes(), side_cst.to_bytes());
        prop_assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
    }

    /// An untouched left side contributes nothing: the merged card is the
    /// right one.
    #[test]
    fn an_untouched_left_side_yields_the_right_one(case in arb_case()) {
        let (base, _, right) = case.cards();

        let base_cst = VcardCst::parse(&base).unwrap();
        let right_cst = VcardCst::parse(&right).unwrap();

        let report = merge(&base_cst, &base_cst, &right_cst);

        prop_assert_eq!(
            canon(&model_of(&report.merged)),
            canon(&model_of(&right_cst)),
        );
        prop_assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
    }

    /// An untouched right side contributes nothing: the merged card is the
    /// left one, byte for byte, since nothing replays onto it.
    #[test]
    fn an_untouched_right_side_yields_the_left_one(case in arb_case()) {
        let (base, left, _) = case.cards();

        let base_cst = VcardCst::parse(&base).unwrap();
        let left_cst = VcardCst::parse(&left).unwrap();

        let report = merge(&base_cst, &left_cst, &base_cst);

        prop_assert_eq!(report.merged.to_bytes(), left_cst.to_bytes());
        prop_assert!(report.conflicts.is_empty(), "{:?}", report.conflicts);
    }

    /// Merging a card with itself against itself is the identity.
    #[test]
    fn merging_a_base_with_itself_is_the_identity(case in arb_case()) {
        let (base, _, _) = case.cards();
        let base_cst = VcardCst::parse(&base).unwrap();

        let report = merge(&base_cst, &base_cst, &base_cst);

        prop_assert_eq!(report.merged.to_bytes(), base_cst.to_bytes());
        prop_assert!(report.left.is_empty());
        prop_assert!(report.right.is_empty());
        prop_assert!(report.conflicts.is_empty());
    }

    /// The merged card reparses, an untouched line keeps its bytes, the
    /// conflict set does not depend on which side is called left, and merging
    /// the merged card again changes nothing.
    #[test]
    fn the_merge_laws_hold(case in arb_case()) {
        let (base, left, right) = case.cards();
        check_laws(&base, &left, &right).map_err(TestCaseError::fail)?;
    }

    /// Every change either lands or is reported, and the merged content and
    /// conflict set agree with the naive reference.
    #[test]
    fn every_change_lands_or_is_reported(case in arb_case()) {
        let (base, left, right) = case.cards();
        check_case(&base, &left, &right).map_err(TestCaseError::fail)?;
    }
}

/// Apply one edit to a parsed card through the crate's own byte-preserving
/// edit layer, the way a real consumer would produce a divergent copy.
/// `editable` lists the line indices an edit may target.
fn apply_cst_edit(cst: &mut VcardCst<'static>, edit: &Edit, editable: &[usize]) {
    if editable.is_empty() {
        return;
    }

    let target = edit
        .prop()
        .map(|prop| editable[prop % editable.len()])
        .unwrap_or_default();

    match *edit {
        Edit::Value { comp, word, .. } => {
            let text = WORDS[word % WORDS.len()];
            let line = &mut cst.props[target];
            let count = line.value.component_count().max(1);
            line.value.set_at(comp % count, &[text]);
        }
        Edit::Clear { comp, .. } => {
            let line = &mut cst.props[target];
            let count = line.value.component_count().max(1);
            let i = comp % count;
            if line.value.value_count(i) > 1 {
                line.value.remove_value_at(i, 0);
            } else {
                line.value.set_at(i, &[""]);
            }
        }
        Edit::Param { key, word, .. } => {
            let key = PARAM_KEYS[key % PARAM_KEYS.len()];
            let text = WORDS[word % WORDS.len()];
            let line = &mut cst.props[target];

            match line
                .params
                .iter_mut()
                .find(|node| node.name.get().eq_ignore_ascii_case(key))
            {
                Some(node) => node.values = vec![VcardLeaf::from(text)],
                None => line.params.push(VcardParamNode {
                    name: VcardLeaf::from(key),
                    values: vec![VcardLeaf::from(text)],
                }),
            }
        }
        Edit::DropParam { key, .. } => {
            let key = PARAM_KEYS[key % PARAM_KEYS.len()];
            cst.props[target]
                .params
                .retain(|node| !node.name.get().eq_ignore_ascii_case(key));
        }
        Edit::DropProp { .. } => {
            cst.props.remove(target);
        }
        Edit::AddProp { seed } => {
            // NOTE: the name carries the word, so two sides adding different
            // words add two differently-named properties rather than two
            // instances of one name, which would make the added instances
            // indistinguishable by identity.
            let names = [
                "X-MERGE-ALPHA",
                "X-MERGE-BETA",
                "X-MERGE-GAMMA",
                "X-MERGE-DELTA",
                "X-MERGE-EPSILON",
                "X-MERGE-ZETA",
            ];
            let name = names[seed % names.len()];

            if cst
                .props
                .iter()
                .all(|line| !line.name.get().eq_ignore_ascii_case(name))
            {
                cst.props.push(VcardLine::text(name, "added"));
            }
        }
    }
}

/// The line indices of a fixture an edit may target: a real property whose
/// name occurs exactly once, so instance identity is unambiguous without a
/// `PID`, and which is not part of an embedded card.
fn editable_lines(cst: &VcardCst<'_>) -> Vec<usize> {
    let names: Vec<String> = cst
        .props
        .iter()
        .map(|line| line.name.get().to_ascii_uppercase())
        .collect();

    names
        .iter()
        .enumerate()
        .filter(|(_, name)| {
            !matches!(name.as_str(), "VERSION" | "BEGIN" | "END" | "AGENT")
                && names.iter().filter(|other| other == name).count() == 1
        })
        .map(|(i, _)| i)
        .collect()
}

/// Derive a side from a fixture by replaying edits through the crate's edit
/// layer.
fn corpus_side(base: &VcardCst<'static>, edits: &[Edit]) -> String {
    let mut cst = base.clone();
    let editable = editable_lines(&cst);

    for edit in edits {
        if matches!(edit, Edit::DropProp { .. }) {
            continue;
        }
        apply_cst_edit(&mut cst, edit, &editable);
    }

    let mut dropped: Vec<usize> = edits
        .iter()
        .filter_map(|edit| match edit {
            Edit::DropProp { prop } if !editable.is_empty() => {
                Some(editable[prop % editable.len()])
            }
            _ => None,
        })
        .collect();
    dropped.sort_unstable();
    dropped.dedup();

    for &line in dropped.iter().rev() {
        cst.props.remove(line);
    }

    cst.to_string()
}

/// How many generated edit pairs each corpus fixture is merged against.
/// `MERGE_CORPUS_ROUNDS` raises it, so the sweep doubles as a long fuzzing run.
fn corpus_rounds() -> usize {
    std::env::var("MERGE_CORPUS_ROUNDS")
        .ok()
        .and_then(|rounds| rounds.parse().ok())
        .unwrap_or(24)
}

/// Every fixture of every corpus, as owned parsed cards.
fn corpora() -> Vec<(String, VcardCst<'static>)> {
    let corpora = [
        ("calcard", 92),
        ("emersion", 5),
        ("ez-vcard", 17),
        ("jeroen", 6),
        ("mixerp", 2),
        ("nuovo", 3),
        ("rfc", 17),
        ("sabre", 2),
        ("vcardigan", 2),
    ];

    let out = RefCell::new(Vec::new());

    for (corpus, expected) in corpora {
        common::each_fixture(corpus, expected, |name, input| {
            for (i, card) in VcardCst::parse_many(input).enumerate() {
                let Ok(card) = card else { continue };
                out.borrow_mut()
                    .push((format!("{corpus}/{name}#{i}"), card.into_static()));
            }
        });
    }

    out.into_inner()
}

#[test]
fn the_laws_hold_over_the_whole_corpus() {
    // NOTE: cargo-fuzz needs a nightly toolchain, which the devshell does not
    // carry, so the corpus is swept with a deterministic, seeded proptest run
    // instead: every fixture is merged against many generated edit pairs.
    let mut runner = TestRunner::deterministic();
    let strategy = (
        proptest::collection::vec(arb_edit(), 2..5),
        proptest::collection::vec(arb_edit(), 2..5),
    );

    let cards = corpora();
    assert!(cards.len() > 140, "the corpus shrank to {}", cards.len());

    let (mut total, mut conflicting) = (0usize, 0usize);
    let mut failures: Vec<String> = Vec::new();

    for (name, base) in &cards {
        for _ in 0..corpus_rounds() {
            let (left, right) = strategy
                .new_tree(&mut runner)
                .expect("a generated edit pair")
                .current();

            let base_text = base.to_string();
            let left_text = corpus_side(base, &left);
            let right_text = corpus_side(base, &right);

            total += 1;

            match check_laws(&base_text, &left_text, &right_text) {
                Ok((_, conflicts)) => {
                    if conflicts > 0 {
                        conflicting += 1;
                    }
                }
                Err(error) => {
                    if failures.len() < 5 {
                        failures.push(format!(
                            "{name} {left:?} / {right:?}: {error}\n--- base\n{base_text}\
                             --- left\n{left_text}--- right\n{right_text}",
                        ));
                    }
                }
            }
        }
    }

    println!(
        "corpus: {total} merges, {conflicting} with conflicts, {} failing",
        failures.len(),
    );

    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn the_reference_agrees_over_the_identifiable_corpus() {
    // NOTE: the completeness law and the differential need unambiguous
    // instance identity, which a fixture only has when no property name
    // repeats; the rest of the corpus is covered by the laws above.
    let mut runner = TestRunner::deterministic();
    let strategy = (
        proptest::collection::vec(arb_edit(), 2..5),
        proptest::collection::vec(arb_edit(), 2..5),
    );

    let (mut cards, mut total) = (0usize, 0usize);
    let mut failures: Vec<String> = Vec::new();

    for (name, base) in &corpora() {
        if index(&model_of(base)).is_none() {
            continue;
        }
        cards += 1;

        for _ in 0..corpus_rounds() {
            let (left, right) = strategy
                .new_tree(&mut runner)
                .expect("a generated edit pair")
                .current();

            let base_text = base.to_string();
            let left_text = corpus_side(base, &left);
            let right_text = corpus_side(base, &right);

            total += 1;

            if let Err(error) = check_case(&base_text, &left_text, &right_text)
                && failures.len() < 5
            {
                failures.push(format!(
                    "{name} {left:?} / {right:?}: {error}\n--- base\n{base_text}\
                     --- left\n{left_text}--- right\n{right_text}",
                ));
            }
        }
    }

    println!(
        "identifiable corpus: {cards} fixtures, {total} merges, {} failing",
        failures.len()
    );

    assert!(
        cards > 20,
        "only {cards} fixtures carry unambiguous identity"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// A 4.0 card wrapping the given property lines.
fn card(props: &str) -> String {
    format!("BEGIN:VCARD\r\nVERSION:4.0\r\n{props}END:VCARD\r\n")
}

/// Merge three cards given as text and return the merged text and the report's
/// conflict count.
fn merge_text(base: &str, left: &str, right: &str) -> (String, usize) {
    let base = VcardCst::parse(base).expect("a base card");
    let left = VcardCst::parse(left).expect("a left card");
    let right = VcardCst::parse(right).expect("a right card");

    let report = merge(&base, &left, &right);

    (report.merged.to_string(), report.conflicts.len())
}

#[test]
fn an_update_beats_a_parameter_removal_on_either_side() {
    // The merge module and cairn/spec/merge.md both promise that an update
    // beats a removal, so that data survives over silent loss. At property
    // granularity it holds on both sides. At parameter granularity the left
    // action simply wins, so a left-side parameter removal outranks a
    // right-side update and the outcome depends on which copy is called left.
    let base = card("TEL;PREF=1:+1\r\n");
    let removed = card("TEL:+1\r\n");
    let updated = card("TEL;PREF=2:+1\r\n");

    let (merged, conflicts) = merge_text(&base, &updated, &removed);
    assert_eq!(merged, card("TEL;PREF=2:+1\r\n"));
    assert_eq!(conflicts, 1);

    let (merged, conflicts) = merge_text(&base, &removed, &updated);
    assert_eq!(conflicts, 1);
    assert_eq!(merged, card("TEL;PREF=2:+1\r\n"), "the update was dropped");
}

#[test]
fn swapping_the_sides_reports_the_same_number_of_conflicts() {
    // One side removes TYPE outright while the other rewrites its single item.
    // Read one way that is one collision on the TYPE parameter; read the other
    // way it is two, one per replayed item edit. The collided field is the
    // same either way, so only a caller counting conflicts sees the
    // difference.
    let base = card("TEL;TYPE=work:+1\r\n");
    let removed = card("TEL:+1\r\n");
    let rewritten = card("TEL;TYPE=cell:+1\r\n");

    let (_, forward) = merge_text(&base, &removed, &rewritten);
    let (_, backward) = merge_text(&base, &rewritten, &removed);

    assert_eq!(forward, backward);
}

#[test]
fn a_quoted_parameter_value_may_carry_a_colon_and_a_semicolon() {
    // RFC 6350 section 3.3 lets a quoted parameter value hold any character
    // but a control or a double quote, `:` and `;` included, and section 6.3.1
    // uses exactly that in its ADR example. The line splitter is not quote
    // aware, so the head is cut at the first `:` and the parameters at every
    // `;`, which shifts part of the head into the value.
    let raw = card(
        "ADR;GEO=\"geo:12.3457,78.910\";TYPE=work:;;123 Main Street;Any Town;CA;91921-1234;U.S.A.\r\n",
    );
    let cst = VcardCst::parse(&raw).expect("an RFC 6350 ADR");
    let line = &cst.props[1];

    assert_eq!(line.params.len(), 2);
    assert_eq!(line.params[0].name.get(), "GEO");
    assert_eq!(line.params[0].values[0].get(), "\"geo:12.3457,78.910\"");
    assert_eq!(line.value.component_count(), 7);
}

#[test]
fn duplicate_list_items_are_diffed_as_a_multiset_and_replayed_as_a_set() {
    // NOTE: this pins current behaviour rather than a law, and is why the
    // generator only produces lists of distinct items. `list_diff` counts
    // duplicates one for one, so the second `a` reads as an addition on both
    // sides, while the replay is presence-guarded, so it lands only once.
    let base = card("NICKNAME:a\r\n");
    let side = card("NICKNAME:a,a\r\n");

    let (merged, conflicts) = merge_text(&base, &side, &side);

    assert_eq!(merged, card("NICKNAME:a,a\r\n"));
    assert_eq!(conflicts, 0);

    // And a duplicate one side removes is removed once, not once per copy.
    let (merged, conflicts) = merge_text(&side, &base, &side);
    assert_eq!(merged, card("NICKNAME:a\r\n"));
    assert_eq!(conflicts, 0);
}

#[test]
fn a_change_past_a_semicolon_of_a_uri_value_is_reported() {
    // A `data:` URI always carries a `;` before its base64 payload, and a URI
    // value does not escape it, so the value node splits into two components
    // while the decoded `VcardUri` reads only the first one. `diff_pair`
    // short-circuits on decoded equality, so a change confined to the payload
    // produces no action at all: it neither lands nor is reported, which is
    // silent loss of a whole photo.
    let base = card("PHOTO:data:image/png;base64,AAAA\r\n");
    let right = card("PHOTO:data:image/png;base64,BBBB\r\n");

    let base_cst = VcardCst::parse(&base).expect("a base card");
    let left_cst = VcardCst::parse(&base).expect("an untouched left card");
    let right_cst = VcardCst::parse(&right).expect("an edited right card");

    let report = merge(&base_cst, &left_cst, &right_cst);

    assert_eq!(report.right.len(), 1, "the change was not even diffed");
    assert_eq!(report.merged.to_string(), right, "the change did not land");
}

#[test]
fn divergent_changes_past_a_semicolon_of_a_uri_value_conflict() {
    // The same blind spot on both sides: two copies replace the photo payload
    // with different images, and the merge reports no disagreement at all.
    let base = card("PHOTO:data:image/png;base64,AAAA\r\n");
    let left = card("PHOTO:data:image/png;base64,BBBB\r\n");
    let right = card("PHOTO:data:image/png;base64,CCCC\r\n");

    let (_, conflicts) = merge_text(&base, &left, &right);

    assert_eq!(conflicts, 1, "two divergent photos were not reported");
}

/// One side of a matching case: the order it lists the three `PID`-tagged
/// instances in, the words it rewrites some of them to, and one it removes.
#[derive(Clone, Debug)]
struct Side {
    /// The positions the three instances appear at.
    order: [usize; 3],
    /// Per instance, the word it is rewritten to, or `None` if untouched.
    edits: [Option<usize>; 3],
    /// The instance this side removed, if any.
    removed: Option<usize>,
}

impl Side {
    /// The value this side gives an instance, or `None` if it removed it.
    fn value(&self, pid: usize) -> Option<String> {
        if self.removed == Some(pid) {
            return None;
        }

        Some(match self.edits[pid] {
            Some(word) => WORDS[word].to_string(),
            None => format!("base{pid}@x.test"),
        })
    }

    /// This side's card: the three instances in its own order, minus the one
    /// it removed.
    fn card(&self) -> String {
        let lines: String = self
            .order
            .iter()
            .filter_map(|&pid| {
                self.value(pid)
                    .map(|value| format!("EMAIL;PID={}:{value}\r\n", pid + 1))
            })
            .collect();

        card(&lines)
    }
}

/// The three instances in order, untouched: the common base.
fn matching_base() -> String {
    let side = Side {
        order: [0, 1, 2],
        edits: [None; 3],
        removed: None,
    };

    side.card()
}

/// One side of a matching case, biased so an edit is more likely than not.
fn arb_side() -> impl Strategy<Value = Side> {
    let orders = prop::sample::select(vec![
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ]);
    let edit = prop_oneof![2 => Just(None), 3 => (0usize..3).prop_map(Some)];
    let removed = prop_oneof![4 => Just(None), 1 => (0usize..3).prop_map(Some)];

    (orders, [edit.clone(), edit.clone(), edit], removed).prop_map(|(order, edits, removed)| Side {
        order,
        edits,
        removed,
    })
}

proptest! {
    #![proptest_config(config(512))]

    /// `PID` is instance identity: however the two sides reorder their
    /// instances, each one's edit lands on the instance carrying its `PID`,
    /// nothing is lost to a reorder, and a divergence is reported.
    #[test]
    fn pid_matching_survives_a_reorder(left in arb_side(), right in arb_side()) {
        let base = matching_base();
        let (left_text, right_text) = (left.card(), right.card());
        let base_cst = VcardCst::parse(&base).unwrap();
        let left_cst = VcardCst::parse(&left_text).unwrap();
        let right_cst = VcardCst::parse(&right_text).unwrap();

        let report = merge(&base_cst, &left_cst, &right_cst);
        let merged = model_of(&report.merged);
        let keys = conflict_keys(&report, &model_of(&base_cst));

        for pid in 0..3 {
            let id = format!("EMAIL#{}", pid + 1);
            let held = merged
                .iter()
                .find(|prop| prop.id() == id)
                .map(|prop| prop.comps[0][0].clone());
            let value = Some;

            let (base, lv, rv) = (
                format!("base{pid}@x.test"),
                left.value(pid),
                right.value(pid),
            );
            let reported = keys.contains(&(id.clone(), Field::Whole))
                || keys.contains(&("EMAIL".to_string(), Field::Presence));

            match (lv, rv) {
                // Both sides kept it: whoever changed it wins, and a
                // divergence is reported.
                (Some(l), Some(r)) => match (l == base, r == base) {
                    (true, true) => prop_assert_eq!(held, value(base)),
                    (false, true) => prop_assert_eq!(held, value(l)),
                    (true, false) => prop_assert_eq!(held, value(r)),
                    (false, false) if l == r => prop_assert_eq!(held, value(l)),
                    (false, false) => {
                        prop_assert_eq!(held, value(l));
                        prop_assert!(reported, "a divergent edit of {} went unreported", id);
                    }
                },
                // One side removed it: an update on the other beats the
                // removal, and the collision is reported.
                (None, Some(r)) | (Some(r), None) if r != base => {
                    prop_assert_eq!(held, value(r), "an update lost to a removal");
                    prop_assert!(reported, "a remove against update of {} went unreported", id);
                }
                // A removal against an untouched or removed instance.
                _ => prop_assert_eq!(held, None),
            }
        }
    }
}

#[test]
fn a_removal_does_not_renumber_what_follows_it() {
    // Every edit runs in place on the left clone before any line is removed
    // or added, removals are then applied on descending indices, and each
    // addition recomputes where it goes. So a line removed near the top does
    // not shift the line an edit further down addresses.
    let base = card("NOTE:drop\r\nFN:a\r\nTEL:+1\r\nNICKNAME:x\r\n");
    let right = card("FN:b\r\nTEL:+2\r\nNICKNAME:x,y\r\nEMAIL:e@x.test\r\n");

    let (merged, conflicts) = merge_text(&base, &base, &right);

    assert_eq!(
        merged,
        card("FN:b\r\nTEL:+2\r\nNICKNAME:x,y\r\nEMAIL:e@x.test\r\n"),
    );
    assert_eq!(conflicts, 0);
}

#[test]
fn an_interchangeable_duplicate_keeps_the_copy_all_three_carry() {
    // Three `GENDER:M` lines, one of them spelled with a different line
    // ending: they all decode alike, so the matching may pair any of them, and
    // pairing by decoded equality alone dropped the copy the other two copies
    // carried byte for byte. Found by the fuzz target.
    let base = "GENDER:M\r\nGENDER:M\nGENDER:M\n";
    let right = "GENDER:M\nGENDER:M\n";

    let (merged, _) = merge_text(base, base, right);

    assert_eq!(merged, "GENDER:M\nGENDER:M\n");
}

#[test]
fn removing_every_line_leaves_no_card() {
    // A bare record carrying only envelope lines has no properties at all, so
    // merging it against a record of one property removes that property and
    // leaves nothing. Nothing is not a card: the merge does not invent a line
    // to keep the output parseable, and a caller reads the emptiness as what
    // it is. Found by the fuzz target.
    let base = "FN:a\r\n";
    let right = "END:VCARD\r\n";

    let (merged, conflicts) = merge_text(base, base, right);

    assert!(merged.is_empty(), "{merged:?}");
    assert_eq!(conflicts, 0);
}

#[test]
fn removing_a_duplicate_item_twice_takes_one_copy() {
    // A list holding one item twice makes a removal non-idempotent: the left
    // clone already dropped one copy, so replaying the right side's identical
    // removal would take a second copy neither side wrote off. `TYPE=work,,`
    // and `NICKNAME:a,a` are both real shapes. Found by the fuzz target.
    let base = card("NICKNAME:a,a\r\nTEL;TYPE=work,,:+1\r\n");
    let side = card("NICKNAME:a\r\nTEL;TYPE=work,:+1\r\n");

    let (merged, conflicts) = merge_text(&base, &side, &side);

    assert_eq!(merged, side);
    assert_eq!(conflicts, 0);
}

#[test]
fn two_instances_under_one_pid_pair_by_equality_first() {
    // `PID` is instance identity, but a card may carry two instances of one
    // name under one `PID`, and pairing them in source order then breaks the
    // pair that needs no change at all: the untouched `TEL` is matched with
    // the edited one, so both lines are rewritten and the one all three copies
    // carry is gone. Found by the fuzz target.
    let base = card("TEL;PID=1.1:+1\r\nTEL;PID=1.1:+2\r\n");
    let right = card("TEL;PID=1.1:+3\r\nTEL;PID=1.1:+2\r\n");

    let (merged, _) = merge_text(&base, &base, &right);

    assert!(merged.contains("TEL;PID=1.1:+2\r\n"), "{merged}");
    assert_eq!(merged, right);
}

#[test]
fn a_replayed_parameter_item_keeps_its_wire_form() {
    // A parameter value is unescaped on the way in and copied verbatim on the
    // way out, so an item replayed as its decoded text turns a `\n` into a
    // real line break and cuts the line in two, leaving a card that does not
    // parse. Found by the fuzz target.
    let base = card("TEL;TYPE=work:+1\r\n");
    let right = card("TEL;TYPE=work,a\\nb:+1\r\n");

    let (merged, conflicts) = merge_text(&base, &base, &right);
    let reparsed = VcardCst::parse(&merged).expect("a merged card");

    assert_eq!(merged, right);
    assert_eq!(reparsed.to_string(), merged);
    assert_eq!(conflicts, 0);
}

#[test]
fn a_list_item_edit_does_not_land_on_a_replaced_value() {
    // The left side replaces the whole value (here by declaring `VALUE=text`,
    // which changes the decoded kind) while the right side only adds one item
    // to the list it still sees. `Slot::collides_with` pairs a left item edit
    // with a right whole-value change but not the other way round, and the
    // item replay is unguarded, so the right item lands on the left's
    // replacement: the merged value is a hybrid neither side wrote, and
    // nothing is reported.
    let base = card("CATEGORIES:a,b\r\n");
    let left = card("CATEGORIES;VALUE=text:x\r\n");
    let right = card("CATEGORIES:a,b,c\r\n");

    let (merged, conflicts) = merge_text(&base, &left, &right);

    assert_ne!(
        merged,
        card("CATEGORIES;VALUE=text:x,c\r\n"),
        "a hybrid value"
    );
    assert_eq!(conflicts, 1, "the collision was not reported");
}

#[test]
fn identical_bytes_at_two_versions_are_not_a_change() {
    // A backslash before a colon is literal in 2.1 and resolves later, so one
    // line's bytes decode into two different values depending on which card
    // carries them. Comparing through the decoded value would call that a
    // change and rewrite a line neither side touched; found by the fuzzer,
    // once the replay started transcoding what it copied.
    let base = "BEGIN:VCARD\r\nVERSION:2.1\r\nURL:http\\://x.test\r\nEND:VCARD\r\n";
    let right = "BEGIN:VCARD\r\nVERSION:4.0\r\nURL:http\\://x.test\r\nEND:VCARD\r\n";

    let (merged, conflicts) = merge_text(base, base, right);

    assert_eq!(merged, base);
    assert_eq!(conflicts, 0);
}

#[test]
fn replaying_across_versions_keeps_the_meaning_of_a_value() {
    // A 4.0 `NOTE` escapes a comma, a 2.1 one does not. Replaying the right
    // line's raw bytes onto a card of the other version carries the escaping
    // with them, so `a,c` arrives in the 2.1 card as the literal `a\,c`.
    let base = "BEGIN:VCARD\r\nVERSION:2.1\r\nFN:x\r\nNOTE:a,b\r\nEND:VCARD\r\n";
    let left = "BEGIN:VCARD\r\nVERSION:2.1\r\nFN:y\r\nNOTE:a,b\r\nEND:VCARD\r\n";
    let right = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:x\r\nNOTE:a\\,c\r\nEND:VCARD\r\n";

    let (merged, _) = merge_text(base, left, right);
    let merged = VcardCst::parse(&merged).expect("a merged card");
    let version = merged.version();

    assert_eq!(
        format!("{:?}", merged.props[2].decode(version).value),
        "Text(VcardText(\"a,c\"))",
    );
}

/// The path an action carries: the property name it names and the instance
/// index inside it.
fn action_path<'a>(action: &'a VcardMergeAction<'_>) -> (&'a str, usize) {
    let at = match action {
        VcardMergeAction::PropAdded { at, .. }
        | VcardMergeAction::PropRemoved { at, .. }
        | VcardMergeAction::ValueChanged { at, .. }
        | VcardMergeAction::ValueComponentChanged { at, .. }
        | VcardMergeAction::ValueItemAdded { at, .. }
        | VcardMergeAction::ValueItemRemoved { at, .. }
        | VcardMergeAction::ParamAdded { at, .. }
        | VcardMergeAction::ParamRemoved { at, .. }
        | VcardMergeAction::ParamChanged { at, .. }
        | VcardMergeAction::ParamItemAdded { at, .. }
        | VcardMergeAction::ParamItemRemoved { at, .. } => at,
    };

    (&at.name, at.index)
}

#[test]
fn identical_edits_of_a_repeated_parameter_do_not_conflict() {
    // `TEL;TYPE=WORK;TYPE=VOICE` is ordinary in 2.1 and 3.0 cards, and the
    // corpus carries it. Two parameters of one name make `diff_params` emit
    // two actions on the one `Slot::Param`, so a side's second action has to
    // be matched against the left action it repeats rather than against the
    // first action on that slot.
    let base = "BEGIN:VCARD\r\nVERSION:3.0\r\nTEL;TYPE=work;TYPE=home:+1\r\nEND:VCARD\r\n";
    let side = "BEGIN:VCARD\r\nVERSION:3.0\r\nTEL;TYPE=cell;TYPE=fax:+1\r\nEND:VCARD\r\n";

    let (merged, conflicts) = merge_text(base, side, side);

    assert_eq!(merged, side);
    assert_eq!(conflicts, 0, "a side disagreed with itself");
}

#[test]
fn a_property_added_after_an_unterminated_line_stays_its_own_line() {
    // A bare RFC 2425 record read without a trailing newline leaves its last
    // line with an empty line ending, which is right while it is last. The
    // merge appends the right side's additions after it without terminating
    // it, so the added line is glued onto the previous value and reparses as
    // part of it: the addition is destroyed, silently.
    let base = "FN:a\r\nNOTE:b";
    let right = "FN:a\r\nNOTE:b\r\nTEL:+1";

    let (merged, _) = merge_text(base, base, right);
    let merged = VcardCst::parse(&merged).expect("a merged record");

    assert_eq!(merged.props.len(), 3, "the addition was glued onto NOTE");
}

#[test]
fn a_folded_line_is_normalised_once_and_then_survives_a_merge() {
    // NOTE: the parser resolves folding and does not restore it (see
    // cairn/spec/parsing.md, Line normalisation), so a folded card is
    // rewritten unfolded the first time it is parsed. What the merge owes is
    // that this happens once: an untouched line of an already-normalised card
    // keeps its bytes through a merge.
    let folded =
        "BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:aaaaaaaaaa\r\n bbbbbbbbbb\r\nFN:a\r\nEND:VCARD\r\n";
    let unfolded = VcardCst::parse(folded).expect("a folded card").to_string();

    assert_eq!(
        unfolded,
        "BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:aaaaaaaaaabbbbbbbbbb\r\nFN:a\r\nEND:VCARD\r\n",
    );

    let right = unfolded.replace("FN:a", "FN:b");
    let (merged, conflicts) = merge_text(&unfolded, &unfolded, &right);

    assert_eq!(merged, right);
    assert_eq!(conflicts, 0);
}

#[test]
fn an_addition_lands_on_the_card_that_owns_the_property() {
    // A vCard 2.1 `AGENT` embeds a whole card, whose lines the parser keeps
    // verbatim among the outer card's properties. `Merger::finish` places an
    // addition after the last line sharing its name, which here is the agent's
    // own `FN`, so the outer card's new `FN` lands inside the agent.
    let base = concat!(
        "BEGIN:VCARD\r\nVERSION:2.1\r\nFN:Boss\r\nAGENT:\r\n",
        "BEGIN:VCARD\r\nVERSION:2.1\r\nFN:Secretary\r\nEND:VCARD\r\n",
        "END:VCARD\r\n",
    );
    let right = concat!(
        "BEGIN:VCARD\r\nVERSION:2.1\r\nFN:Boss\r\nFN:Boss II\r\nAGENT:\r\n",
        "BEGIN:VCARD\r\nVERSION:2.1\r\nFN:Secretary\r\nEND:VCARD\r\n",
        "END:VCARD\r\n",
    );

    let (merged, _) = merge_text(base, base, right);

    assert_eq!(merged, right, "the addition landed inside the agent");
}

#[test]
fn an_edit_inside_an_embedded_agent_still_merges() {
    // The other half of the same rule: only the envelope lines are skipped and
    // only the *placement* of an addition is kept out of the agent, so an edit
    // to a line the agent owns is diffed and replayed like any other. Making
    // the whole embedded run opaque instead would drop this edit and report
    // nothing, trading one silent loss for another.
    let base = concat!(
        "BEGIN:VCARD\r\nVERSION:2.1\r\nFN:Boss\r\nAGENT:\r\n",
        "BEGIN:VCARD\r\nVERSION:2.1\r\nFN:Secretary\r\nEND:VCARD\r\n",
        "END:VCARD\r\n",
    );
    let right = base.replace("FN:Secretary", "FN:Assistant");

    let (merged, conflicts) = merge_text(base, base, &right);

    assert_eq!(merged, right);
    assert_eq!(conflicts, 0);
}

#[test]
fn an_end_line_replayed_as_a_property_does_not_close_the_card() {
    // A bare RFC 2425 record has no envelope, so an `END:VCARD` line in it is
    // an ordinary property. Replaying it into a wrapped card puts a second
    // `END` before the real one, and the reparse stops at the first: every
    // line after it is dropped.
    let base = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:a\r\nEND:VCARD\r\n";
    let right = "FN:a\r\nEND:VCARD\r\n";

    let (merged, _) = merge_text(base, base, right);
    let reparsed = VcardCst::parse(&merged).expect("a merged card");

    assert_eq!(
        reparsed.to_string(),
        merged,
        "the merged card lost its tail"
    );
}

#[test]
fn two_sides_adding_one_type_set_in_two_orders_agree() {
    // RFC 6350 section 5.6 gives `TYPE` a comma-separated list of type values
    // with no ordering, and the merge agrees with that as long as the base
    // carries the parameter: it then diffs the items and a reorder is a no-op.
    // When the base does not, both sides' additions are whole parameters,
    // compared in order, so adding one set in two orders is a disagreement.
    let base = card("TEL:+1\r\n");
    let left = card("TEL;TYPE=work,cell:+1\r\n");
    let right = card("TEL;TYPE=cell,work:+1\r\n");

    let (_, added) = merge_text(&base, &left, &right);

    let reordered = card("TEL;TYPE=work,cell:+1\r\n");
    let (_, existing) = merge_text(&reordered, &right, &right);

    assert_eq!(existing, 0);
    assert_eq!(added, 0, "an order-only difference was a disagreement");
}
