---
cairn: log
change: merge-diffs-value-nodes
landed: 2026-08-29
---

# The merge diffs raw value nodes, not the lossy decoded projection

Closed a silent data-loss path. `diff_pair` returned early when the two sides' *decoded* values agreed, and that projection reads a non-structured value's first `;`-component alone, truncated again at its first unescaped `,`. Two divergent inline photos, or two divergent notes carrying a comma, produced no action at all: the change neither landed nor was reported, and a caller that resolves on an empty conflict report discarded one of them with nobody asked.

Values now compare on the raw value node, component by component, at the three sites that decided anything from them: the short circuit in `diff_pair`, the equality pass of the instance matching, and the both-sides-added check. Whole-value agreement is decided on the nodes rather than on action equality, since the action keeps the decoded value as its payload. `VcardText` also stopped truncating at an unescaped comma.

tests/merge.rs models a non-structured value as its whole node rather than as the decoded projection, which lifted the exclusion the suite carried, and both reproductions run.

Spec updated: `merge` (ADDED: values are compared on the raw node), `decoded-model` (MODIFIED: open payloads keep unmodelled data).
