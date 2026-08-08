# sabre-io/vobject vCard fixtures

Real-world vCard cards copied verbatim from the test data of [sabre-io/vobject](https://github.com/sabre-io/vobject) (BSD-3-Clause), from tests/VObject/*.vcf, used here as a parser robustness corpus (parse, serialize to a fixpoint, decode without panicking), not as golden output. One card per file.

- issue64_photo_2.1.vcf: 2.1 with a two-space-folded base64 `PHOTO`.
- issue153_photo_3.0.vcf: 3.0 with a two-space-folded base64 `PHOTO`.
