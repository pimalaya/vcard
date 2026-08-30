//! Read and edit the components of a structured value, one at a time.
//!
//! N and ADR are semicolon-separated components, CATEGORIES a comma-separated
//! list. Each has a cursor that names its parts, and every setter rewrites
//! only the part it touches: the rest of the line, its parameters and their
//! quoting included, keeps the bytes it was parsed with.
//!
//! Run with: `cargo run --example structured_values`

use vcard::{
    prop::{adr::ADR, categories::CATEGORIES, n::N},
    tree::{
        cst::VcardCst,
        param::{label::LABEL, r#type::TYPE},
    },
};

fn main() {
    let input = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "FN:John Doe\r\n",
        "N:Doe;John;Quinlan;Dr.;Jr.\r\n",
        "ADR;TYPE=work;LABEL=\"Acme HQ^nSpringfield\":;;123 Main St;Springfield;IL;62704;USA\r\n",
        "CATEGORIES:friend,work\r\n",
        "END:VCARD\r\n",
    );

    let mut card = VcardCst::parse(input).unwrap();

    // Components read back as lists, since each one may hold several values.
    let name = card.prop::<N>().unwrap();
    println!("family: {:?}", name.family);
    println!("given:  {:?}", name.given);
    println!("prefix: {:?}", name.prefixes);

    let address = card.prop::<ADR>().unwrap();
    println!("street: {:?}", address.street);
    println!("city:   {:?}", address.locality);

    let mut address = card.prop_mut::<ADR>().unwrap();

    // Parameters are read through their own lenses, off the same cursor. The
    // double quotes RFC 6350 wraps a value in are the grammar's delimiters, so
    // LABEL reads as its text, its `^n` resolved to a real line break.
    println!("type:   {:?}", address.param::<TYPE>());
    println!("label:  {:?}", address.param::<LABEL>());

    // The contact moves: two components change, the other sixteen and the
    // LABEL parameter's bytes are not rewritten.
    address.set_street(&["456 Oak Ave"]);
    address.set_postal_code(&["62705"]);

    // A list value is edited item by item, splicing a single leaf per call.
    let mut categories = card.prop_mut::<CATEGORIES>().unwrap();
    let mut items = categories.list_mut();
    items.remove(0);
    items.push("carddav");

    print!("\n{card}");
}
