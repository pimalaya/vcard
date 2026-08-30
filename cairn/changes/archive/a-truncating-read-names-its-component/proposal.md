---
cairn: change
id: a-truncating-read-names-its-component
status: landed
created: 2026-08-30
---

# A truncating read names the component it truncates at

## Why

The value node offered `decode_at(i)`, `decode_scalar_at(i)` and `decode_joined_at(i)`, and almost every caller passed `0`. Reading component zero looks like reading the value, and is not: it stops at the first unescaped `;`, and the scalar form stops again at the first unescaped `,`. The shape invites the mistake, and the mistake has been made four times in two days across three crates:

- a URI decoded to its media type, dropping a `data:` payload,
- a merge comparing decoded values, so an edit past the first `;` was lost,
- a merge action whose `old` and `new` were both truncated and therefore equal,
- a contested value rendered cut off in the document a person decides from.

Each was fixed where it was found. The API that produces them was left standing, with roughly fifteen more `..._at(0)` call sites in each of the two crates.

## What

The whole-value read becomes the short, unqualified one, and every truncating read names the component it truncates at:

- `decode()`, `decode_list()` and `decode_bytes()` read the whole value, its `;` (and, for the first and last, its `,`) kept literal.
- `decode_component(i)` and `decode_component_list(i)` read one `;`-component, an index always spelled out.
- `decode_scalar_at` and `decode_bytes_at` are gone. Both cut twice, at a `;` and then at a `,`, and no caller wanted the second cut.

The writers follow the readers so a read and a write back are inverse: `set` and `set_bytes` replace the whole value, `set_component` and `set_component_bytes` name their slot.

Every call site was then reviewed one at a time and became the whole-value read, unless the value is one the specification structures with `;`, where it became an explicitly indexed read.

## Judgement calls, for review

**The un-indexed setters replace the whole value.** Keeping them on component zero while their readers read the whole value would break the identity round trip: `text()` on `NOTE:a;b` reading `a;b` and `set_text("a;b")` writing only component zero leaves `NOTE:a\;b;b`. A reader and a writer at different scopes is a data-loss generator, so they were moved to the same scope. `set_component` still rewrites nothing but the component it names, which is what byte preservation rests on.

**A single-valued component of a structured value reads joined, not scalar.** `CLIENTPIDMAP`'s URI, `GENDER`'s identity and every `ORG` unit are a URI or a text, where a comma separates nothing. The deleted `decode_scalar_at` was cutting them there.

**`GEO` reads by component in both its shapes.** vCard 2.1 packs the coordinate pair into one component with a comma, 3.0 into two components with a semicolon. Both are the structured value's own shape, so both name their component rather than reading the value whole.
