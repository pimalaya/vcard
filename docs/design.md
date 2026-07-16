# Design

This note records the reasoning behind the architecture summarized in the src/lib.rs header, and the alternatives that were weighed and rejected. The header is the entry point; read it first.

## One version-agnostic model

vcard-rs is a single model that reads and writes vCard 2.1, 3.0 (RFC 2426) and 4.0 (RFC 6350) alike. The card version is a decoded indicator, never a type parameter or a separate dialect: the syntax tree ignores it entirely, and only the codec and the per-property spec branch on it, and only where escaping or a value's shape genuinely differ. A generic root type parameterized by version was rejected: the version is a value, not a type, and threading it through every type buys nothing the runtime indicator does not.

## The two layers

The decoded model (the vcard, version, prop, param and value modules) is pure data with no dependency on the syntax side, so a consumer that only needs the model can depend on it alone, with the parser feature off. Property names, parameter names and value types are closed identity enums whose wire spelling is reached through FromStr and Deref; a property is a struct of a name, parameters and one value, its parameters and value being open payload enums with an Unknown arm, so anything outside the model survives.

The syntax tree (the tree module, gated behind the parser feature, on by default) is everything byte-faithful. Its hub is a concrete syntax tree of generic nodes reproducing the wire bytes exactly. Parsing reads one card or a bare RFC 2425 record; a lazy iterator walks a multi-card file. Decoding projects the tree onto the model; encoding projects the model back to a canonical tree. Per-property lens markers read or edit a single line through byte-preserving cursors, so editing one property leaves every other byte intact.

## Postel's law

The library is liberal in what it accepts and strict in what it sends. Parsing is maximally liberal: any real card, including properties, parameters and value types no version officially defines, is accepted and round-trips byte for byte. The decoded model keeps that openness through the Unknown arms. Strictness lives only on the way out, as two runtime steps: the builder, which refuses to construct a property the spec forbids, and validate, which checks a decoded card against its version's RFC contract.

Validity and lossiness are orthogonal, which is why validity is a runtime predicate rather than a second, stricter type. A conformant card may still carry X- or IANA extensions, so a no-Unknown "strict" type would mean "no extensions", a useless category. There is one lossy property type; a card that passes validation earns a Valid proof marker that only validation can mint, and both the model and its proof convert back into the syntax tree.

## The spec layer

Each property carries a spec on its lens marker: the versions it lives in, its cardinality, the value types and parameters it may take per version, and the value type in force given a declared VALUE. A single vtable dispatch bridges the open property-kind enum back to those static specs, so the decoder consults it to pick a value kind, validation consults it to check conformance, and the builder consults it to reject illegal construction: one source of truth, three readers.

## Bytes, not transcoding

A property value is held as raw bytes, so a value in a foreign charset (a vCard 2.1 CHARSET) survives byte for byte; a name or parameter must be UTF-8, as every version's grammar guarantees, and a non-UTF-8 name or parameter is a hard parse error. The byte-faithful serializer is therefore the bytes form, while Display is a convenience that is lossy only for a non-UTF-8 value.

The core transforms no content: a QUOTED-PRINTABLE or BASE64 transfer encoding and a CHARSET are surfaced raw, with their parameters kept, so nothing is silently lost or transcoded; only the value grammar (escapes and line folding) is resolved, because that is parsing, not content decoding. Decoding the content is opt-in, one small no_std crate per feature (quoted-printable, base64, encoding). An earlier design where the core quoted-printable-decoded values was reverted: it ran a lossy UTF-8 conversion that silently destroyed foreign-charset bytes.

## jCard and JSContact

jCard (RFC 7095) and JSContact (RFC 9553, mapped by RFC 9555) both sit on the decoded model, behind features, and both cross the boundary as a raw serde_json value rather than through serde derives on any vcard type: one type can have two JSON spellings (jCard versus JSContact), so serde, which keys one representation per type, is the wrong tool, and a raw-value boundary keeps the public API free of a serialization commitment. jCard was built first because RFC 9555 encodes its vCardProps and vCardParams escape hatches in jCard syntax, so the JSContact conversion reuses the jCard codec for anything it cannot map first-class. The RFC 9554 vocabulary (newer properties and the extended address components) is modeled first-class so the conversion maps it directly instead of dropping it into the escape hatch.

## Rejected designs

A root generic parameterized by version. A second, stricter data model splitting a property into strict and lossy variants: validity and lossiness are orthogonal, so this splits the wrong axis. A generic lossy wrapper across prop, param and value: it fits only the property name, since value and param are payload unions whose Unknown already carries structured raw data. Modeling the 2.1 inline AGENT recursively: recursion is a denial-of-service risk, so AGENT stays raw text with an opt-in single-level re-parse helper.
