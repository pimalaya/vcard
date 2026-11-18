---
cairn: log
change: client-cards-round-trip
landed: 2026-11-18
---

# Client cards round-trip

Landed [client-cards-round-trip](../changes/archive/client-cards-round-trip/proposal.md), asked for by Pimalaya Server, which stores contacts as vCard and serves them as JSContact.

What already held: the RFC 9555 escape hatches (`vCardProps`, `vCardParams`, `JSPROP` with `JSPTR`) carried every unknown property and parameter both ways, and an in-crate test held a Card to a fixpoint through vCard. What was missing: any conversion between vCard 3.0 and 4.0, and a check of the vCard side line for line on cards as clients write them.

`Vcard::to_version(&self, version)` (src/vcard.rs) rewrites a decoded card between 3.0 and 4.0 on the model alone, so it needs no parser. Up to 4.0: `TYPE=pref` is `PREF=1`; `PHOTO`, `LOGO`, `SOUND`, `KEY` with `ENCODING=b` become a `data:` URI typed from their `TYPE` (`image/`, `audio/`, `application/`), a referenced one drops `VALUE=uri`; an `AGENT` URI becomes `RELATED;TYPE=agent`, an inline agent card is dropped (RFC 6350 removed `AGENT`); `X-ADDRESSBOOKSERVER-KIND` and `-MEMBER` become `KIND` and `MEMBER`. Down to 3.0 runs each rule backwards, `PREF=1` folding into the first `TYPE` list, any other `PREF` kept. Groups (`item1.` with `X-ABLabel`) and all other lines and parameters are kept as they are. Other version pairs change the version only.

Five fixtures under tests/corpus/clients, in each client's shape: an iOS contact and an iOS group (3.0), a Google contact (3.0), an Outlook contact (3.0, `X-MS-*`, `AGENT`), a Thunderbird contact (4.0). tests/jscontact_clients.rs takes each up to 4.0, to a Card, back to vCard and down to its version, and holds the result line for line to the fixture once normalized, then the Card to a second pass. It found two losses, both fixed:

- `GENDER:M` came back `GENDER:M;`: the writer always wrote the identity component. `VcardGender::emit` (src/tree/value/gender.rs) now writes it only when set. This touches every written `GENDER`, not only JSContact.
- `TEL;VALUE=URI:tel:...` came back as a text `TEL`: the export drops `VALUE` as reflected by the value kind, and the import made every phone a text. A `tel:` number now imports as `TEL;VALUE=uri` with a URI value (`simple` in src/jscontact/import.rs).

Normalized by the check, and stated in the json-codecs spec: line and parameter order, `TYPE` shape and case, `VALUE` case and a `VALUE` of the default type, dates and timestamps spelled basic (`1990-04-12` becomes `19900412`, as 4.0 writes them), Google's non-RFC `\:` escape read as `:`, `CATEGORIES` order, folds and name case.

Tests: tests/jscontact_clients.rs, and tests/version.rs (the iOS group and contact up and back byte for byte, an inline agent dropped, a `PREF=2` kept going down, a `GENDER` with no identity). The fixture sweep failed on `GENDER` and `TEL` before their fixes. Nothing was timed; the parse loop is untouched.

Left out: the 3.0 `LABEL` property is kept as written rather than moved onto its `ADR` as a 4.0 `LABEL` parameter; `SORT-STRING`, `NAME`, `MAILER`, `CLASS` are kept as written in 4.0; dates are not rewritten between extended and basic; Apple's `X-APPLE-OMIT-YEAR` birthdays are not mapped to `--MMDD`; 2.1 cards go up by the 3.0 rules and nothing goes down to 2.1.

Capabilities moved: decoded-model, json-codecs.
