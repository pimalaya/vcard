---
cairn: log
change: a-folded-line-is-stripped-once-assembled
landed: 2026-08-29
---

# A folded line is stripped once assembled, not only at its first physical line

Found by the merge fuzz target, on a card the parser had already read wrong: the merge was handed a line whose name began with a space, and the merged card folded it back into its predecessor on reparse.

The leading-whitespace strip ran on a line's first physical line only, so a first line made entirely of whitespace stripped to nothing and the continuation that followed contributed everything but its one fold marker, leaving the leftover in front of the name. `"   \r\n  A:b\r\n"` parsed to a line named `" A"`, and no card carrying one reparses to itself.

The strip now runs again on the assembled logical line, in both the folded and the QUOTED-PRINTABLE join paths.

Spec updated: `parsing` (MODIFIED: line normalisation).
