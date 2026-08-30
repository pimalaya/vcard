---
cairn: log
change: a-truncating-read-names-its-component
landed: 2026-08-30
---

# A truncating read names the component it truncates at

The value node read one `;`-component through `decode_at`, `decode_scalar_at` and `decode_joined_at`, and almost every caller passed `0`. Component zero looks like the value and is not: the read stops at the first unescaped `;`, and the scalar form stops again at the first unescaped `,`. Four defects in two days across three crates came out of that one shape, each fixed where it was found while the shape that produces them stayed, with fifteen more `..._at(0)` call sites behind it.

The readers now say what they cut. `decode`, `decode_list` and `decode_bytes` read the whole value with its separators literal; `decode_component` and `decode_component_list` read one slot and always spell out which. `decode_scalar_at` and `decode_bytes_at` are gone: both cut twice, and no honest caller wanted the second cut. The writers moved with them, `set` and `set_bytes` replacing the whole value against `set_component` and `set_component_bytes` naming their slot, because a reader and a writer at different scopes turn a read-modify-write into data loss.

Every call site was then reviewed one at a time. Most became the whole-value read, which fixed what they were quietly dropping: a note or a language tag or a timestamp cut at a `;` it was supposed to escape, a text list losing everything past one, a `CLIENTPIDMAP` URI and a `GENDER` identity and every `ORG` unit cut at a comma that separates nothing. The `;`-structured kinds (`N`, `ADR`, `GENDER`, `ORG`, `GEO`, `CLIENTPIDMAP`) kept their component reads, now written as the deliberate act they are.

The generic cursor moved with the node, so `text`, `bytes` and `list` read the value rather than its first slot, and their setters replace it rather than leaving a tail behind. One test asserted the truncated reading of `NOTE:a,b` and was corrected rather than kept.

Spec updated: `decoded-model` (MODIFIED: a value with no `;`-structure decodes whole), `editing` (ADDED: a truncating read names its component; MODIFIED: edits are byte-preserving, now distinguishing the two setter scopes).
