//! Parse a card, edit one field, and write it back.
//!
//! This shows the byte-faithful editing: only the field you touch changes,
//! every other byte of the original card is preserved exactly.
//!
//! Run with: `cargo run --example parse_edit_export`

use vcard::tree::cst::VcardCst;
use vcard::tree::prop::r#fn::FN;

fn main() {
    let input = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "FN:John Doe\r\n",
        "EMAIL;TYPE=work:john@example.com\r\n",
        "END:VCARD\r\n",
    );

    let mut card = VcardCst::parse(input).unwrap();

    // Read the display name.
    println!("before: {}", card.prop::<FN>().unwrap().0);

    // Rename the contact in place.
    card.prop_mut::<FN>().unwrap().set_text("Jane Doe");

    // The EMAIL line, its TYPE parameter and the exact line endings all survive
    // untouched; only FN changed.
    print!("{card}");
}
