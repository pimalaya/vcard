//! Assemble a card by hand and export it, with no checks.
//!
//! This is the escape hatch: you place whatever properties you like and write
//! them out directly. Nothing verifies that the result conforms to the RFC, so
//! correctness is entirely your responsibility. Reach for the strict builder
//! (see the strict_builder example) when you want that guarantee.
//!
//! Run with: `cargo run --example raw_builder`

use std::borrow::Cow;

use vcard::prop::VcardProp;
use vcard::value::VcardValue;
use vcard::value::text::VcardText;
use vcard::vcard::Vcard;
use vcard::version::VcardVersion;

fn main() {
    let card = Vcard {
        version: VcardVersion::V4_0,
        properties: vec![
            VcardProp {
                name: "FN".into(),
                params: vec![],
                value: VcardValue::Text(VcardText(Cow::Borrowed("John Doe"))),
            },
            // A custom extension property the strict builder has no marker for.
            // Nothing here is validated; it is written out as given.
            VcardProp {
                name: "X-CUSTOM".into(),
                params: vec![],
                value: VcardValue::Text(VcardText(Cow::Borrowed("anything goes"))),
            },
        ],
    };

    print!("{card}");
}
