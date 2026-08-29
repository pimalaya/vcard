---
cairn: change
id: a-folded-line-is-stripped-once-assembled
status: landed
created: 2026-08-29
---

# A folded line is stripped once assembled, not only at its first physical line

## Why

Found by the merge fuzz target, which produced a merged card that was not a serialization fixpoint. The merge was innocent: one of its inputs was a card the parser had already read wrong.

A line beginning with folding whitespace that has no line to continue gets that whitespace stripped, which is the documented normalisation. The stripping runs on the *first* physical line only. A first line made entirely of whitespace therefore strips to nothing, and the continuation that follows it contributes everything, with only its one fold marker removed:

    "   \r\n  A:b\r\n"   parses to a line named " A"

That line serializes as ` A:b`, which reparses as a continuation of whatever precedes it, or as a line named `A` when it stands first. Either way the card is not a fixpoint, which the parsing capability requires of any input.

## What

The strip runs again on the assembled logical line, in both the folded and the QUOTED-PRINTABLE join paths, so no line ever begins with folding whitespace whatever it was assembled from.

A card with no whitespace-only line is unaffected: its assembled line already starts with the stripped first physical line.
