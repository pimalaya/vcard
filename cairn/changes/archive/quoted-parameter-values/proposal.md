---
cairn: change
id: quoted-parameter-values
status: landed
created: 2026-08-29
---

# A quoted parameter value may carry a colon and a semicolon

## Why

RFC 6350 section 3.3 gives `param-value = *SAFE-CHAR / DQUOTE *QSAFE-CHAR DQUOTE`, and `QSAFE-CHAR` is any character but a control and a double quote, so `:` and `;` are both legal inside a quoted parameter value. Two places in the line splitter ignore that: the value separator is found with the first `:` anywhere in the line, and the head is split on every `;`.

The RFC's own section 6.3.1 example is the casualty:

    ADR;GEO="geo:12.3457,78.910";LABEL="Mr. John Q. Public, Esq. ...":;;123 Main Street;...

parses to one parameter `GEO` holding `"geo`, no `LABEL` at all, and an address whose components are all shifted by one. tests/corpus/rfc/rfc6350_adr_params.vcf carries exactly that line, and tests/corpus/mixerp/complex_4.0.vcf breaks the same way on `TZ="05:45"`. Round-tripping hides it, since every piece is written back verbatim.

## What

Both scans become quote aware: the value separator is the first `:` outside a double-quoted run, and the head splits on the first `;` outside one. The quoted-printable head probe uses the same separator, so a quoted colon no longer moves it either.

## Judgement call, for review

**An unbalanced quote falls back to the quote-blind scan.** Naive quote tracking would let one stray `"` swallow the rest of a junk line and turn a parseable line into `MissingPropertyColon`. When no colon sits outside quotes, the parser takes the first colon anywhere instead, which keeps the liberal parse the crate promises. An unbalanced quote in the head does leave the remaining parameters as one parameter node, which round-trips verbatim.
