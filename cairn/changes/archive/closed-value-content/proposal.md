---
cairn: change
id: closed-value-content
status: landed
created: 2026-08-31
---

# Validation reaches the content a definition closes

## Why

`validate` checks a property's shape and stops there: the versions that define it, how often it may appear, the value types it takes and the parameters it carries. Nothing looks at what is *in* the value.

So `GENDER:X` passes. RFC 6350 section 6.2.7 closes that component:

```abnf
GENDER-value = sex [";" text]
sex          = "" / "M" / "F" / "O" / "N" / "U"
```

There is no `iana-token` and no `x-name` in that alternation, which is what makes it different from every other vocabulary in the format. `X` is simply not a value the property has, and a card carrying it is not conformant. The escape hatch the RFC intends is the second component, free text: `GENDER:O;intersex`, `GENDER:;it's complicated`.

A consumer that asks whether a card conforms and is told yes has been told something untrue. Cardamum asks exactly that before it writes a card someone composed, so the gap surfaces as a card that reaches a server and is rejected there, or worse is not.

## What

**`VcardPropSpec::invalid_value`**, a fifth member of the per-property contract and a fifth entry in the vtable. The four it joins describe a property's shape; this one describes the content of its value, and defaults to allowing everything so only the few properties whose RFC closes that content override it.

Three do:

- `GENDER`, whose sex component is the closed set above (RFC 6350 6.2.7).
- `PROFILE`, whose value "MUST be the case-insensitive string `VCARD`" (RFC 2426 3.6.3).
- `CLIENTPIDMAP`, whose first field is a positive integer (RFC 6350 6.7.7).

**Three parameters** carry the same kind of constraint, and theirs does not vary by property, so one check covers each wherever it appears:

- `PREF`, "an integer between 1 and 100" (RFC 6350 5.3).
- `PID`, one or more digits optionally followed by a dot and more digits (RFC 6350 5.5).
- `DERIVED`, "true" or "false" (RFC 9554 3.4).

**Two error variants**, `Value` and `ParamValue`, each naming what was found. Adding them is breaking for anyone matching the enum exhaustively.

## What this is not

Not format validation. A date, a URI, a UTC offset and a language tag all have grammars, and checking those is a different appetite with a much fuzzier edge: the crate would end up owning a URI parser to answer a question nobody asked. What lands here is closed vocabularies and small integers, which are unambiguous and cheap.

Not a tightening of the read path. Parsing stays maximally liberal, `GENDER:X` still round-trips byte for byte, and the sex component stays a `Cow<str>` rather than becoming an enum. Strictness lives in `validate`, which is where a caller goes to ask.

Not the open vocabularies. `KIND`, `CLASS`, `GRAMGENDER`, `CALSCALE`, `PHONETIC` and every `TYPE` set end their grammars in `iana-token / x-name`. An open set is nothing to validate against, and treating one as closed would reject cards that conform.
