# mixerp/MixERP.Net.VCards vCard fixtures

Real-world vCard cards copied verbatim from the test data of [mixerp/MixERP.Net.VCards](https://github.com/mixerp/MixERP.Net.VCards) (Apache-2.0), from `src/MixERP.Net.VCards.UI/*.vcf`, used here as a parser robustness corpus (parse, serialize to a fixpoint, decode without panicking), not as golden output. One card per file.

- `simple_4.0.vcf`: 4.0 with `CLASS` / `KIND` / `GENDER`.
- `complex_4.0.vcf`: 4.0 with a large inline base64 `LOGO` and many properties.
