//! Shared corpus harness for the round-trip tests (`corpus`, `calcard`, `rfc`,
//! `github`).
//!
//! Each corpus is a single mixed-version directory under tests/corpus/, with
//! provenance and licensing in each one's ATTRIBUTION.md.
//!
//! It is a robustness harness, not a golden-output suite: the fixtures are
//! real-world vCard 2.1 / 3.0 / 4.0 cards plus the RFC 2426 / 6350 examples.
//!
//! The one version-agnostic parser handles them all, so each test sweeps the
//! whole corpus and asserts every fixture parses, comes back byte for byte, is
//! a serialization fixpoint, and decodes without panicking.

// Each integration test compiles this module separately and uses only the part
// of it that it needs.
#![allow(dead_code)]

use std::{fs, path::PathBuf};

use vcard::tree::cst::VcardCst;

/// Runs `check` against every `.vcf` fixture of `corpus`, asserting exactly
/// `expected` of them are present so a misfiled, renamed or newly added fixture
/// is caught. `check` receives the fixture name and its raw text.
pub fn each_fixture(corpus: &str, expected: usize, mut check: impl FnMut(&str, &str)) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus")
        .join(corpus);

    let mut total = 0;

    for entry in fs::read_dir(&dir).expect("read corpus dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("vcf") {
            continue;
        }

        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let input = String::from_utf8(fs::read(&path).expect("read fixture"))
            .unwrap_or_else(|_| panic!("{name} is not valid UTF-8"));

        total += 1;
        check(&name, &input);
    }

    assert_eq!(
        total, expected,
        "expected {expected} fixtures, found {total}"
    );
}

/// Asserts every guarantee the corpus is swept for: the fixture parses, comes
/// back byte for byte however it was laid out, reparses to those same bytes,
/// and decodes without panicking.
pub fn round_trips(name: &str, input: &str) {
    let output = parse_whole(input).unwrap_or_else(|| panic!("parse {name}"));

    // Byte-faithful: the fixture serializes back to itself, its folds, its
    // blank lines and its QUOTED-PRINTABLE soft breaks included.
    assert_eq!(output, input, "not byte-faithful: {name}");

    // Fixpoint: the serialized bytes reparse to the same bytes.
    let reparsed = parse_whole(&output).unwrap_or_else(|| panic!("reparse {name}"));
    assert_eq!(reparsed, output, "not a serialize fixpoint: {name}");

    // Decoding must never panic.
    for card in cards(input).expect("already parsed") {
        let _ = card.decode();
    }
}

/// Every card of a file, in order, or `None` when the file cannot be
/// structured. A file holding no `BEGIN` at all is read as one bare,
/// envelope-less record.
fn cards(input: &str) -> Option<Vec<VcardCst<'_>>> {
    let mut all = Vec::new();

    for result in VcardCst::parse_many(input) {
        match result {
            Ok(cst) => all.push(cst),
            Err(_) if all.is_empty() => return VcardCst::parse(input).ok().map(|cst| vec![cst]),
            Err(_) => return None,
        }
    }

    Some(all)
}

/// The whole file, parsed and serialized straight back.
fn parse_whole(input: &str) -> Option<String> {
    let mut out = String::new();

    for card in cards(input)? {
        out.push_str(&card.to_string());
    }

    Some(out)
}
