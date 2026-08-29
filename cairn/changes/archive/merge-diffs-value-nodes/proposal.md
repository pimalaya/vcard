---
cairn: change
id: merge-diffs-value-nodes
status: landed
created: 2026-08-29
---

# The merge diffs raw value nodes, not the lossy decoded projection

## Why

`diff_pair` opens by comparing the two sides' *decoded* values and returning when they agree. For every value kind but the five component-structured ones, that projection is built from the value node's first `;`-component alone, and a text value is truncated again at its first unescaped `,`. Two divergent values therefore look identical, no `VcardMergeAction` is emitted at all, and the change neither lands nor appears in the report.

The reachable case is an inline photo: a `data:` URI always carries a `;` before its payload, so `PHOTO:data:image/png;base64,AAAA` and `PHOTO:data:image/png;base64,BBBB` compare equal. A plain unescaped comma does the same to any text value, so `NOTE:hello, world` and `NOTE:hello, everyone` also compare equal.

This is silent loss, and it reaches production: neverest's automatic merge resolves on an empty conflict report, so two devices each setting a different photo produce no conflict, the left copy wins, and the right one is discarded with nobody asked.

## What

The merge stops reading values through the decoded model and compares them on the raw value node, component by component, over the whole value. Three sites move: the equality short circuit in `diff_pair`, the equality pass of the instance `matching`, and the both-sides-added check in `apply_added`.

Agreement between two whole-value changes is likewise decided on the value nodes rather than on `VcardMergeAction` equality, since the action keeps the decoded value as its payload and two actions can compare equal while the values differ.

`VcardText` also stops truncating at an unescaped comma: it reads the whole first component, joined, as `VcardUri` already does. A comma inside a text value must be escaped, so an unescaped one is malformed input that Postel's law says to keep, never to cut the value at.

The completeness and differential layers in tests/merge.rs model a non-structured value as the decoded projection precisely because that is what the merge saw. They now model the whole value node, which lifts the exclusion, and the two reproductions run.

## Judgement calls, for review

**Components are compared item by item, not as joined text.** Two nodes agree when every component decodes to the same list of items. That makes the comparison escaping-sensitive: `NOTE:a\,b` (one item holding a comma) and `NOTE:a,b` (two items) are a difference, and two sides that wrote the same text with different escaping collide. The alternative, comparing each component joined, is escaping-insensitive but silently misses a change that only re-escapes, and "conflicts are reported, never silently resolved away" says a visible false conflict beats an invisible loss.

**Trailing empty components are still absent.** `component_eq` already treats an absent component and an empty one alike for a structured value; whole-value comparison inherits that, so `NOTE:a` and `NOTE:a;` agree.

**The action payload stays decoded.** `VcardMergeAction::ValueChanged` keeps `old` and `new` as `VcardValue`, so a caller can display them, and the merge no longer decides anything from them. `VcardUri` still reads only the first component, so a `data:` URI's payload is still missing from a decoded card and from that payload. Repairing it needs a matching non-escaping encode (a URI is not backslash-escaped on the wire, but the value codec escapes every scalar leaf), which moves the decoded model, jCard and JSContact. That is a separate defect and wants its own change.
