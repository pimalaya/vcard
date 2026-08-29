---
cairn: change
id: replayed-parameter-items-keep-their-wire-form
status: landed
created: 2026-08-29
---

# A replayed parameter item is written as the right card spelled it

## Why

Found by the merge fuzz target, which produced a card the merge could no longer parse back: `MissingPropertyColon("ADR;TYPE=WORK,P Waters Edge")`.

A parameter value is read through the value unescaper (`TYPE::decode` runs `unescape` over every leaf) and written back verbatim, since a parameter's wire form is quoted rather than backslash-escaped. The two halves are not inverses, and the item replay wrote the decoded text:

    node.values.push(VcardLeaf::from(item.to_string()))

So a wire `TYPE=a\nb` decodes to an item holding a real line break, and pushing that item back puts the break in the middle of the line's head. The line ends there, the rest folds onto whatever follows, and the merged card no longer has a colon on that line.

## What

The replay writes the leaf the right card holds rather than the decoded item, which is what the neighbouring whole-parameter replay already does. The item was decoded from that node, so the leaf is always there; with no leaf there is no wire form to write and nothing is pushed.

## Judgement call, for review

**The fix stays in the merge, not in the parameter codec.** The same asymmetry lets `VcardParam::encode` write a decoded parameter value verbatim, so a decoded card carrying a line break in a `TYPE` serializes to a broken card too. Repairing that means deciding how a parameter value is quoted on the way out (RFC 6350 section 3.3 gives `DQUOTE`-quoting, not escaping, and no spelling at all for a control character), which moves every lens and the jCard export with it. The merge does not need that decision: it is a byte-preserving layer and has the right card's own bytes to hand.
