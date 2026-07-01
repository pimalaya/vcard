#![cfg(feature = "parser")]
//! Cards found in the specs / in the wild that the parser does not yet handle to
//! full fidelity. Each test pins one known gap and is `#[ignore]`d so the suite
//! stays green; drop the attribute once the gap is closed. Fixtures live in
//! tests/todo/, quarantined out of the tests/corpus/ database.

use std::{fs, path::PathBuf};

use vcard::tree::cst::VcardCst;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/todo")
        .join(name);
    String::from_utf8(fs::read(path).unwrap()).unwrap()
}

/// A 2.1 `AGENT` embeds a whole vCard as its value. Parsing stops at the nested
/// `END:VCARD`, dropping the outer one and reading the nested card as stray
/// properties, so the card does not round-trip byte for byte.
#[test]
#[ignore = "nested AGENT vCard is truncated at the inner END:VCARD"]
fn nested_agent_vcard_round_trips() {
    let input = fixture("vcard21_agent_nested.vcf");
    let card = VcardCst::parse(&input).expect("parse");
    assert_eq!(card.to_string(), input);
}

/// A bare RFC 2425 directory record has no `BEGIN:VCARD` envelope. The parser
/// requires one, so the record is rejected outright.
#[test]
#[ignore = "bare RFC 2425 directory record (no BEGIN:VCARD) is rejected"]
fn bare_directory_record_parses() {
    let input = fixture("rfc2425_record1_nobegin.vcf");
    VcardCst::parse(&input).expect("parse");
}
