#![cfg(all(feature = "parser", feature = "quoted-printable", feature = "base64"))]
//! The content decoders under the detected CPU token decode every corpus value
//! as the codecs' portable baseline does.
//!
//! The token is process-wide, so this test has its own binary: installing it
//! here leaves the other test binaries on the baseline.

use std::{fs, path::PathBuf};

use base64_codec::codec::Base64Codec;
use quoted_printable_codec::codec::QuotedPrintableCodec;
use simd_cpu::SimdCpu;
use vcard::{
    tree::{cst::VcardCst, value::cursor::VcardValueCursor},
    value::{VcardValue, binary::VcardBinary},
};

#[test]
fn the_detected_token_decodes_as_the_baseline() {
    let _ = SimdCpu::install_detected();

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let (mut quoted, mut binary) = (0, 0);

    for corpus in fs::read_dir(&root).unwrap().flatten() {
        if !corpus.path().is_dir() {
            continue;
        }

        for fixture in fs::read_dir(corpus.path()).unwrap().flatten() {
            let path = fixture.path();
            if path.extension().and_then(|e| e.to_str()) != Some("vcf") {
                continue;
            }

            let input = fs::read(&path).unwrap();

            for card in VcardCst::parse_many(&input).flatten() {
                let mut card = card.into_static();

                for line in &mut card.props {
                    let declared = line.params.iter().any(|param| {
                        let param = param.to_string();
                        param.eq_ignore_ascii_case("QUOTED-PRINTABLE")
                            || param.eq_ignore_ascii_case("ENCODING=QUOTED-PRINTABLE")
                    });

                    if !declared {
                        continue;
                    }

                    let cursor = VcardValueCursor { line };
                    let mut expected = Vec::new();
                    QuotedPrintableCodec::BASELINE.decode_vec(&cursor.bytes(), &mut expected);
                    assert_eq!(cursor.quoted_printable(), expected, "{path:?}");
                    quoted += 1;
                }

                for prop in card.decode().properties {
                    let VcardValue::Binary(value @ VcardBinary::Base64(data)) = &prop.value else {
                        continue;
                    };

                    let mut expected = Vec::new();
                    let clean = Base64Codec::BASELINE.decode_vec(data.as_bytes(), &mut expected);
                    assert_eq!(value.decode_base64(), Some((expected, clean)), "{path:?}");
                    binary += 1;
                }
            }
        }
    }

    assert!(quoted > 0 && binary > 0, "{quoted} quoted, {binary} binary");
}
