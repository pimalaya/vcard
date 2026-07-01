#![cfg(feature = "parser")]
//! Round-trip robustness sweep over vCard fixtures harvested from popular
//! open-source vCard libraries on GitHub, run through the one version-agnostic
//! parser. Each source repo has its own corpus directory under tests/corpus/
//! (with its own ATTRIBUTION.md), so provenance and licensing stay per project.

mod common;

use vcard::tree::cst::VcardCst;

#[test]
fn parses_and_round_trips() {
    // One entry per source repo: its corpus directory and its fixture count.
    let projects = [
        ("emersion", 5),
        ("jeroen", 6),
        ("mixerp", 2),
        ("nuovo", 3),
        ("sabre", 2),
        ("vcardigan", 2),
    ];

    for (project, count) in projects {
        common::each_fixture(project, count, |name, input| {
            let card = VcardCst::parse(input).unwrap_or_else(|e| panic!("parse {name}: {e}"));

            // Anything we parse must serialize to a fixpoint (stable under
            // reparse).
            let output = card.to_string();
            let reparsed =
                VcardCst::parse(&output).unwrap_or_else(|e| panic!("reparse {name}: {e}"));
            assert_eq!(reparsed.to_string(), output, "not a serialize fixpoint: {name}");

            // Decoding the whole card must not panic.
            let _ = card.decode();
        });
    }
}
