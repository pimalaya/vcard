---
cairn: change
id: the-card-written-straight
status: landed
created: 2026-11-02
---

# The card written straight

The second speed pass over the parser, the decoded model and the serializers, taken from the handoff [the-structure-scanned-once](../../the-structure-scanned-once/proposal.md) on 2026-11-02 and landed on 2026-11-03. The handoff stays active, rewritten at the end of this pass for the next one.

## Why

The first pass ([the-lines-built-in-place](../the-lines-built-in-place/proposal.md)) left `to_string` of a decoded card flat: it encoded the whole card into a syntax tree, a leaf allocation per value, then printed that tree through the formatting machinery, checking every leaf for UTF-8 again. Over the test corpus it cost more than parsing and decoding together. Decoding still checked UTF-8 and searched for a backslash once per comma-separated piece, and copied a 432-byte value and a 504-byte property for every line, the address's eighteen lists sitting inline in every value.

## What

No change to what any reader or writer returns, with two exceptions the user approved on 2026-11-03 (the handoff's levers 5 and 6):

- `Display` for a decoded card writes its text straight from the model. Each value type states its wire shape once, in an `emit` that both its `VcardCodec::encode` (a node) and the text writer read, so the two cannot drift apart. The value escapes are found sixteen bytes at a time.
- A value is checked for UTF-8 and searched for a backslash once, then split and unescaped as text; a value with no backslash is cut at every separator.
- The parser pushes a constant placeholder line and overwrites its fields without dropping them; reads each fold continuation once; and `to_bytes` lays a folded value out straight from its node, copying it once.
- `VcardValue::Adr` holds a `Box<VcardAdr>` (an API break, in the already breaking 0.6), so a value is 120 bytes and a property 192.
- ASCII bytes become text through `str::from_utf8_unchecked` after `<[u8]>::is_ascii`, the crate's first `unsafe`, with `str::from_utf8` for anything else.

The log, cairn/log/2026-11-03-the-card-written-straight.md, gives every change with its cycles and the ones tried and dropped.
