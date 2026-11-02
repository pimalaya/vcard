#![no_main]

//! Coverage-guided fuzz target for the parser. Four oracles: parsing arbitrary
//! bytes must never panic, whatever parses must serialize to a byte-stable
//! fixpoint (its own output reparses identically), a file whose every card
//! parses must come back byte for byte, and the content decoders, run under
//! the detected CPU token, never panic and never grow a quoted-printable value.

use libfuzzer_sys::fuzz_target;
use simd_cpu::SimdCpu;
use vcard::{
    tree::{cst::VcardCst, value::cursor::VcardValueCursor},
    value::VcardValue,
};

fuzz_target!(
    init: {
        let _ = SimdCpu::install_detected();
    },
    |data: &[u8]| {
        if let Ok(cst) = VcardCst::parse(data) {
            let bytes = cst.to_bytes();
            let _ = cst.decode();
            let _ = cst.to_string();

            let reparsed = VcardCst::parse(&bytes).expect("serialized output must reparse");
            assert_eq!(reparsed.to_bytes(), bytes, "serialization is not idempotent");
        }

        // The multi-card and bare-record paths must not panic either, and a
        // file read whole is written whole: the cards a reader yields
        // concatenate back to the input, folds, blank lines and soft breaks
        // included.
        let mut whole = Vec::new();
        let mut faithful = true;

        for card in VcardCst::parse_many(data) {
            match card {
                Ok(mut card) => {
                    whole.extend_from_slice(&card.to_bytes());
                    decode_contents(&mut card);
                }
                Err(_) => faithful = false,
            }
        }

        // An input holding nothing but blank lines yields no card at all, so
        // there is nothing for the file to come back as.
        if faithful && !whole.is_empty() {
            assert_eq!(whole, data, "a file did not come back byte for byte");
        }
    }
);

/// Decodes every value of `card` through the content decoders.
fn decode_contents(card: &mut VcardCst<'_>) {
    for prop in card.decode().properties {
        if let VcardValue::Binary(binary) = &prop.value {
            let _ = binary.decode_base64();
        }
    }

    for line in &mut card.props {
        let cursor = VcardValueCursor { line };
        let decoded = cursor.quoted_printable();
        assert!(decoded.len() <= cursor.bytes().len(), "a decoded value grew");
        let _ = cursor.charset();
    }
}
