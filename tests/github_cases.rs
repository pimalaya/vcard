#![cfg(feature = "parser")]
//! Focused input -> expected cases translated from the unit tests of popular
//! open-source vCard libraries on GitHub. Each case reproduces an explicit
//! assertion from a source repo (parse this card, expect that decoded value)
//! against this crate's `VcardCst::parse(...).prop::<LENS>()` / `.version()`
//! API. Provenance and licensing for every source repo live in its own corpus
//! directory under tests/corpus/ (see each project's ATTRIBUTION.md).

use std::borrow::Cow;

use vcard::{
    tree::{
        cst::VcardCst,
        prop::{r#fn::FN, n::N, nickname::NICKNAME, note::NOTE, org::ORG},
    },
    version::VcardVersion,
};

/// Collect a decoded component list into borrowed `&str`s for comparison.
fn strs<'a>(list: &'a [Cow<'a, str>]) -> Vec<&'a str> {
    list.iter().map(|c| c.as_ref()).collect()
}

// emersion/go-vcard (MIT), decoder_test.go / card_test.go: the "RFC" test card
// decodes to FN "J. Doe" and N family ["Doe"], given ["J."].
#[test]
fn emersion_rfc_card_fn_and_n() {
    let card = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "UID:urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1\r\n",
        "FN;PID=1.1:J. Doe\r\n",
        "N:Doe;J.;;;\r\n",
        "EMAIL;PID=1.1:jdoe@example.com\r\n",
        "CLIENTPIDMAP:1;urn:uuid:53e374d9-337e-4727-8803-a1e9c14e0556\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();

    assert_eq!(&*card.prop::<FN>().unwrap().0, "J. Doe");

    let n = card.prop::<N>().unwrap();
    assert_eq!(strs(&n.family), ["Doe"]);
    assert_eq!(strs(&n.given), ["J."]);
}

// emersion/go-vcard (MIT), decoder_test.go TestParseLine_escaped: the escaped
// NOTE value unescapes `\n` to newlines and `\,` to a literal comma.
#[test]
fn emersion_escaped_note_unescapes() {
    let card = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "NOTE:Mythical Manager\\nHyjinx Software Division\\nBabsCo\\, Inc.\\n\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();

    assert_eq!(
        &*card.prop::<NOTE>().unwrap().0,
        "Mythical Manager\nHyjinx Software Division\nBabsCo, Inc.\n",
    );
}

// Heymdall/vcard (MIT), test/vcard.parse.spec.js "Should parse line with
// complex properties": N:Gump;Forrest;;Mr.; decodes to the named components.
#[test]
fn heymdall_structured_name() {
    let card = VcardCst::parse("N:Gump;Forrest;;Mr.;").unwrap();
    let n = card.prop::<N>().unwrap();

    // Heymdall's expected value is ['Gump', 'Forrest', '', 'Mr.', ''], so the
    // empty middle and trailing components decode to a single empty string.
    assert_eq!(strs(&n.family), ["Gump"]);
    assert_eq!(strs(&n.given), ["Forrest"]);
    assert_eq!(strs(&n.additional), [""]);
    assert_eq!(strs(&n.prefixes), ["Mr."]);
    assert_eq!(strs(&n.suffixes), [""]);
}

// Heymdall/vcard (MIT), "Should parse props with semicolon-separated values":
// ORG:ABC\, Inc.;North American Division;Marketing decodes to three components,
// the first with its escaped comma unescaped.
#[test]
fn heymdall_org_components() {
    let card = VcardCst::parse("ORG:ABC\\, Inc.;North American Division;Marketing").unwrap();

    assert_eq!(
        strs(&card.prop::<ORG>().unwrap().0),
        ["ABC, Inc.", "North American Division", "Marketing"],
    );
}

// Heymdall/vcard (MIT), "Should parse props with comma-separated values":
// NICKNAME:Jim,Jimmie decodes to a two-item text list.
#[test]
fn heymdall_nickname_list() {
    let card = VcardCst::parse("NICKNAME:Jim,Jimmie").unwrap();

    assert_eq!(strs(&card.prop::<NICKNAME>().unwrap().0), ["Jim", "Jimmie"]);
}

// Heymdall/vcard (MIT), "Should parse simple vcard lines": FN:Forrest Gump.
#[test]
fn heymdall_formatted_name() {
    let card = VcardCst::parse("FN:Forrest Gump").unwrap();

    assert_eq!(&*card.prop::<FN>().unwrap().0, "Forrest Gump");
}

// nilclass/vcardjs (MIT), test/vcf.js sample8: a three-part name sets family,
// given and additional components.
#[test]
fn nilclass_three_part_name() {
    let card = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "N:Lessing;Gotthold;Ephraim;;\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();
    let n = card.prop::<N>().unwrap();

    assert_eq!(strs(&n.family), ["Lessing"]);
    assert_eq!(strs(&n.given), ["Gotthold"]);
    assert_eq!(strs(&n.additional), ["Ephraim"]);
}

// nilclass/vcardjs (MIT), test/vcf.js sample9: a complex name with multiple
// values per component decodes every component as a list.
#[test]
fn nilclass_complex_name() {
    let card = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "N:Lessing;Gotthold;Ephraim,Soundso;Dr.,Prof.;von und zu hier und da\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();
    let n = card.prop::<N>().unwrap();

    assert_eq!(strs(&n.family), ["Lessing"]);
    assert_eq!(strs(&n.given), ["Gotthold"]);
    assert_eq!(strs(&n.additional), ["Ephraim", "Soundso"]);
    assert_eq!(strs(&n.prefixes), ["Dr.", "Prof."]);
    assert_eq!(strs(&n.suffixes), ["von und zu hier und da"]);
}

// nilclass/vcardjs (MIT), test/vcf.js sample10: NICKNAME:foo,bar,baz decodes to
// a three-item text list.
#[test]
fn nilclass_nicknames() {
    let card = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "NICKNAME:foo,bar,baz\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();

    assert_eq!(
        strs(&card.prop::<NICKNAME>().unwrap().0),
        ["foo", "bar", "baz"],
    );
}

// nilclass/vcardjs (MIT), test/vcf.js sample6: the version attribute is read
// back as 4.0.
#[test]
fn nilclass_version() {
    let card = VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nEND:VCARD\r\n").unwrap();

    assert_eq!(card.version(), VcardVersion::V4_0);
}
