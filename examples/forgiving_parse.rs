//! Parse a card no standard would bless, and write it back byte for byte.
//!
//! Real exports are messy: folded lines, blank lines in the middle, mixed line
//! endings, quoted-printable soft breaks, group prefixes, bare 2.1 parameters
//! and properties no RFC defines. All of it parses, and all of it comes back
//! exactly as it went in, so a card can be read and stored without a rewrite.
//!
//! Run with: `cargo run --example forgiving_parse`

use vcard::{
    prop::{r#fn::FN, note::NOTE},
    tree::cst::VcardCst,
};

fn main() {
    let input = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:2.1\r\n",
        // A blank line in the middle of the card.
        "\r\n",
        // A line folded the RFC way, with a leading space on the next one.
        "FN:John Quinlan\r\n",
        "  Doe\r\n",
        // A group prefix, and two bare 2.1 parameters carrying no value.
        "item1.EMAIL;INTERNET;PREF:john@example.com\r\n",
        // A quoted-printable soft break: the `=` at the end of the line
        // continues the value on the next physical line, with no folding.
        "NOTE;ENCODING=QUOTED-PRINTABLE:a note that keeps=\r\n",
        " going\r\n",
        // A property no version defines, kept as written.
        "X-MS-CARDPICTURE:whatever\r\n",
        // A line ending in a bare LF instead of a CRLF.
        "TEL;CELL:+15550100\n",
        "END:VCARD\r\n",
    );

    let card = VcardCst::parse(input).unwrap();

    // Folding is resolved for every layer above, so a read sees the logical
    // value, not the physical lines it was written on.
    println!("FN:   {:?}", card.prop::<FN>().unwrap().0);
    println!("NOTE: {:?}", card.prop::<NOTE>().unwrap().0);
    println!("lines: {}", card.props.len());

    // Nothing was normalised away: the exact input bytes come back out.
    assert_eq!(card.to_bytes(), input.as_bytes());
    println!("round-trips byte for byte");
}
