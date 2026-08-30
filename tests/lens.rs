#![cfg(feature = "parser")]
//! vCard 4.0 lens isolation: editing one property's value through its typed
//! cursor must leave every other byte of the card untouched. Each test asserts
//! the whole serialization equals the input with exactly one line swapped, so a
//! setter that bled into another component, parameter or line would fail.

use vcard::{
    prop::{
        adr::ADR, bday::BDAY, client_pid_map::CLIENTPIDMAP, r#fn::FN, gender::GENDER, lang::LANG,
        n::N, nickname::NICKNAME, org::ORG, rev::REV, tz::TZ, url::URL,
    },
    tree::cst::VcardCst,
};

const CARD: &str = concat!(
    "BEGIN:VCARD\r\n",
    "VERSION:4.0\r\n",
    "FN:John\r\n",
    "NICKNAME:Nick\r\n",
    "URL:http://a.test/\r\n",
    "BDAY:19800102\r\n",
    "REV:20200102T030405Z\r\n",
    "TZ:-0500\r\n",
    "N:Doe;John;;;\r\n",
    "ADR:;;Old St;;;;\r\n",
    "ORG:Acme;Widgets\r\n",
    "GENDER:M;male\r\n",
    "CLIENTPIDMAP:1;urn:uuid:aaa\r\n",
    "LANG:en\r\n",
    "END:VCARD\r\n",
);

/// Parse `CARD`, apply `edit`, and assert the only change is `old` -> `new`.
fn check(old: &str, new: &str, edit: impl FnOnce(&mut VcardCst)) {
    let mut cst = VcardCst::parse(CARD).unwrap();
    edit(&mut cst);
    assert_eq!(
        cst.to_string(),
        CARD.replacen(old, new, 1),
        "editing `{old}` was not isolated"
    );
}

#[test]
fn card_round_trips_unmodified() {
    assert_eq!(VcardCst::parse(CARD).unwrap().to_string(), CARD);
}

#[test]
fn text() {
    check("FN:John", "FN:Jane", |c| {
        c.prop_mut::<FN>().unwrap().set_text("Jane");
    });
}

#[test]
fn text_list() {
    check("NICKNAME:Nick", "NICKNAME:A,B", |c| {
        c.prop_mut::<NICKNAME>().unwrap().set_list(&["A", "B"]);
    });
}

#[test]
fn uri() {
    check("URL:http://a.test/", "URL:http://b.test/", |c| {
        c.prop_mut::<URL>().unwrap().set_text("http://b.test/");
    });
}

#[test]
fn date_and_or_time() {
    check("BDAY:19800102", "BDAY:19901112", |c| {
        c.prop_mut::<BDAY>().unwrap().set_text("19901112");
    });
}

#[test]
fn timestamp() {
    check("REV:20200102T030405Z", "REV:20211213T141516Z", |c| {
        c.prop_mut::<REV>().unwrap().set_text("20211213T141516Z");
    });
}

#[test]
fn utc_offset() {
    check("TZ:-0500", "TZ:-0400", |c| {
        c.prop_mut::<TZ>().unwrap().set_text("-0400");
    });
}

#[test]
fn structured_n() {
    check("N:Doe;John;;;", "N:Doe;Jane;;;", |c| {
        c.prop_mut::<N>().unwrap().set_given(&["Jane"]);
    });
}

#[test]
fn structured_adr() {
    check("ADR:;;Old St;;;;", "ADR:;;New St;;;;", |c| {
        c.prop_mut::<ADR>().unwrap().set_street(&["New St"]);
    });
}

#[test]
fn org() {
    check("ORG:Acme;Widgets", "ORG:NewCo;Widgets", |c| {
        c.prop_mut::<ORG>().unwrap().set_component(0, &["NewCo"]);
    });
}

#[test]
fn structured_gender() {
    check("GENDER:M;male", "GENDER:F;male", |c| {
        c.prop_mut::<GENDER>().unwrap().set_sex("F");
    });
}

#[test]
fn structured_client_pid_map() {
    check(
        "CLIENTPIDMAP:1;urn:uuid:aaa",
        "CLIENTPIDMAP:2;urn:uuid:aaa",
        |c| {
            c.prop_mut::<CLIENTPIDMAP>().unwrap().set_id("2");
        },
    );
}

