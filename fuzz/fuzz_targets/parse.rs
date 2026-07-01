#![no_main]

//! Coverage-guided fuzz target for the parser. Two oracles: parsing arbitrary
//! bytes must never panic, and whatever parses must serialize to a byte-stable
//! fixpoint (its own output reparses identically).

use libfuzzer_sys::fuzz_target;
use vcard::tree::cst::VcardCst;

fuzz_target!(|data: &[u8]| {
    if let Ok(cst) = VcardCst::parse(data) {
        let bytes = cst.to_bytes();
        let _ = cst.decode();
        let _ = cst.to_string();

        let reparsed = VcardCst::parse(&bytes).expect("serialized output must reparse");
        assert_eq!(reparsed.to_bytes(), bytes, "serialization is not idempotent");
    }

    // The multi-card and bare-record paths must not panic either.
    for card in VcardCst::parse_many(data).flatten() {
        let _ = card.to_bytes();
        let _ = card.decode();
    }
});
