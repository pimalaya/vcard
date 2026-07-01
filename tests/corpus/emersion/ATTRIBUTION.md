# emersion/go-vcard vCard fixtures

Real-world vCard cards copied verbatim from the test data of [emersion/go-vcard](https://github.com/emersion/go-vcard) (MIT), transcribed from `decoder_test.go`, used here as a parser robustness corpus (parse, serialize to a fixpoint, decode without panicking), not as golden output. One card per file.

- `rfc_4.0.vcf`: 4.0 with `PID` parameters and `CLIENTPIDMAP`.
- `handmade_4.0.vcf`: 4.0 with a quoted `TYPE="cell,home"`, `IMPP` and `X-SOCIALPROFILE`.
- `google_3.0.vcf`: 3.0 Google Contacts export; `item`-grouped `URL` / `X-ABLabel` and escaped `http\://` colons.
- `apple_3.0.vcf`: 3.0 Apple Contacts export; lowercase `type=` parameters and `X-SERVICE-TYPE`.
- `linefolding_4.0.vcf`: 4.0 with a folded `NOTE` and interior blank lines.
