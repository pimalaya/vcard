#![cfg(feature = "parser")]

//! The vCard 4.0 slice of the shared ez-vcard corpus, parsed through the
//! [`crate::v40`](vcard::v40) tree. See tests/common/mod.rs for the harness
//! and tests/corpus/ATTRIBUTION.md for provenance.

mod common;

use vcard::v40::tree::cst::VcardCst;

#[test]
fn corpus_parses_and_round_trips() {
    common::each_fixture("4.0", 2, |name, input| {
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
