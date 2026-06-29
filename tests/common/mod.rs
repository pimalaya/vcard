//! Shared corpus harness for the per-version round-trip tests
//! (`corpus_v2_1`, `corpus_v3_0`, `corpus_v4_0`).
//!
//! The ez-vcard corpus is a single mixed-version directory (provenance and
//! licensing live in tests/corpus/ATTRIBUTION.md). It is a robustness harness,
//! not a golden-output suite: the fixtures are vCard 2.1 / 3.0 / 4.0 exports
//! from real apps (Android, BlackBerry, Evolution, Gmail, iPhone, Lotus Notes,
//! Mac Address Book, MS Outlook, Thunderbird, FullContact) plus the RFC 2426 /
//! 6350 examples. Each version test routes the fixtures whose VERSION line
//! matches its spec to its own parser and asserts they parse, serialize to a
//! fixpoint (stable under reparse) and decode without panicking.

use std::{fs, path::PathBuf};

/// Runs `check` against every corpus fixture whose `VERSION` line is `version`,
/// asserting exactly `expected` of them are present so a misfiled, renamed or
/// newly added fixture is caught. `check` receives the fixture name and its raw
/// text, and parses, round-trips and decodes it.
pub fn each_fixture(version: &str, expected: usize, check: impl Fn(&str, &str)) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/ez-vcard");

    let mut total = 0;

    for entry in fs::read_dir(&dir).expect("read corpus dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("vcf") {
            continue;
        }

        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let input = String::from_utf8(fs::read(&path).expect("read fixture"))
            .unwrap_or_else(|_| panic!("{name} is not valid UTF-8"));

        if fixture_version(&input) != version {
            continue;
        }
        total += 1;

        check(&name, &input);
    }

    assert_eq!(
        total, expected,
        "expected {expected} vCard {version} fixtures, found {total}"
    );
}

/// The value of the fixture's `VERSION` line (e.g. `2.1`, `3.0`, `4.0`), or the
/// empty string if the card has none. Any parameters before the `:` are
/// dropped.
fn fixture_version(input: &str) -> &str {
    for line in input.lines() {
        let rest = match line.trim().split_once([':', ';']) {
            Some((name, rest)) if name.eq_ignore_ascii_case("VERSION") => rest,
            _ => continue,
        };

        return rest.rsplit(':').next().unwrap_or(rest).trim();
    }

    ""
}
