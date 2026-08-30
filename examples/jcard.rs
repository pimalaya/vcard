//! Write a card as jCard JSON, and read it back.
//!
//! jCard (RFC 7095) is the JSON spelling of the same model: a card is a
//! `["vcard", [...]]` array of `[name, {params}, type, value]` entries. It is
//! a projection of the decoded model, so it normalises rather than preserves;
//! byte fidelity stays the syntax tree's job.
//!
//! Run with: `cargo run --example jcard --features jcard`

use vcard::{tree::cst::VcardCst, vcard::Vcard};

fn main() {
    let input = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "FN:John Doe\r\n",
        "N:Doe;John;;Dr.;\r\n",
        "EMAIL;TYPE=work;PREF=1:john@acme.example\r\n",
        "CATEGORIES:work,rust\r\n",
        "X-CUSTOM:kept as written\r\n",
        "END:VCARD\r\n",
    );

    let card = VcardCst::parse(input).unwrap().decode().to_jcard();

    // Structured values become arrays, list values too, and an extension
    // property rides along under its own name.
    println!("{card:#}");

    // Back to the model, then to the wire. The card says the same thing, in
    // the codec's canonical spelling rather than the original's.
    let card = Vcard::from_jcard(&card).unwrap();
    print!("\n{card}");
}
