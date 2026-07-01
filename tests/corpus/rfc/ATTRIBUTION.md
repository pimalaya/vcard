# RFC / spec vCard corpus

These fixtures are transcribed verbatim from the vCard specifications, used here
only as a parser robustness corpus (parse, serialize to a fixpoint, decode
without panicking), not as golden output. Spec text is published by the IETF
Trust and the versit consortium; the examples are reproduced for interoperability
testing.

One card per file. Snippets that appear in a spec as bare property lines were
wrapped in a minimal `BEGIN:VCARD` / `VERSION` / `END:VCARD` envelope; every
property line is otherwise verbatim. The exception is
`rfc2425_record1_nobegin.vcf`, kept envelope-free on purpose to exercise the
bare-record parse path.

## Provenance

- **RFC 6350** (vCard 4.0): `rfc6350_sync_pid_1.vcf`, `rfc6350_sync_pid_2.vcf`
  (section 7.1.3), `rfc6350_creation.vcf` (7.2.1), `rfc6350_shared.vcf` (7.2.3),
  `rfc6350_adr_params.vcf` (6.3.1 `ADR` with `GEO` / `LABEL` parameters),
  `rfc6350_escaping.vcf` (6.6.4 / 6.7.1 / 6.7.2 escaped delimiters).
- **RFC 6474** (birth / death): `rfc6474_places.vcf`.
- **RFC 6715** (OMA CAB, `EXPERTISE` / `HOBBY` / `INTEREST` / `ORG-DIRECTORY`):
  `rfc6715_oma.vcf`.
- **RFC 8605** (RDAP, `CONTACT-URI` / `CC`): `rfc8605_rdap.vcf`.
- **RFC 2425** (text/directory): `rfc2425_record1_nobegin.vcf` (section 6.1
  example: a bare directory record with no `BEGIN:VCARD` envelope),
  `rfc2425_record2.vcf`, `rfc2425_record3_groups.vcf` (section 8 examples;
  grouped properties, `ENCODING=B`, `language`).
- **versit vCard 2.1**: `vcard21_basic.vcf` (bare-parameter `TYPE`),
  `vcard21_qp_label.vcf` (`QUOTED-PRINTABLE` soft line breaks).
- **Per-version `GEO`**: `vcard21_geo.vcf` (2.1 comma pair), `vcard30_geo.vcf`
  (3.0 semicolon pair), `vcard40_geo.vcf` (4.0 `geo:` URI): the same coordinate
  in each dialect's shape.
