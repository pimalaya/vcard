//! Read a vCard 2.1 card whose content is not plain UTF-8 text.
//!
//! The core transforms nothing: a QUOTED-PRINTABLE, a BASE64 and a foreign
//! CHARSET reach you exactly as they sit on the wire, their parameters kept,
//! so nothing is silently transcoded. Resolving them is opt-in, one small
//! crate per cargo feature.
//!
//! Run with: `cargo run --example content_encodings --all-features`

use vcard::{
    prop::{note::NOTE, photo::PHOTO},
    tree::cst::VcardCst,
    value::VcardValue,
};

fn main() {
    // "café" written in ISO-8859-1 (its 0xE9 is not valid UTF-8), through
    // quoted-printable octets, the way a 2.1 exporter writes it.
    let input = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:2.1\r\n",
        "FN:John Doe\r\n",
        "NOTE;CHARSET=ISO-8859-1;ENCODING=QUOTED-PRINTABLE:met at the caf=E9\r\n",
        "PHOTO;ENCODING=BASE64;TYPE=JPEG:Zm9vYmFy\r\n",
        "END:VCARD\r\n",
    );

    let mut card = VcardCst::parse(input).unwrap();
    let note = card.prop_mut::<NOTE>().unwrap();

    // Raw: the value is the wire bytes, escaping resolved and nothing else.
    println!(
        "raw:             {}",
        String::from_utf8_lossy(&note.bytes())
    );

    // `quoted-printable` resolves the =XX octets. Still bytes, and still in
    // their own charset: the trailing 0xE9 is no UTF-8 café.
    println!("octets resolved: {:02x?}", note.quoted_printable());

    // `encoding` transcodes to text through the CHARSET parameter, resolving
    // the quoted-printable first when that feature is on too.
    println!("as text:         {}", note.charset());

    // A 2.1 binary value keeps its base64 payload verbatim; `base64` decodes
    // it to bytes on demand.
    match card.prop::<PHOTO>().unwrap() {
        VcardValue::Binary(binary) => {
            println!("\nphoto payload:   {binary:?}");
            let bytes = binary.decode_base64().unwrap().unwrap();
            println!("photo bytes:     {} bytes, {:02x?}", bytes.len(), bytes);
        }
        other => println!("\nphoto:           {other:?}"),
    }

    // None of the reads touched the card: it is still the bytes it arrived as.
    assert_eq!(card.to_bytes(), input.as_bytes());
}
