---
cairn: change
id: merge-property-testing
status: landed
created: 2026-08-29
---

# The three-way merge earns property, differential and fuzz coverage

The merge is the reconciliation unit every Pimalaya synchronisation engine builds on, and until now it was covered by fourteen hand-written scenarios in its own module. Hand-written scenarios pin the cases their author thought of; they cannot state the laws the merge owes a caller, and they cannot say that nothing is lost in the cases nobody thought of. A sibling merge system lost data silently the day before this work started, which is the class this change hunts.

## What changes

Only tests, one dev-dependency and one fuzz target. No behaviour changes.

tests/merge.rs adds three layers over one plain-data model of a card. The algebraic **laws**: an untouched side contributes nothing, two identical edits are not a disagreement, the merged card reparses to a fixpoint, a line all three copies carry keeps its bytes, swapping the sides names the same collided fields, and re-merging the merged card changes nothing. The **completeness law**, stated field by field: every change either lands or is named in the report's conflicts, and nothing appears that neither side wrote. A **differential** against a deliberately naive second merge that models a card as plain field maps and reconciles by the documented rules, compared on normalised content and on conflict keys.

The generator is a fixed base card carrying one property of each value shape, with and without parameters, and in one mode with repeated property names and a distinct `PID` on every instance. Both sides draw their edits from one small biased target space, so 36% of generated triples actually collide; the rate is measured by a test that fails if the generator drifts.

The same edits run over the whole corpus through the crate's own edit layer, and fuzz/fuzz_targets/merge.rs carves three cards from one libFuzzer input and asserts the laws that need no model.

`proptest` joins the dev-dependencies, with its regression seeds in tests/merge.proptest-regressions.

## What it found

Seven defects with minimal reproductions, all committed as `#[ignore]` tests naming their write-up, none fixed here: a value edit past the first semicolon of a URI value is neither merged nor reported (silent loss of an inline photo), two parameters of one name make a side disagree with itself, an addition after an unterminated line is glued onto it, an addition lands inside an embedded `AGENT` card, a right list-item edit lands on a value the left side replaced, an update does not beat a removal at parameter granularity, and two sides adding one `TYPE` set in two orders are reported as disagreeing. Each wants its own decision, so each wants its own change. One further defect found on the way is not the merge's: a quoted parameter value carrying a colon or a semicolon is split by the line parser, which mis-decodes the RFC 6350 section 6.3.1 ADR example the corpus already carries.
