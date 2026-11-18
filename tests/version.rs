//! vCard 3.0 and 4.0, one written as the other.

#![cfg(feature = "parser")]

use vcard::{tree::cst::VcardCst, vcard::Vcard, version::VcardVersion};

fn convert(vcf: &str, version: VcardVersion) -> String {
    let cst = VcardCst::parse(vcf).expect("a card");
    let card: Vcard<'_> = cst.decode().to_version(version);
    VcardCst::from(card).to_string()
}

const IOS_GROUP: &str = concat!(
    "BEGIN:VCARD\r\n",
    "VERSION:3.0\r\n",
    "FN:Climbing club\r\n",
    "X-ADDRESSBOOKSERVER-KIND:group\r\n",
    "X-ADDRESSBOOKSERVER-MEMBER:urn:uuid:5AD380FD\r\n",
    "X-ADDRESSBOOKSERVER-MEMBER:urn:uuid:0C3A2B1D\r\n",
    "END:VCARD\r\n",
);

#[test]
fn reads_an_apple_group_as_a_kind_and_its_members() {
    let up = convert(IOS_GROUP, VcardVersion::V4_0);

    assert_eq!(
        up,
        concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:Climbing club\r\n",
            "KIND:group\r\n",
            "MEMBER:urn:uuid:5AD380FD\r\n",
            "MEMBER:urn:uuid:0C3A2B1D\r\n",
            "END:VCARD\r\n",
        )
    );
    assert_eq!(convert(&up, VcardVersion::V3_0), IOS_GROUP);
}

const IOS_CONTACT: &str = concat!(
    "BEGIN:VCARD\r\n",
    "VERSION:3.0\r\n",
    "FN:Camille\r\n",
    "EMAIL;TYPE=INTERNET,HOME,pref:camille@example.org\r\n",
    "item1.TEL;TYPE=CELL:+33 7 00 00 00 01\r\n",
    "item1.X-ABLabel:_$!<Mobile>!$_\r\n",
    "PHOTO;ENCODING=b;TYPE=JPEG:/9j/4AAQ\r\n",
    "LOGO;VALUE=uri:https://example.org/logo.png\r\n",
    "AGENT;VALUE=uri:mailto:assistant@example.org\r\n",
    "END:VCARD\r\n",
);

#[test]
fn writes_a_three_point_zero_card_as_four_point_zero_and_back() {
    let up = convert(IOS_CONTACT, VcardVersion::V4_0);

    assert_eq!(
        up,
        concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:Camille\r\n",
            "EMAIL;TYPE=INTERNET,HOME;PREF=1:camille@example.org\r\n",
            "item1.TEL;TYPE=CELL:+33 7 00 00 00 01\r\n",
            "item1.X-ABLabel:_$!<Mobile>!$_\r\n",
            "PHOTO:data:image/jpeg;base64,/9j/4AAQ\r\n",
            "LOGO:https://example.org/logo.png\r\n",
            "RELATED;TYPE=agent:mailto:assistant@example.org\r\n",
            "END:VCARD\r\n",
        )
    );

    let down = convert(&up, VcardVersion::V3_0);

    assert_eq!(
        down,
        concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:3.0\r\n",
            "FN:Camille\r\n",
            "EMAIL;TYPE=INTERNET,HOME,pref:camille@example.org\r\n",
            "item1.TEL;TYPE=CELL:+33 7 00 00 00 01\r\n",
            "item1.X-ABLabel:_$!<Mobile>!$_\r\n",
            "PHOTO;ENCODING=b;TYPE=JPEG:/9j/4AAQ\r\n",
            "LOGO;VALUE=uri:https://example.org/logo.png\r\n",
            "AGENT;VALUE=uri:mailto:assistant@example.org\r\n",
            "END:VCARD\r\n",
        )
    );
}

#[test]
fn drops_an_inline_agent_card_going_to_four_point_zero() {
    let vcf = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:3.0\r\n",
        "FN:Boss\r\n",
        "AGENT:BEGIN:VCARD\\nFN:Assistant\\nEND:VCARD\r\n",
        "END:VCARD\r\n",
    );

    assert_eq!(
        convert(vcf, VcardVersion::V4_0),
        "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Boss\r\nEND:VCARD\r\n"
    );
}

#[test]
fn keeps_a_pref_other_than_one_going_to_three_point_zero() {
    let vcf = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "FN:Bob\r\n",
        "EMAIL;PREF=1:a@example.org\r\n",
        "EMAIL;PREF=2:b@example.org\r\n",
        "END:VCARD\r\n",
    );

    assert_eq!(
        convert(vcf, VcardVersion::V3_0),
        concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:3.0\r\n",
            "FN:Bob\r\n",
            "EMAIL;TYPE=pref:a@example.org\r\n",
            "EMAIL;PREF=2:b@example.org\r\n",
            "END:VCARD\r\n",
        )
    );
}

#[test]
fn writes_a_gender_with_no_identity_as_its_sex_alone() {
    let vcf = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Bob\r\nGENDER:M\r\nEND:VCARD\r\n";
    let cst = VcardCst::parse(vcf).expect("a card");

    assert_eq!(VcardCst::from(cst.decode()).to_string(), vcf);
}
