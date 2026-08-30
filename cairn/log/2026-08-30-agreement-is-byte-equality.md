---
cairn: log
change: agreement-is-byte-equality
landed: 2026-08-30
---

# Agreement is byte equality

One defect shape has now been fixed at four sites in three days: comparing the decoded form of something whose decode is not injective. Values (`value_eq`), URIs and parameters (`param_eq`) were the first three. Change-level agreement was the fourth and the last.

`same_change` was decoded throughout, with `left == right` as its fallback, and the whole-value arm of the replay short-circuited on `value_eq`, which compares decoded components. `\N` and `\n` both unescape to a line break (RFC 6350 section 3.4), so a left copy holding `FN:Ada\nLovelace` and a right copy holding `FN:Ada\NLovelace` compared equal, the right side's change was skipped as already made, and the two sides were told they had agreed on bytes they had not agreed on.

`already_made` is now `same_change` over the decoded action followed by `wrote_alike` over the bytes the change itself put on the wire: the value it wrote, the `;`-component it wrote, the item a list gained, the parameter it wrote. A change that only takes something away wrote no bytes, and what it names lives in the base both sides share, so the change itself settles it. The whole-value, list-item and parameter-item arms of the replay now go through that one gate rather than each carrying its own test. `VcardValueNode::raw_list` and `raw_component_list` are the raw twins of `decode_list` and `decode_component_list`, which is what lets an item or a component be weighed on its own bytes.

The merged bytes did not move. A refused agreement is judged like any other change, meets the left side's on the same field, and is recorded as a conflict, and the left side keeps its value. Only the report gains an entry, which is the whole point: the difference is said out loud instead of vanishing.

`TYPE` and `PID` stay the one exception, and it now holds on the raw side as well. RFC 6350 sections 5.6 and 7 give them no order, so `param_alike` compares an unordered parameter's raw items as a set; `unordered` names the two kinds once, beside `same_param`, which already sorted their decoded items. Without that second half the byte rule would have undone the exception: `TYPE=work,cell` and `TYPE=cell,work` are different bytes.

Spec updated: `merge` (MODIFIED: a change both sides made is never a conflict).
