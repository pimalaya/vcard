//! Shared corpus harness for the round-trip tests (`corpus`, `calcard`).
//!
//! Each corpus is a single mixed-version directory under tests/corpus/
//! (provenance and licensing live in each one's ATTRIBUTION.md). It is a
//! robustness harness, not a golden-output suite: the fixtures are real-world
//! vCard 2.1 / 3.0 / 4.0 cards plus the RFC 2426 / 6350 examples. The one
//! version-agnostic parser handles them all, so each test sweeps the whole
//! corpus and asserts every fixture parses, serializes to a fixpoint (stable
//! under reparse) and decodes without panicking.

use std::{fs, path::PathBuf};

/// Runs `check` against every `.vcf` fixture of `corpus`, asserting exactly
/// `expected` of them are present so a misfiled, renamed or newly added fixture
/// is caught. `check` receives the fixture name and its raw text.
pub fn each_fixture(corpus: &str, expected: usize, check: impl Fn(&str, &str)) {
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