#[test]
fn language_tag() {
    check("LANG:en", "LANG:fr", |c| {
        c.prop_mut::<LANG>().unwrap().set_text("fr");
    });
}

/// Every `ADR` component, in the order RFC 6350 6.3.1 and RFC 9554 3.1 lay them
/// out, filled with its own index so a reader off by one reads a neighbour.
const ADR_COMPONENTS: [&str; 18] = [
    "c0-po-box",
    "c1-extended",
    "c2-street",
    "c3-locality",
    "c4-region",
    "c5-postal-code",
    "c6-country",
    "c7-room",
    "c8-apartment",
    "c9-floor",
    "c10-street-number",
    "c11-street-name",
    "c12-building",
    "c13-block",
    "c14-subdistrict",
    "c15-district",
    "c16-landmark",
    "c17-direction",
];

#[test]
fn every_adr_component_reads_its_own_slot() {
    let card = format!(
        "BEGIN:VCARD\r\nVERSION:4.0\r\nADR:{}\r\nEND:VCARD\r\n",
        ADR_COMPONENTS.join(";")
    );
    let mut cst = VcardCst::parse(&card).unwrap();
    let adr = cst.prop_mut::<ADR>().unwrap();

    let read = [
        adr.po_box(),
        adr.extended(),
        adr.street(),
        adr.locality(),
        adr.region(),
        adr.postal_code(),
        adr.country(),
        adr.room(),
        adr.apartment(),
        adr.floor(),
        adr.street_number(),
        adr.street_name(),
        adr.building(),
        adr.block(),
        adr.subdistrict(),
        adr.district(),
        adr.landmark(),
        adr.direction(),
    ];

    for (component, expected) in read.iter().zip(ADR_COMPONENTS) {
        assert_eq!(component.as_slice(), [expected], "{expected} is misread");
    }
}

#[test]
fn every_adr_component_writes_its_own_slot() {
    let blank = ";".repeat(17);
    let card = format!("BEGIN:VCARD\r\nVERSION:4.0\r\nADR:{blank}\r\nEND:VCARD\r\n");
    let mut cst = VcardCst::parse(&card).unwrap();

    {
        let mut adr = cst.prop_mut::<ADR>().unwrap();
        adr.set_po_box(&[ADR_COMPONENTS[0]]);
        adr.set_extended(&[ADR_COMPONENTS[1]]);
        adr.set_street(&[ADR_COMPONENTS[2]]);
        adr.set_locality(&[ADR_COMPONENTS[3]]);
        adr.set_region(&[ADR_COMPONENTS[4]]);
        adr.set_postal_code(&[ADR_COMPONENTS[5]]);
        adr.set_country(&[ADR_COMPONENTS[6]]);
        adr.set_room(&[ADR_COMPONENTS[7]]);
        adr.set_apartment(&[ADR_COMPONENTS[8]]);
        adr.set_floor(&[ADR_COMPONENTS[9]]);
        adr.set_street_number(&[ADR_COMPONENTS[10]]);
        adr.set_street_name(&[ADR_COMPONENTS[11]]);
        adr.set_building(&[ADR_COMPONENTS[12]]);
        adr.set_block(&[ADR_COMPONENTS[13]]);
        adr.set_subdistrict(&[ADR_COMPONENTS[14]]);
        adr.set_district(&[ADR_COMPONENTS[15]]);
        adr.set_landmark(&[ADR_COMPONENTS[16]]);
        adr.set_direction(&[ADR_COMPONENTS[17]]);
    }

    assert_eq!(
        cst.to_string(),
        format!(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nADR:{}\r\nEND:VCARD\r\n",
            ADR_COMPONENTS.join(";")
        ),
    );
}

/// A card serializes back through `to_bytes` as well as through `Display`.
///
/// The two are separate paths: `to_bytes` writes into one buffer without the
/// intermediate `String`, and its multi-value parameter branch (the `,` between
/// a parameter's values) had nothing driving it.
#[test]
fn a_multi_valued_parameter_survives_the_byte_serializer() {
    let card = concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:4.0\r\n",
        "TEL;TYPE=work,voice;PID=1,2:+1\r\n",
        "END:VCARD\r\n",
    );
    let cst = VcardCst::parse(card).unwrap();

    assert_eq!(cst.to_bytes(), card.as_bytes());
    assert_eq!(cst.to_string(), card);
}
