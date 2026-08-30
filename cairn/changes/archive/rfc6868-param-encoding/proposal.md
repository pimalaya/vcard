---
cairn: change
id: rfc6868-param-encoding
status: landed
created: 2026-08-30
---

# RFC 6868 parameter value encoding

## Why

A parameter value is decoded with the RFC 6350 section 3.4 *text* escapes, `\\` `\,` `\;` `\n`. Section 3.3 gives a parameter no backslash escapes at all, which is the whole reason RFC 6868 exists: a parameter that must carry a double quote, a newline or a caret encodes it as `^'`, `^n` or `^^`.

So the crate does the wrong thing twice over. A backslash a parameter legitimately carries, a Windows path in an `X-` parameter or a `LABEL`, is eaten on the way in and cannot be written back. A real `^n` or `^'` from any RFC 6868 producer is handed to the caller raw, so a `LABEL` reading `Mr. Public, "Bob"` arrives with its encoding showing.

ical-rs has the identical defect against RFC 5545 section 3.2, and it is fixed under the same change id. The two crates are deliberate twins, so the fix belongs in both.

## What

Decode a parameter value by the RFC 6868 rules and encode it back by them, leaving the text escapes to text values where they belong.

The decoding is: `^n` becomes a newline, `^^` a caret, `^'` a double quote, and a caret before anything else stays a caret with the character after it (RFC 6868 section 3.1, which forbids inventing an error there). Encoding is the inverse, applied only where the parameter is not already quoted around the character.

RFC 6868 updates RFC 6350 and nothing earlier, so the rules apply to vCard 4.0 alone: a 2.1 or 3.0 caret is a literal caret, and decoding it would corrupt the value. `VcardEscaper` is the seam, which means splitting its `Modern` variant into `V3_0` and `V4_0`, the two versions it stood for, and giving a parameter node an escaper the way a value node has one.

Postel governs the transition: a value carrying no caret and no backslash means the same under both readings, which is nearly every parameter in the corpus, so the change must be invisible for those. What moves is a value carrying a backslash, which stops being unescaped, and a 4.0 value carrying a caret, which starts being decoded.

Done when a parameter round trips byte for byte through decode and encode, when the three RFC 6868 sequences decode and re-encode, when a lone caret survives untouched, when a 3.0 caret is left alone, and when the golden corpus is unchanged except where a fixture genuinely carries one of these. The byte-for-byte round trip holds for the canonical spelling: RFC 6868 decoding is deliberately not injective, `^x` and `^^x` both reading as `^x`, and the encoder writes the canonical `^^x` for both.

## Blast radius

The decoded parameter model, jCard and JSContact (which carry parameters), the merge, and the corpus comparisons. Worth doing before the release rather than after, since it changes what every decoded parameter says.
