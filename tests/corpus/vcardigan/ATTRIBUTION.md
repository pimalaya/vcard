# brewster/vcardigan vCard fixtures

Real-world vCard cards copied verbatim from the test data of [brewster/vcardigan](https://github.com/brewster/vcardigan) (MIT), from `spec/helpers/*.vcf`, used here as a parser robustness corpus (parse, serialize to a fixpoint, decode without panicking), not as golden output. One card per file.

- `google_3.0.vcf`: 3.0 with a folded base64 `PHOTO` and an `X-SOCIALPROFILE` carrying custom parameters and escaped colons.
- `joe_4.0.vcf`: minimal 4.0 card.
