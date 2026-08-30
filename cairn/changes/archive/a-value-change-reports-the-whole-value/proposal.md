---
cairn: change
id: a-value-change-reports-the-whole-value
status: landed
created: 2026-08-30
---

# A whole-value change reports what changed

## Why

The follow-up the `merge-diffs-value-nodes` change named and deferred: the comparison moved onto the raw value nodes, but the `ValueChanged` payload stayed decoded, and a non-structured value decodes its first `;`-component alone.

A base and a left copy holding `NOTE:a;b` against a right copy holding `NOTE:a;CHANGED` therefore report `ValueChanged { old: Text("a"), new: Text("a") }`: truncated on both ends, and equal, so the report says nothing changed. The merged bytes are right, which is what keeps this a reporting defect rather than data loss, but a caller resolving a conflict from the report cannot see either value, and a caller displaying one shows the wrong text.

## What

The payload is built from the node rather than from the decoded value. A value whose decoded projection encodes back to what its node holds is reported as it is decoded, which is every value the model reads whole; anything else is reported as the node's raw components, `VcardValue::Unknown`, which is what `VcardValueUnknown` exists for.

## Judgement calls, for review

**The round trip is the test, not the value kind.** Asking whether the node has more than one component would demote every `GEO:lat;lon` and every `data:` URI, both of which decode whole. Re-encoding the decoded value and comparing it with the node asks the question that actually matters, and costs one encode on a path that only runs for a value that already changed.

**A truncated value is reported as `Unknown` rather than as its kind, joined.** Joining the components back into a `VcardText` would keep the kind, but it would also invent a value: the joined text re-encodes with the `;` escaped, which is not the bytes the card carries. The raw components are what the merged card holds.
