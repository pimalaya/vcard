//! Convert a card to a JSContact Card and back.
//!
//! JSContact (RFC 9553) is a different model, not another spelling: a card is
//! an object of named members, an ADR line is a structured Address and a TYPE
//! parameter is a context. RFC 9555 defines the mapping, and both directions
//! are lossless through its escape hatches.
//!
//! Run with: `cargo run --example jscontact --features jscontact`

use vcard::{tree::cst::VcardCst, vcard::Vcard};

fn main() {
    let input = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "UID:urn:uuid:11111111-1111-1111-1111-111111111111\r\n",
        "FN:John Doe\r\n",
        "N:Doe;John;;Dr.;\r\n",
        "EMAIL;TYPE=home:john@home.example\r\n",
        "TEL;TYPE=cell:+33123456789\r\n",
        "ADR;TYPE=work:;;123 Main St;Springfield;IL;62704;USA\r\n",
        // No JSContact counterpart: this one rides the escape hatch.
        "X-CUSTOM:kept as written\r\n",
        "END:VCARD\r\n",
    );

    let card = VcardCst::parse(input).unwrap().decode().to_jscontact();

    // TYPE=home becomes a `private` context, TYPE=cell a `mobile` feature, and
    // the unmappable X-CUSTOM is preserved whole in `vCardProps`, in jCard
    // syntax.
    println!("{card:#}");

    // The mirror hatch on the way back: nothing is lost, X-CUSTOM included.
    // The conversion normalises rather than preserves, so the properties come
    // back in the Card's member order, each carrying the PROP-ID that keeps
    // its map key stable across a round trip.
    let card = Vcard::from_jscontact(&card).unwrap();
    print!("\n{card}");
}
