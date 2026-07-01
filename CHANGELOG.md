# Changelog

All notable changes to this project are documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

## [Unreleased]

### Added

- Added the closed kind vocabularies (VcardPropKind, VcardParamKind, VcardValueKind, VcardVersion), each with FromStr and Deref<str> for its wire name, plus VcardValue::kind and VcardParam::kind.
- Added VcardPropName (a known VcardPropKind or a verbatim unknown name) and switched VcardProp::name to it; an unrecognised or missing card version normalises to VcardVersion::V4_0 in the decoded model (byte-faithful round-tripping stays on the syntax tree).
- Added the per-property VcardPropSpec contract (allowed_versions, cardinality, allowed_values, allowed_params and the in-force value) on the lens markers, with VcardPropCardinality, filled per RFC 6350 (including version-forked FN/N cardinality), plus VcardPropKind::ALL.
- Added Vcard::validate, an RFC 6350 conformance check over the decoded model (per-version property existence, value kind, version-aware parameters, and cardinality including required-but-absent) that permits extensions; Valid<T>, a marker only validation can mint (TryFrom<Vcard> for Valid<Vcard>); and From conversions Vcard -> VcardCst and Valid<Vcard> -> VcardCst.
- Added VcardPropBuilder, a version-aware, spec-driven builder for strict construction: it pins the property name and reuses the per-property validation, rejecting (via Result) a disallowed value kind or known parameter while allowing extension parameters.
- Added the version-agnostic decoded model (parser-free, always available): the Vcard aggregate (a version plus a list of VcardProp), VcardProp (name, parameters and one value), the open VcardParam and VcardValue payload enums (each with an Unknown arm), and the structured value types VcardN, VcardAdr, VcardGender, VcardOrg, VcardGeo, VcardClientPidMap, VcardBinary, VcardDateAndOrTime, VcardTimestamp, VcardUtcOffset, VcardText/VcardTextList, VcardUri and VcardLanguageTag.
- Added the byte-faithful syntax tree (parser feature, on by default): VcardCst parses bytes or text into a tree that reproduces the wire exactly, decodes onto the model, encodes back to a canonical tree, and edits one property in place through per-property lenses and byte-preserving cursors; to_bytes is the byte-faithful serializer, while Display / to_string is a lossy-for-non-UTF-8 convenience.
- Added raw-byte value handling: a property value is kept as bytes so a value in a foreign CHARSET survives byte for byte, while a name or parameter must be UTF-8 (a non-UTF-8 name or parameter is a parse error), plus a byte hatch (VcardValueCursor::bytes / set_bytes).
- Added multi-card and bare-record parsing: VcardCst::parse_many iterates every card in a file, and VcardCst::parse also accepts a bare RFC 2425 directory record with no BEGIN/END envelope.
- Added VcardCst::agent, which re-parses the vCard embedded in an AGENT property (opting into exactly one level, never recursively).
- Added opt-in content-decoding features, each backed by a no_std crate: quoted-printable (=XX octets, VcardValueCursor::quoted_printable), base64 (inline binary values, VcardBinary::decode_base64), and encoding (foreign CHARSET transcoding via encoding_rs, VcardValueCursor::charset).
