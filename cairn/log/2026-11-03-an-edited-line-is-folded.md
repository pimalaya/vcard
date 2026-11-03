---
cairn: log
change: an-edited-line-is-folded
landed: 2026-11-03
---

# An edited line is folded

Landed the fold of an edited or built line. Every cursor edit (the value cursor, the list cursor and the typed property cursors of ADR, N, GENDER, CLIENTPIDMAP) goes through `VcardLine::edit_value`, which compares the value before and after and cuts the wire shape at the first byte that changed (`VcardWire::cut`): the pieces before it stay, the tail is refolded at 75 octets from the last kept fold with that fold's whitespace and break. A line with no recorded shape, built or encoded from the model, is folded whole, as is one whose recorded length went stale through a direct assignment to its public fields. The fold never cuts a UTF-8 sequence (`refold`, after ical-rs's `fold`). A 2.1 line and a `QUOTED-PRINTABLE` line are never refolded, a line pushed raw and a parsed line nothing edited keep their shape. A decoded card's `Display` folds through the same `refold` (`fold_text`), and a `VcardLine`'s `Display` keeps its fast path only for a line that cannot need a fold, so `to_string` and `to_bytes` agree.

It also fixes an edit keeping a line's length that moved a multi-octet character across an old fold: the old fold was re-applied inside the character, writing invalid UTF-8 (RFC 6350 3.2 says such a character MUST stay contiguous). A first version also dropped a parsed shape whose fold sat before a byte read as a UTF-8 continuation, to catch that case for edits bypassing the cursors; the parse fuzz target found it breaking the round trip of a foreign-charset value folded before a raw `0x85`, and it was removed. A same-length edit made by assigning the public fields directly still re-applies the old folds.

tests/folding.rs covers the delta's scenario, an edit in the first physical line, a shrinking edit, a list item edit, a built line, a two-octet character at the cut, a same-length edit across a fold, a direct assignment, a bare LF line, a 2.1 card, a `QUOTED-PRINTABLE` line, an unedited long line, an unedited fold before a continuation byte, and a decoded card whose text matches its encoding. The behaviour tests failed on the old code. Four older tests pinning the unfolded text of a long encoded or edited line now compare it with the folds removed.

Verification: the suite under default, all and no features; clippy `--all-targets` under all and no features with `-D warnings`; docs with `-Dwarnings`; the bare build depends on nothing; the merge fuzz target three times for 180 to 240 seconds (19,414 to 28,871 runs each) and the parse target for 240 and 180 seconds after the fix (372,686 and 276,957 runs), no crash. Speed not measured: an unedited line pays one more flag check and a length compare.

Downstream, checked on 2026-11-03 (every consumer is on crates.io 0.5, so nothing moves until it bumps): no consumer loops on re-serialized bytes. The connectors rebuilding a card from a JSON model (io-gpeople, io-msgraph, io-jmap through neverest and the Android app) write new bytes for an unchanged card with a long line once after the bump, which the pimdir byte-hash comparison reads as a source change and pushes once. io-gpeople 0.5.0 and io-msgraph (rev 2ae2637) keep unmapped lines as `line.to_string()` in a stash joined and split on `\n`: a folded line splits apart on its next read, an existing bug that now fires for every edited or built long line, to fix there before they bump. tcard's spec (template.md: a moved line is written unfolded) and two of its doc comments, and the Android golden files (contact-form.txt, mapping-corpus.txt over a 121-octet NOTE), need updating with the bump.

Capabilities moved: parsing (ADDED: an edited or built line is folded; MODIFIED: line normalisation keeps the folds before an edit).
