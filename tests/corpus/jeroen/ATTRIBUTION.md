# jeroendesloovere/vcard vCard fixtures

Real-world vCard cards copied verbatim from the test data of [jeroendesloovere/vcard](https://github.com/jeroendesloovere/vcard) (MIT), from tests/example.vcf and examples/assets/contacts.vcf (split into one card per file), used here as a parser robustness corpus (parse, serialize to a fixpoint, decode without panicking), not as golden output.

- grouped_3.0.vcf: 3.0 with `item`-grouped `EMAIL` / `URL` / `X-ABLabel`.
- contact_1_3.0.vcf .. contact_5_3.0.vcf: 3.0 UTF-8 cards with `X-MAIDENNAME`, `BIRTHPLACE`, `CATEGORIES` and escaped `\n` in `NOTE`.
