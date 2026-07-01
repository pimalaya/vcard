#![cfg(feature = "parser")]
//! Tests focused on content encodings, driven through the public API. Two
//! things are checked: that the core keeps the `CHARSET` / `ENCODING` metadata
//! and transforms no content (the value stays raw), and that the opt-in feature
//! helpers (`quoted-printable`, `base64`, `encoding`) decode it correctly.

use std::borrow::Cow;

use vcard::param::VcardParam;
use vcard::tree::cst::VcardCst;
use vcard::tree::prop::note::NOTE;
use vcard::value::VcardValue;

// --- core: encoding metadata is kept, content is not transformed ---

#[test]
fn charset_parameter_survives_on_the_decoded_model() {
    let cst = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:2.1\r\n",
        "NOTE;CHARSET=ISO-8859-1:hello\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();
    let card = cst.decode();

    assert!(
        card.properties[0]
            .params
            .contains(&VcardParam::Charset(Cow::Borrowed("ISO-8859-1"))),
    );
}

#[test]
fn quoted_printable_is_kept_raw_and_its_param_preserved() {
    let mut card = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:2.1\r\n",
        "NOTE;ENCODING=QUOTED-PRINTABLE:caf=C3=A9\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();

    // The core does not resolve QP: the raw wire value survives untouched.
    assert_eq!(
        card.prop_mut::<NOTE>().unwrap().bytes().as_ref(),
        b"caf=C3=A9"
    );

    // ...and the ENCODING parameter is kept (not dropped) so a consumer knows to
    // decode it.
    assert!(
        card.decode().properties[0]
            .params
            .contains(&VcardParam::Encoding(Cow::Borrowed("QUOTED-PRINTABLE"))),
    );
}

#[test]
fn a_non_utf8_charset_value_round_trips_byte_for_byte() {
    // A vCard 2.1 NOTE in ISO-8859-1: 0xE9 ('é' in Latin-1) is not valid UTF-8.
    let mut raw = Vec::new();
    raw.extend_from_slice(b"BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE;CHARSET=ISO-8859-1:caf");
    raw.push(0xE9);
    raw.extend_from_slice(b"\r\nEND:VCARD\r\n");

    let card = VcardCst::parse(&raw).unwrap();
    assert_eq!(card.to_bytes(), raw);
}

#[test]
fn set_bytes_writes_a_raw_foreign_charset_value() {
    let mut card =
        VcardCst::parse("BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE;CHARSET=ISO-8859-1:x\r\nEND:VCARD\r\n")
            .unwrap();

    // "café" in ISO-8859-1: the trailing 0xE9 is not valid UTF-8.
    let latin1 = [b'c', b'a', b'f', 0xE9];
    card.prop_mut::<NOTE>().unwrap().set_bytes(latin1);

    assert_eq!(card.prop_mut::<NOTE>().unwrap().bytes().as_ref(), &latin1);
    assert!(card.to_bytes().windows(4).any(|window| window == latin1));
}

// --- quoted-printable feature ---

#[cfg(feature = "quoted-printable")]
#[test]
fn quoted_printable_helper_resolves_utf8_octets() {
    let mut card = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:2.1\r\n",
        "NOTE;ENCODING=QUOTED-PRINTABLE:caf=C3=A9\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();

    let bytes = card.prop_mut::<NOTE>().unwrap().quoted_printable();
    assert_eq!(String::from_utf8(bytes).unwrap(), "café");
}

#[cfg(feature = "quoted-printable")]
#[test]
fn quoted_printable_helper_is_a_no_op_without_the_encoding() {
    let mut card =
        VcardCst::parse("BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE:plain=text\r\nEND:VCARD\r\n").unwrap();

    // No ENCODING=QUOTED-PRINTABLE, so `=` is a literal, not an octet escape.
    let bytes = card.prop_mut::<NOTE>().unwrap().quoted_printable();
    assert_eq!(bytes, b"plain=text");
}

// --- encoding (charset) feature ---

#[cfg(feature = "encoding")]
#[test]
fn charset_helper_transcodes_iso_8859_1() {
    let mut raw = Vec::new();
    raw.extend_from_slice(b"BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE;CHARSET=ISO-8859-1:caf");
    raw.push(0xE9);
    raw.extend_from_slice(b"\r\nEND:VCARD\r\n");

    let mut card = VcardCst::parse(&raw).unwrap();
    assert_eq!(card.prop_mut::<NOTE>().unwrap().charset(), "café");
}

#[cfg(feature = "encoding")]
#[test]
fn charset_helper_transcodes_windows_1252() {
    // 0x80 is the euro sign in windows-1252 (unmapped in ISO-8859-1).
    let mut raw = Vec::new();
    raw.extend_from_slice(b"BEGIN:VCARD\r\nVERSION:2.1\r\nNOTE;CHARSET=windows-1252:price ");
    raw.push(0x80);
    raw.extend_from_slice(b"\r\nEND:VCARD\r\n");

    let mut card = VcardCst::parse(&raw).unwrap();
    assert_eq!(card.prop_mut::<NOTE>().unwrap().charset(), "price €");
}

#[cfg(feature = "encoding")]
#[test]
fn charset_helper_defaults_to_utf8_without_a_charset_param() {
    let mut card =
        VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:café\r\nEND:VCARD\r\n").unwrap();

    assert_eq!(card.prop_mut::<NOTE>().unwrap().charset(), "café");
}

#[cfg(all(feature = "encoding", feature = "quoted-printable"))]
#[test]
fn charset_helper_composes_quoted_printable_then_transcodes() {
    // The real 2.1 foreign form: quoted-printable octets in a foreign charset.
    let mut card = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:2.1\r\n",
        "NOTE;CHARSET=ISO-8859-1;ENCODING=QUOTED-PRINTABLE:caf=E9\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();

    assert_eq!(card.prop_mut::<NOTE>().unwrap().charset(), "café");
}

// --- base64 feature ---

#[cfg(feature = "base64")]
#[test]
fn base64_helper_decodes_inline_binary() {
    // A 2.1 PHOTO with ENCODING=BASE64 decodes to an inline binary value.
    let cst = VcardCst::parse(concat!(
        "BEGIN:VCARD\r\n",
        "VERSION:2.1\r\n",
        "PHOTO;ENCODING=BASE64:Zm9vYmFy\r\n",
        "END:VCARD\r\n",
    ))
    .unwrap();
    let card = cst.decode();

    match &card.properties[0].value {
        VcardValue::Binary(binary) => {
            assert_eq!(binary.decode_base64().unwrap().unwrap(), b"foobar");
        }
        other => panic!("expected an inline binary value, got {other:?}"),
    }
}
