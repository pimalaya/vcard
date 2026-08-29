---
cairn: change
id: merged-lines-are-terminated
status: landed
created: 2026-08-29
---

# A line that stops being last is terminated

## Why

`VcardCst::parse` accepts a bare RFC 2425 record with no `BEGIN` / `END` envelope, and a file read without a trailing newline leaves its last line with an empty ending. That is right while the line is last. The merge then writes additions after it without terminating it, and a line serializes as its name, parameters, value and ending with nothing in between, so the added line lands inside the previous line's value:

    base   "FN:a\r\nNOTE:b"
    right  "FN:a\r\nNOTE:b\r\nTEL:+1"

    merged "FN:a\r\nNOTE:bTEL:+1"

Reparsing gives two properties, `NOTE` holding `bTEL:+1`. The addition is destroyed and nothing is reported. The same gluing can make the merged card fail to reparse at all, which breaks the module's promise that a caller can serialize the merged card and read it back.

An added line carries the same hazard from the other end: a right card whose own last line is unterminated hands the merge an unterminated addition, which then glues onto whatever follows it.

## What

After the deferred removals and additions have run, every line of the merged card but its last gets a line ending if it has none. A card that was already terminated is untouched, and a card whose unterminated line is still last is untouched, so nothing gains a byte it does not need.

## Judgement call, for review

**The byte-preservation law is narrowed, deliberately.** It said a line all three copies carry keeps its serialized bytes, ending included. A line that stops being last cannot keep an empty ending and still be a line, so the law now allows exactly that one gain: a line whose base spelling carries no ending may appear in the merged card with the default `\r\n` appended. Nothing else about the line may change. The alternative, placing every addition before a trailing unterminated line, keeps the bytes but reorders the card for a reason that has nothing to do with the addition, and still has to terminate the addition itself.
