#![cfg(feature = "parser")]
//! Round-trip robustness sweep over the Stalwart calcard corpus through the one
//! version-agnostic parser. See tests/common/mod.rs for the harness and
//! tests/corpus/calcard/ATTRIBUTION.md for provenance.

mod common;

#[test]
fn parses_and_round_trips() {
    common::each_fixture("calcard", 92, common::round_trips);
}
