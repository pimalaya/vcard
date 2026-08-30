//! Read the same properties out of a 2.1, a 3.0 and a 4.0 card.
//!
//! The version is decoded data, never a type parameter, so one piece of code
//! reads all three. Only what the RFCs actually changed comes back different:
//! here, the shape of a PHOTO value.
//!
//! Run with: `cargo run --example all_versions`

use vcard::{
    prop::{r#fn::FN, photo::PHOTO, tel::TEL},
    tree::{cst::VcardCst, param::r#type::TYPE},
    value::VcardValue,
};

const CARDS: [&str; 3] = [
    concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:2.1\r\n",
        "N:Doe;John;;;\r\n",
        "FN:John Doe\r\n",
        "TEL;WORK;VOICE:+33123456789\r\n",
        "PHOTO;ENCODING=BASE64;TYPE=JPEG:Zm9vYmFy\r\n",
        "END:VCARD\r\n",
    ),
    concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:3.0\r\n",
        "N:Doe;John;;;\r\n",
        "FN:John Doe\r\n",
        "TEL;TYPE=WORK,VOICE:+33123456789\r\n",
        "PHOTO;VALUE=URI:https://example.org/john.jpg\r\n",
        "END:VCARD\r\n",
    ),
    concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "FN:John Doe\r\n",
        "TEL;TYPE=work,voice;VALUE=uri:tel:+33123456789\r\n",
        "PHOTO:data:image/jpeg;base64,Zm9vYmFy\r\n",
        "END:VCARD\r\n",
    ),
];

fn main() {
    for input in CARDS {
        let card = VcardCst::parse(input).unwrap();

        // The same lens reads the same property whatever the version.
        println!("vCard {}", &*card.version());
        println!("  FN:    {}", card.prop::<FN>().unwrap().0);
        println!("  TEL:   {}", card.prop::<TEL>().unwrap().0);

        // A 2.1 card writes its types as bare parameters (`TEL;WORK;VOICE`),
        // which no version defines as a TYPE. Nothing is invented: the lens
        // finds a TYPE only where one was written.
        let types = card
            .props
            .iter()
            .find(|line| line.name.get() == "TEL")
            .and_then(|line| line.param::<TYPE>());
        println!("  TYPE:  {types:?}");

        // PHOTO is where the versions genuinely differ: 2.1 and 3.0 carry a
        // binary value (inline base64 or a URI reference), 4.0 a data: URI.
        // The lens resolves that shape from the card's own version.
        match card.prop::<PHOTO>().unwrap() {
            VcardValue::Binary(binary) => println!("  PHOTO: {binary:?}"),
            VcardValue::Uri(uri) => println!("  PHOTO: Uri({})", uri.0),
            other => println!("  PHOTO: {other:?}"),
        }

        println!();
    }
}
