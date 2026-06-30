# Changelog

All notable changes to this project are documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

## [Unreleased]

### Added

- Added the closed kind vocabularies (VcardPropKind, VcardParamKind, VcardValueKind, VcardVersion), each with FromStr and Deref<str> for its wire name, plus VcardValue::kind and VcardParam::kind.
- Added VcardPropName (a known VcardPropKind or a verbatim unknown name) and switched VcardProp::name to it; an unrecognised or missing card version normalises to VcardVersion::V4_0 in the decoded model (byte-faithful round-tripping stays on the syntax tree).
- Added the per-property VcardPropSpec contract (allowed_versions, cardinality, allowed_values, allowed_params and the in-force value) on the lens markers, with VcardPropCardinality, filled per RFC 6350 (including version-forked FN/N cardinality), plus VcardPropKind::ALL.
- Added Vcard::validate, an RFC 6350 conformance check over the decoded model (per-version property existence, value kind, version-aware parameters, and cardinality including required-but-absent) that permits extensions; Valid<T>, a marker only validation can mint (TryFrom<Vcard> for Valid<Vcard>); and From conversions Vcard -> VcardCst and Valid<Vcard> -> VcardCst.
- Added VcardPropBuilder, a version-aware, spec-driven builder for strict construction: it pins the property name and reuses the per-property validation, rejecting (via Result) a disallowed value kind or known parameter while allowing extension parameters.
- Added the initial pure, parser-free vCard model covering every RFC 6350 property: the Vcard aggregate, the generic VcardProperty value-plus-parameters wrapper, the VcardParameters set, the structured VcardName, VcardAddress, VcardGender, VcardDateAndOrTime, VcardUtcOffset and VcardClientPidMap values, the VcardUriOrText, VcardDateOrText and VcardTzValue value choices, and the VcardVersion, VcardKind and VcardSex enums.
- Added a generic VcardExtension property (name, parameters and decoded values) for anything beyond RFC 6350, with From-only typed views for the RFC 6474, 6715, 8605, 9554 and 9555 extension properties.
