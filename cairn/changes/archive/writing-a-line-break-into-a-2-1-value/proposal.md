---
cairn: change
id: writing-a-line-break-into-a-2-1-value
status: landed
created: 2026-08-29
---

# A line break written into a vCard 2.1 value destroys the card

## Why

Found by the merge fuzz target, which produced a card the merge itself could no longer parse back: `MissingPropertyColon("Baytown, LA 30314")`.

The vCard 2.1 escaper escapes `;` and nothing else, so a value carrying a line break is written raw. A raw break ends the content line, so the rest of the value becomes a line of its own with no `:` in it and the card no longer parses:

    set_at(0, ["x\ny"]) on a 2.1 card  ->  NOTE:x
                                           y
    parse                              ->  MissingPropertyColon("y")

That is reachable from the plain edit layer on any 2.1 card, and it became reachable from the merge as soon as a value replayed from a later version was transcoded into a 2.1 one, since the modern reader resolves `\n` into a real line break.

## What

The 2.1 escaper writes a line break as `\n`, the spelling every 2.1 exporter uses, alongside the `;` escape it already had.

## Judgement call, for review

**The 2.1 escaper is deliberately not the inverse of the 2.1 reader.** vCard 2.1 defines no line-break escape, so this crate reads a 2.1 `\n` as a literal backslash and an `n`, which two unit tests pin on purpose. That reading is left alone: changing it would move how every existing 2.1 card decodes, through jCard and JSContact with it, which is a separate question. The writer cannot be left alone, because its only alternative spelling is a card that does not parse. A value the wire cannot hold exactly is worse held imprecisely than not held at all.
