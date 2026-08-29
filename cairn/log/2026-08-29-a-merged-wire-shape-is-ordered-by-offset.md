---
cairn: log
change: a-merged-wire-shape-is-ordered-by-offset
landed: 2026-08-29
---

# A merged wire shape is ordered by offset

Spec only. `VcardWire::prepend` already merges the tokeniser's pieces with the line splitter's by a stable sort on the offset, and the reason was written down in the log of `a-line-remembers-its-wire-shape` and nowhere else.

The reason is worth a requirement. A value ending on two `=` is recorded by both recorders at once, the tokeniser's soft break one byte past the last logical byte and the splitter's dangling `=` on it, and emitting them in list order writes `x=\r\n=` for an input that read `x==`. Those bytes reparse with the following line joined into the value, so a round-trip that promises byte fidelity loses a line.

ical-rs carried the concatenation this rule replaced and lost `END:VCALENDAR` into a `NOTE` for exactly that input. It found out only because the shape was ported and the twin's fuzz regression came with it. A rule whose absence costs a line belongs in the spec of both crates.

Capabilities moved: parsing ("Line normalisation" MODIFIED, gaining the ordering rule and its scenario).
