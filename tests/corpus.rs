#![cfg(feature = "parser")]
//! Round-trip robustness sweep over the ez-vcard corpus through the one
//! version-agnostic parser. See tests/common/mod.rs for the harness and
//! tests/corpus/ez-vcard/ATTRIBUTION.md for provenance.

mod common;

use vcard::tree::cst::VcardCst;

#[test]
fn parses_and_round_trips() {
    common::each_fixture("ez-vcard", 17, |name, input| {
        let card = VcardCst::parse(input).unwrap_or_else(|e| panic!("parse {name}: {e}"));

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
    });
}
