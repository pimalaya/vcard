---
cairn: log
change: a-line-remembers-its-wire-shape
landed: 2026-08-29
---

# A line remembers how it was laid out on the wire

The crate promised a byte-faithful CST while the parser quietly unfolded continuation lines, dropped the blank lines between properties and resolved QUOTED-PRINTABLE soft breaks without restoring any of them. Real exports from Apple, iOS and Google fold heavily, so an untouched round trip rewrote most of a card. Of the 146 corpus fixtures, 50 fold, 22 carry a QUOTED-PRINTABLE encoding and 15 hold a blank line. All 146 now come back byte for byte.

The mechanism is `tree::wire::VcardWire`, the twin of ical-rs's `IcalWire`: a list of byte offsets into a line's *logical* bytes (its name, its parameters and its value, exactly as the serializer lays them out) with the piece of wire that sat at each offset. Three pieces cover everything the tokeniser resolves. A fold keeps the break and the single whitespace apart, so `\n\t` does not come back as `\r\n `. A soft break is the `=` and the break after it, which is what tells the two mechanisms apart: a fold is a break followed by whitespace, a soft break is an `=` followed by a break, and only a line whose head declares `ENCODING=QUOTED-PRINTABLE` can carry one. A run of skipped bytes covers the rest: the blank lines before a line, the whitespace of a dangling continuation, a trailing `=` with nothing to continue.

The offsets are checked rather than trusted. The logical length is sealed alongside them, and a shape whose length no longer matches the line is dropped rather than applied: an edit that changes a value's length moves every byte after it, so the old fold points would land in the wrong places. An edited line therefore goes out unfolded, which RFC 6350 3.2 permits, and an edit that keeps the length keeps the shape. The guard is on the length rather than on a mutation flag deliberately: the fields of a line are public, so a mutation flag would be bypassed by a direct write and a length check cannot be.

Two divergences from ical-rs, both deliberate. The first is that the leading whitespace of a line assembled from continuations is still stripped from the *assembled* line rather than from its first physical line alone, the rule `a-folded-line-is-stripped-once-assembled` landed earlier today. ical-rs lets the whitespace stay in the name and leans on the wire shape to reproduce it, which is enough there; here the merge composes a card line by line and gives a composed line no wire shape, so a line named `" A"` would go out bare and fold into its predecessor on reparse. The strip now runs as each continuation lands, while the logical line is still empty, so the bytes it takes are recorded at offset 0 in wire order and the card still round-trips.

The second is that `VcardWire::prepend` merges the tokeniser's pieces with the line splitter's by offset with a stable sort rather than concatenating them. A value ending on two `=` gives the tokeniser a soft break past the last logical byte and the splitter a dangling `=` before it, and concatenating alone emitted the soft break first, which made the reparsed line swallow the line after it. That input is one of the fuzz regressions the cst tests replay, so it failed the moment the shape went in.

Blank lines *after* the last line had nowhere to live, so `VcardCst` gained a `trailing` field, set only when nothing but whitespace follows the card, and the card-level `trim_leading_eol` is gone, since the tokeniser records what it skips. `parse_many` no longer trims between cards either: a blank line between two cards belongs to the card that follows it, so concatenating what the iterator yields reproduces the file.

The proof moved with the code. The four corpus sweeps assert byte identity outright instead of a fixpoint, through a shared harness that reads a whole file rather than its first card. The merge generator's noise block gained a blank line and a QUOTED-PRINTABLE soft break next to the fold it already carried, so the law that an untouched line keeps its bytes now has the whole wire vocabulary to bite on. The merge test that pinned a folded card being rewritten unfolded, the one fixture that was lossy only because of folding, asserts the fold survives instead. The parse fuzz target gained a third oracle: a file whose every card parses comes back byte for byte.

The merge itself needed no change. It clones the left card's lines, wire shape included, and replays the right side's actions through the byte-preserving edit layer, so a line nobody touched keeps its folds and an edited line drops them by the length guard.

Spec updated: `parsing` (MODIFIED: round-trip fidelity, line normalisation, envelope-free and multi-card input).
