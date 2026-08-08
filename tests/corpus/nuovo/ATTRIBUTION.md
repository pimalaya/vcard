# nuovo/vCard-parser vCard fixtures

Real-world vCard cards copied verbatim from the test data of [nuovo/vCard-parser](https://github.com/nuovo/vCard-parser) (MIT), from its Example*.vcf, used here as a parser robustness corpus (parse, serialize to a fixpoint, decode without panicking), not as golden output. One card per file.

- forrest_2.1.vcf: 2.1 with `QUOTED-PRINTABLE` `LABEL`s and bare `TYPE` parameters.
- forrest_3.0.vcf: 3.0 with both a URI `AGENT` and a nested (folded) `AGENT` vCard.
- agent_3.0.vcf: 3.0 with a nested (folded) `AGENT` vCard and a `PHOTO` URL.
