//! Build a card the strict way, then export it.
//!
//! Each property is checked against the RFC as it is built, and the finished
//! card is validated as a whole before it is written out. If anything does not
//! conform, you get the list of problems instead of a bad card.
//!
//! Run with: `cargo run --example strict_builder`

use std::borrow::Cow;

use vcard::param::VcardParam;
use vcard::tree::cst::VcardCst;
use vcard::tree::prop::email::EMAIL;
use vcard::tree::prop::r#fn::FN;
use vcard::tree::vcard::builder::VcardPropBuilder;
use vcard::value::VcardValue;
use vcard::vcard::Vcard;
use vcard::version::VcardVersion;

fn main() {
    let version = VcardVersion::V4_0;

    // The builder pins each property's name and rejects a value or a parameter
    // the property is not allowed to carry.
    let full_name = VcardPropBuilder::<FN>::new(version)
        .build(VcardValue::Text("John Doe".into()))
        .expect("FN accepts a text value");

    let email = VcardPropBuilder::<EMAIL>::new(version)
        .param(VcardParam::Pref(Cow::Borrowed("1")))
        .build(VcardValue::Text("john@example.com".into()))
        .expect("EMAIL accepts a text value with a PREF parameter");

    let card = Vcard {
        version,
        properties: vec![full_name, email],
    };

    // Validate the whole card: this is where a missing required property or a
    // wrong multiplicity would be caught. Success yields a proof value.
    let valid = card.validate().expect("a conformant 4.0 card");

    // A validated card converts straight back to bytes.
    print!("{}", VcardCst::from(valid));
}
