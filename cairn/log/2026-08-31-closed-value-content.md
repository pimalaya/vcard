---
cairn: log
change: closed-value-content
landed: 2026-08-31
---

# Validation reaches the content a definition closes

`validate` checked a property's shape and stopped there, so `GENDER:X` passed. RFC 6350 section 6.2.7 closes that component with no `iana-token` and no `x-name` in the alternation, which is what makes it different from every other vocabulary in the format:

```abnf
sex = "" / "M" / "F" / "O" / "N" / "U"
```

A consumer asking whether a card conforms was told yes about a card that does not. Cardamum asks exactly that before writing a composed card, which is where this surfaced.

**`VcardPropSpec::invalid_value`** (prop/spec.rs) is a fifth member of the per-property contract and a fifth entry in the vtable. The four it joins describe a property's shape; this one describes the content of its value, and defaults to allowing everything, so the file reads as "these three properties close their content and no others do".

Three override it: `GENDER` on its sex component, `PROFILE` on its single value, "the case-insensitive string `VCARD`" (RFC 2426 3.6.3), and `CLIENTPIDMAP` on its identifier, "a small integer" (RFC 6350 6.7.7). Matching is case-insensitive throughout, RFC 5234 making a quoted ABNF literal so, which is why `GENDER:m` passes.

**Three parameters** carry the same kind of constraint and theirs does not vary by property, so one check in the validator covers each wherever it appears: `PREF` from 1 to 100 (5.3), `PID` as `1*DIGIT ["." 1*DIGIT]` (5.5), `DERIVED` as `true` or `false` (RFC 9554 3.4).

**Two error variants**, `Value` and `ParamValue`, each naming what was found. Adding them breaks anyone matching `VcardValidateError` exhaustively.

**What was deliberately left out**, and the reason it is worth writing down: every other vocabulary in the format is open. `KIND`, `CLASS`, `GRAMGENDER`, `CALSCALE`, `PHONETIC` and every `TYPE` set end their grammars in `iana-token / x-name`, so a value outside the listed ones still conforms and checking it would reject good cards. A test pins that, using `KIND` with an `x-` value.

Format validation is out too. Dates, URIs, UTC offsets and language tags have grammars, and the crate would end up owning a URI parser to answer a question no caller asked. What landed is closed vocabularies and small integers, which are unambiguous and cheap.

Reading is untouched. `GENDER:X` still parses, still round-trips byte for byte, and the sex component stays a `Cow<str>` rather than becoming an enum. Strictness lives in `validate`, which is where a caller goes to ask, and this is the crate's Postel posture rather than an exception to it.

Verified: 207 unit tests and the corpus, calcard and coverage suites green, nine of them new over each accepted and rejected spelling, the case-insensitivity and the open-vocabulary case included. Clippy clean, and the bare core still builds dependency-free with no std leak.

Spec updated: `conformance` (ADDED: "A closed value vocabulary is validated", "A constrained parameter value is validated", "Content validation is not format validation").
