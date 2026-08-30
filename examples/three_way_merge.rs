//! Reconcile two divergent edits of the same card against their common base.
//!
//! This is what a synchronisation engine needs: the phone and the server both
//! edited the card they last agreed on, and the merge says what each side did,
//! where they collided, and hands back one card carrying both sets of changes.
//!
//! Run with: `cargo run --example three_way_merge`

use vcard::tree::{cst::VcardCst, merge::VcardMerge};

fn main() {
    let base = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "FN:John Doe\r\n",
        "EMAIL;TYPE=work:john@acme.example\r\n",
        "TEL:+33123456789\r\n",
        "CATEGORIES:work\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();

    // The phone renamed the contact, added a nickname and a category.
    let left = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "FN:John D.\r\n",
        "EMAIL;TYPE=work:john@acme.example\r\n",
        "TEL:+33123456789\r\n",
        "CATEGORIES:work,friend\r\n",
        "NICKNAME:Johnny\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();

    // The server renamed it differently, changed the phone number and dropped
    // the email.
    let right = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "FN:Johnathan Doe\r\n",
        "TEL:+33987654321\r\n",
        "CATEGORIES:work\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();

    let report = VcardMerge {
        base: &base,
        left: &left,
        right: &right,
    }
    .merge();

    println!("left did:");
    for action in &report.left {
        println!("  {action:?}");
    }

    println!("\nright did:");
    for action in &report.right {
        println!("  {action:?}");
    }

    // Divergent changes to the same field. The left side wins in the merged
    // card, but both are reported, so a caller free to resolve otherwise can.
    println!("\nconflicts:");
    for conflict in &report.conflicts {
        println!("  left  {:?}", conflict.left);
        println!("  right {:?}", conflict.right);
    }

    // The merged card is the left card's bytes with the right side's
    // non-conflicting changes replayed onto them.
    print!("\n{}", report.merged);
}
