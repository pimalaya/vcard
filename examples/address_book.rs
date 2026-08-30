//! Walk a multi-card export, and read a bare directory record.
//!
//! A CardDAV or Google export is one file holding many cards. `parse_many`
//! iterates it card by card, and each card is the same tree as a single one.
//! A record carrying no BEGIN and END envelope (RFC 2425) parses too.
//!
//! Run with: `cargo run --example address_book`

use vcard::{
    prop::{r#fn::FN, uid::UID},
    tree::cst::VcardCst,
};

fn main() {
    let export = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "UID:urn:uuid:11111111-1111-1111-1111-111111111111\r\n",
        "FN:John Doe\r\n",
        "EMAIL;TYPE=work:john@acme.example\r\n",
        "EMAIL;TYPE=home;PREF=1:john@home.example\r\n",
        "END:VCARD\r\n",
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "UID:urn:uuid:22222222-2222-2222-2222-222222222222\r\n",
        "FN:Jane Roe\r\n",
        "EMAIL:jane@example.com\r\n",
        "END:VCARD\r\n",
        "\r\n",
        "BEGIN:VCARD\r\n",
        "VERSION:3.0\r\n",
        "N:Smith;Ada;;;\r\n",
        "FN:Ada Smith\r\n",
        "END:VCARD\r\n",
    );

    // The iterator stops at the first malformed card, so collecting into a
    // Result surfaces a broken file as one error.
    let cards: Vec<VcardCst> = VcardCst::parse_many(export)
        .collect::<Result<_, _>>()
        .unwrap();
    println!("{} cards\n", cards.len());

    for card in &cards {
        // A lens reads the first instance of a property. Where a card may
        // carry several (EMAIL, TEL), walk the lines instead.
        let uid = card.prop::<UID>().map(|uid| uid.0);
        println!("{} ({})", card.prop::<FN>().unwrap().0, &*card.version());
        println!("  uid:    {}", uid.as_deref().unwrap_or("none"));

        for line in &card.props {
            if line.name.get().eq_ignore_ascii_case("EMAIL") {
                println!("  email:  {}", line.value.decode());
            }
        }
    }

    // A bare RFC 2425 directory record: no envelope, no VERSION line. The card
    // parses, its version normalises to 4.0, and it round-trips as written.
    let record = "FN:John Doe\r\nEMAIL:john@example.com\r\n";
    let card = VcardCst::parse(record).unwrap();
    println!("\nbare record: {}", card.prop::<FN>().unwrap().0);
    assert!(card.begin.is_none());
    assert_eq!(card.to_bytes(), record.as_bytes());
}
