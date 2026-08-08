# calcard vCard corpus

These fixtures are the vCard test cards from Stalwart `calcard` (https://github.com/stalwartlabs/calcard), resources/vcard, licensed `Apache-2.0 OR MIT` (compatible with this crate's `MIT OR Apache-2.0`).

They are real-world 2.1 / 3.0 / 4.0 cards plus the RFC 2426 / 6350 examples, used here only as a parser robustness corpus (parse, serialize to a fixpoint, decode without panicking), not as golden output.

Transformation on import: calcard ships some files holding several concatenated cards; each card was split into its own <file>_<index>.vcf so the corpus is one card per file, matching the ez-vcard corpus. Cards with no recognised `VERSION` were dropped. Every other card is kept verbatim, including 004_0.vcf (`VERSION` not the line after `BEGIN`) and 065_0.vcf (a 3.0 card using 2.1-style `QUOTED-PRINTABLE` soft line breaks): the liberal parser accepts both.
