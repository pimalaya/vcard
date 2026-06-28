//! Real-world vCard corpus, imported from the ez-vcard project (see
//! `tests/corpus/ATTRIBUTION.md` for provenance and licensing).
//!
//! This is a robustness harness, not a golden-output suite. The fixtures are
//! vCard 2.1 / 3.0 / 4.0 exports from real apps (Android, BlackBerry, Evolution,
//! Gmail, iPhone, Lotus Notes, Mac Address Book, MS Outlook, Thunderbird,
//! FullContact) plus the RFC 2426 / 6350 examples. For every fixture we assert
//! that parsing never panics. A fixture we expect to parse must also be a
//! serialize fixpoint (stable under reparse) and decode without panicking; a
//! fixture in [`EXPECTED_PARSE_FAILURES`] must return a parse error instead. The
//! list is a deliberate ledger of currently-unmodelled inputs: if one starts
//! parsing, update the list (and celebrate).

use std::{fs, path::PathBuf};

use vcard::tree::cst::VcardCst;

/// Fixtures the parser does not yet accept, with the reason. Each must fail to
/// parse (without panicking); everything else must parse and round-trip.
const EXPECTED_PARSE_FAILURES: &[&str] = &[
    // vCard 2.1 QUOTED-PRINTABLE soft line breaks: a `=`-terminated transfer
    // encoding distinct from RFC 6350 whitespace folding, not yet modelled.
    "outlook-2003.vcf",
    "outlook-2007.vcf",
    "John_Doe_MS_OUTLOOK.vcf",
];

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/ez-vcard")
}

#[test]
fn corpus_parses_and_round_trips() {
    let mut total = 0;

    for entry in fs::read_dir(corpus_dir()).expect("read corpus dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("vcf") {
            continue;
        }
        total += 1;

        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let input = String::from_utf8(fs::read(&path).expect("read fixture"))
            .unwrap_or_else(|_| panic!("{name} is not valid UTF-8"));

        let result = VcardCst::parse(&input);

        if EXPECTED_PARSE_FAILURES.contains(&name.as_str()) {
            assert!(
                result.is_err(),
                "{name} now parses; remove it from the ledger"
            );
            continue;
        }

        let card = result.unwrap_or_else(|e| panic!("parse {name}: {e}"));

        // Anything we parse must serialize to a fixpoint (stable under reparse).
        let output = card.to_string();
        let reparsed = VcardCst::parse(&output).unwrap_or_else(|e| panic!("reparse {name}: {e}"));
        assert_eq!(
            reparsed.to_string(),
            output,
            "not a serialize fixpoint: {name}"
        );

        // Decoding the whole card must not panic.
        let _ = card.decode();
    }

    assert_eq!(total, 17, "expected 17 imported fixtures, found {total}");
}
