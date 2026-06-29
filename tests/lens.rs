#![cfg(feature = "parser")]
//! vCard 4.0 lens isolation: editing one property's value through its typed
//! cursor must leave every other byte of the card untouched. Each test asserts
//! the whole serialization equals the input with exactly one line swapped, so a
//! setter that bled into another component, parameter or line would fail.

use vcard::tree::cst::VcardCst;
use vcard::tree::prop::{
    adr::ADR, bday::BDAY, client_pid_map::CLIENTPIDMAP, r#fn::FN, gender::GENDER, lang::LANG, n::N,
    nickname::NICKNAME, org::ORG, rev::REV, tz::TZ, url::URL,
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
