---
cairn: change
id: param-quotes-are-delimiters
status: landed
created: 2026-08-30
---

# A parameter's double quotes are delimiters, not content

## Why

RFC 6350 section 3.3 gives `param-value = *SAFE-CHAR / DQUOTE *QSAFE-CHAR DQUOTE`. The DQUOTE pair is the production's own delimiter: `QSAFE-CHAR` is every character but a control and a double quote, so a quote can never be part of what the pair encloses.

The decoded model keeps that pair today, on purpose (`decoded-model.md`, "A parameter value is encoded by RFC 6868"). That decision is wrong, and it is wrong at the boundary a consumer reads:

- `line.param::<GEO>()` on the RFC's own section 6.3.1 address hands back `"geo:37.386,-122.083"`, quotes included, which no URI parser accepts. `TZ`, `LABEL`, `SORT-AS`, `MEDIATYPE` and `ALTID` all carry a quoted form in the wild and read back the same way.
- The jCard and JSContact exports write that string into JSON, where RFC 7095 and RFC 9553 have no vCard quoting: the quotes become content of the JSON string.
- The merge compares parameters through the same decoder, so `TZ="America/New_York"` and `TZ=America/New_York` are two different values, and a card whose server re-quotes a parameter reports a change nobody made.
- RFC 6868 `^'` becomes unreachable: a value holding a literal double quote decodes to the same text as a value the wire quoted.

It is also the one place the crate mixes syntax into the decoded model. A value node does not keep its `\,` escapes, a folded line does not keep its folds; both are recorded on the syntax side, which is where the quotes already are.

## What

`unescape_param` strips a balanced surrounding DQUOTE pair before resolving the RFC 6868 carets, and `escape_param` wraps its result in one when the escaped text carries a `,`, a `;` or a `:`, the three delimiters RFC 6350 keeps out of a bare `SAFE-CHAR` run. A fourth, the double quote itself, cannot occur: in 4.0 the caret encoding has already spelled it `^'`.

Quoting is a vCard 3.0 and 4.0 rule (RFC 2425 section 5.1 defines the `quoted-string` that RFC 2426 inherits), so `VcardEscaper` grows `has_param_quoting`, false for 2.1 alone, whose grammar has no quoting and whose double quote is therefore content.

Byte fidelity is untouched: the quotes live on the syntax leaf, which parsing and serialization never read through the codec. Only the canonical `decode` and `encode` projections change, and they stay lossless, a value that needs quoting being quoted again.

## Judgement call, for review

**A 3.0 value carrying both a double quote and a delimiter has no conformant spelling.** RFC 2426 has no RFC 6868, so a literal quote cannot be encoded, and quoting a value that already holds one produces a pair that closes early. Such a value is written quoted anyway rather than dropped or mangled, and the case is noted where the rule is applied. In 4.0 it cannot arise.

**This is a breaking change for a caller that builds parameters by hand.** A `VcardParam::Geo(Cow::Borrowed("\"geo:...\""))` written with its own quotes now means a value whose text starts and ends with a quote, and goes out as `GEO="^'geo:...^'"`. The value is not lost and round-trips, but the card is not the one the caller meant, so it lands in the CHANGELOG as a breaking change.
