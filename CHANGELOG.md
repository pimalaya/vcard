# Changelog

All notable changes to this project are documented in this file. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

## [Unreleased]

### Added

- Added the initial pure, parser-free vCard model covering every RFC 6350 property: the Vcard aggregate, the generic VcardProperty value-plus-parameters wrapper, the VcardParameters set, the structured VcardName, VcardAddress, VcardGender, VcardDateAndOrTime, VcardUtcOffset and VcardClientPidMap values, the VcardUriOrText, VcardDateOrText and VcardTzValue value choices, and the VcardVersion, VcardKind and VcardSex enums.
- Added a generic VcardExtension property (name, parameters and decoded values) for anything beyond RFC 6350, with From-only typed views for the RFC 6474, 6715, 8605, 9554 and 9555 extension properties.
