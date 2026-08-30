//! Check a card against its version's RFC, and repair one that fails.
//!
//! Validation reports every violation at once rather than the first: a
//! property the version does not define, a value kind or a parameter the
//! property may not carry, and a multiplicity it breaks (a required property
//! missing included).
//!
//! Run with: `cargo run --example validate_errors`

use std::borrow::Cow;

use vcard::{
    param::VcardParam, prop::VcardProp, tree::cst::VcardCst, value::VcardValue, vcard::Vcard,
    version::VcardVersion,
};

fn main() {
    // A card assembled by hand, with no check on the way in.
    let card = Vcard {
        version: VcardVersion::V4_0,
        properties: vec![
            // MAILER was removed in 4.0.
            VcardProp {
                name: "MAILER".into(),
                params: vec![],
                value: VcardValue::Text("Mutt".into()),
            },
            // EMAIL carries no LANGUAGE parameter.
            VcardProp {
                name: "EMAIL".into(),
                params: vec![VcardParam::Language(Cow::Borrowed("en"))],
                value: VcardValue::Text("john@example.com".into()),
            },
            // N takes its own structured value, not a text.
            VcardProp {
                name: "N".into(),
                params: vec![],
                value: VcardValue::Text("Doe, John".into()),
            },
            // and FN, which 4.0 requires, is missing entirely.
        ],
    };

    let errors = card.validate().unwrap_err();
    println!("{} violations:", errors.len());
    for error in errors {
        println!("  {error}");
    }

    // A likelier failure: a 3.0 card exported without N, which strict servers
    // (iCloud, Fastmail) reject.
    let input = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:3.0\r\n",
        "FN:John Doe\r\n",
        "EMAIL;TYPE=INTERNET:john@example.com\r\n",
        "END:VCARD\r\n",
    );

    let mut card = VcardCst::parse(input).unwrap();
    println!();
    report("as exported", &card);

    // Seed a blank instance of whatever the version requires. The existing
    // lines are left untouched, so only the repair shows up in the bytes.
    card.fill_required();
    report("repaired   ", &card);

    print!("\n{card}");
}

/// Validate a parsed card and print the outcome on one line.
fn report(label: &str, card: &VcardCst) {
    match card.decode().validate() {
        Ok(_) => println!("{label}: conformant"),
        Err(errors) => {
            let errors: Vec<_> = errors.iter().map(ToString::to_string).collect();
            println!("{label}: {}", errors.join("; "));
        }
    }
}
