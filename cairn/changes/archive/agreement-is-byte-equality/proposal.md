---
cairn: change
id: agreement-is-byte-equality
status: landed
created: 2026-08-30
---

# Agreement is byte equality

## Why

One defect shape has been fixed three times in two days: comparing the decoded form of something whose decode is not injective. It was fixed for values (`value_eq`), for URIs and for parameters (`param_eq`). Change-level agreement was the fourth site and still compared decoded actions.

`same_change` was decoded throughout, with `left == right` as its fallback, and the whole-value arm short-circuited on `value_eq`, which is a decoded comparison too. So two sides that wrote different bytes decoding alike, `FN:a\nb` against `FN:a\Nb`, compared equal, the right side's change was skipped as already made, and the divergence was never reported.

## What

Make agreement byte equality at the granularity of the change itself: the value it wrote, the `;`-component it wrote, the item a list gained, the parameter it wrote. A change that only takes something away wrote no bytes, and what it names lives in the base both sides share, so the change settles it.

`TYPE` and `PID` stay the exception, now on the raw side as well as the decoded one: RFC 6350 sections 5.6 and 7 give them no order, so writing one set in two orders stays one change.

Done when a spelling-only difference is reported rather than swallowed, and both crates state one rule.

## Consequence

The merged bytes do not move: a refused agreement is judged normally, collides with the left change, and the left side keeps its value, so only the report gains an entry.
