#![cfg(feature = "parser")]
//! Round-trip robustness sweep over cards transcribed from the vCard RFCs and
//! the versit 2.1 spec through the one version-agnostic parser. See
//! tests/common/mod.rs for the harness and tests/corpus/rfc/ATTRIBUTION.md for
//! provenance.

mod common;

#[test]
fn parses_and_round_trips() {
    common::each_fixture("rfc", 17, common::round_trips);
}
