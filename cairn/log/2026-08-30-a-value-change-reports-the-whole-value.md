---
cairn: log
change: a-value-change-reports-the-whole-value
landed: 2026-08-30
---

# A whole-value change reports what changed

The follow-up `merge-diffs-value-nodes` named and deferred. That change moved the comparison onto the raw value nodes, so a change past a value's first `;`-component is seen and lands, but the `ValueChanged` payload stayed decoded and a non-structured value decodes its first component alone. A right copy rewriting `NOTE:a;b` to `NOTE:a;CHANGED` was reported as `ValueChanged { old: Text("a"), new: Text("a") }`: truncated on both ends, and equal, so the report said nothing had changed while the merged card carried the change.

The payload is now built from the node. A decoded value that encodes back to what its node holds is reported as it is decoded, which covers every value the model reads whole, a `data:` URI and a `GEO:lat;lon` included; anything else is reported as the node's raw components, `VcardValue::Unknown`. The merged bytes are untouched: this was always a reporting defect, and its cost was a caller resolving a conflict, or displaying one, on values that were not there.

Spec updated: `merge` (ADDED: a whole-value change reports the whole value).
