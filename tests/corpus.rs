#![cfg(feature = "parser")]
//! Round-trip robustness sweep over the ez-vcard corpus through the one
//! version-agnostic parser. See tests/common/mod.rs for the harness and
//! tests/corpus/ez-vcard/ATTRIBUTION.md for provenance.

mod common;

#[test]
fn parses_and_round_trips() {
    common::each_fixture("ez-vcard", 17, common::round_trips);
}
