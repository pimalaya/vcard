#![cfg(feature = "parser")]
//! Coverage-oriented exercises for the pure model surface: the value and
//! parameter vocabularies (round-trip and `Deref`), every value/parameter
//! variant's `kind`, the model newtype conversions, error `Display`, and a
//! maximal encode that drives every value kind and parameter through the codec.

use std::borrow::Cow;

use vcard::param::{VcardParam, VcardParamKind};
use vcard::prop::{VcardProp, VcardPropName};
use vcard::tree::cst::VcardCst;
use vcard::value::adr::VcardAdr;
use vcard::value::binary::VcardBinary;
use vcard::value::client_pid_map::VcardClientPidMap;
use vcard::value::datetime::{VcardDateAndOrTime, VcardTimestamp};
use vcard::value::gender::VcardGender;
use vcard::value::geo::VcardGeo;
use vcard::value::language::VcardLanguageTag;
use vcard::value::n::VcardN;
use vcard::value::org::VcardOrg;
use vcard::value::text::{VcardText, VcardTextList};
use vcard::value::uri::VcardUri;
use vcard::value::utc_offset::VcardUtcOffset;
use vcard::value::{VcardUnknownValue, VcardValue, VcardValueKind};
use vcard::vcard::Vcard;
use vcard::version::VcardVersion;

const VALUE_KINDS: [VcardValueKind; 14] = [
    VcardValueKind::Adr,
    VcardValueKind::Binary,
    VcardValueKind::ClientPidMap,
    VcardValueKind::DateAndOrTime,
    VcardValueKind::Gender,
    VcardValueKind::Geo,
    VcardValueKind::LanguageTag,
    VcardValueKind::N,
    VcardValueKind::Org,
    VcardValueKind::Text,
    VcardValueKind::TextList,
    VcardValueKind::Timestamp,
    VcardValueKind::Uri,
    VcardValueKind::UtcOffset,
];

const PARAM_KINDS: [VcardParamKind; 14] = [
    VcardParamKind::AltId,
    VcardParamKind::CalScale,
    VcardParamKind::Charset,
    VcardParamKind::Encoding,
    VcardParamKind::Geo,
    VcardParamKind::Label,
    VcardParamKind::Language,
    VcardParamKind::MediaType,
    VcardParamKind::Pid,
    VcardParamKind::Pref,
    VcardParamKind::SortAs,
    VcardParamKind::Type,
    VcardParamKind::Tz,
    VcardParamKind::Value,
];

#[test]
fn every_kind_derefs_to_its_wire_name_and_round_trips() {
    for kind in VALUE_KINDS {
        let name: &str = &kind;
        assert_eq!(name.parse::<VcardValueKind>().unwrap(), kind);
    }
    for kind in PARAM_KINDS {
        let name: &str = &kind;
        assert_eq!(name.parse::<VcardParamKind>().unwrap(), kind);
    }
}

#[test]
fn every_value_variant_reports_its_kind() {
    let values = [
        (
            VcardValue::Adr(VcardAdr::default()),
            Some(VcardValueKind::Adr),
        ),
        (
            VcardValue::Binary(VcardBinary::Uri(Cow::Borrowed("x"))),
            Some(VcardValueKind::Binary),
        ),
        (
            VcardValue::ClientPidMap(VcardClientPidMap::default()),
            Some(VcardValueKind::ClientPidMap),
        ),
        (
            VcardValue::DateAndOrTime(VcardDateAndOrTime::default()),
            Some(VcardValueKind::DateAndOrTime),
        ),
        (
            VcardValue::Gender(VcardGender::default()),
            Some(VcardValueKind::Gender),
        ),
        (
            VcardValue::Geo(VcardGeo::default()),
            Some(VcardValueKind::Geo),
        ),
        (
            VcardValue::LanguageTag(VcardLanguageTag::default()),
            Some(VcardValueKind::LanguageTag),
        ),
        (VcardValue::N(VcardN::default()), Some(VcardValueKind::N)),
        (
            VcardValue::Org(VcardOrg::default()),
            Some(VcardValueKind::Org),
        ),
        (
            VcardValue::Text(VcardText::default()),
            Some(VcardValueKind::Text),
        ),
        (
            VcardValue::TextList(VcardTextList::default()),
            Some(VcardValueKind::TextList),
        ),
        (
            VcardValue::Timestamp(VcardTimestamp::default()),
            Some(VcardValueKind::Timestamp),
        ),
        (
            VcardValue::Uri(VcardUri::default()),
            Some(VcardValueKind::Uri),
        ),
        (
            VcardValue::UtcOffset(VcardUtcOffset::default()),
            Some(VcardValueKind::UtcOffset),
        ),
        (VcardValue::Unknown(VcardUnknownValue::default()), None),
    ];

    for (value, kind) in values {
        assert_eq!(value.kind(), kind);
    }
}

#[test]
fn every_param_variant_reports_its_kind() {
    let params = [
        (
            VcardParam::AltId(Cow::Borrowed("1")),
            Some(VcardParamKind::AltId),
        ),
        (
            VcardParam::CalScale(Cow::Borrowed("gregorian")),
            Some(VcardParamKind::CalScale),
        ),
        (
            VcardParam::Charset(Cow::Borrowed("UTF-8")),
            Some(VcardParamKind::Charset),
        ),
        (
            VcardParam::Encoding(Cow::Borrowed("BASE64")),
            Some(VcardParamKind::Encoding),
        ),
        (
            VcardParam::Geo(Cow::Borrowed("x")),
            Some(VcardParamKind::Geo),
        ),
        (
            VcardParam::Label(Cow::Borrowed("x")),
            Some(VcardParamKind::Label),
        ),
        (
            VcardParam::Language(Cow::Borrowed("en")),
            Some(VcardParamKind::Language),
        ),
        (
            VcardParam::MediaType(Cow::Borrowed("text/plain")),
            Some(VcardParamKind::MediaType),
        ),
        (
            VcardParam::Pid(vec![Cow::Borrowed("1")]),
            Some(VcardParamKind::Pid),
        ),
        (
            VcardParam::Pref(Cow::Borrowed("1")),
            Some(VcardParamKind::Pref),
        ),
        (
            VcardParam::SortAs(vec![Cow::Borrowed("x")]),
            Some(VcardParamKind::SortAs),
        ),
        (
            VcardParam::Type(vec![Cow::Borrowed("home")]),
            Some(VcardParamKind::Type),
        ),
        (VcardParam::Tz(Cow::Borrowed("x")), Some(VcardParamKind::Tz)),
        (
            VcardParam::Value(Cow::Borrowed("text")),
            Some(VcardParamKind::Value),
        ),
        (
            VcardParam::Unknown {
                name: Cow::Borrowed("X-CUSTOM"),
                values: vec![Cow::Borrowed("v")],
            },
            None,
        ),
    ];

    for (param, kind) in params {
        assert_eq!(param.kind(), kind);
    }
}

#[test]
fn model_value_types_convert_from_owned_and_borrowed() {
    let owned = String::from("x");
    let cow: Cow<'static, str> = Cow::Owned(String::from("y"));
    let list = vec![Cow::Borrowed("a"), Cow::Borrowed("b")];

    let _ = VcardText::from(owned.clone());
    let _ = VcardText::from(cow.clone());
    let _ = VcardTextList::from(list.clone());
    let _ = VcardUri::from(owned.clone());
    let _ = VcardUri::from(cow.clone());
    let _ = VcardLanguageTag::from(owned.clone());
    let _ = VcardLanguageTag::from(cow.clone());
    let _ = VcardUtcOffset::from(owned.clone());
    let _ = VcardUtcOffset::from(cow.clone());
    let _ = VcardDateAndOrTime::from(owned.clone());
    let _ = VcardDateAndOrTime::from(cow.clone());
    let _ = VcardTimestamp::from(owned.clone());
    let _ = VcardTimestamp::from(cow.clone());
    let _ = VcardOrg::from(list);
}

#[test]
fn encodes_a_card_with_every_value_kind_and_parameter() {
    let value_props = [
        ("FN", VcardValue::Text(VcardText::from("John"))),
        (
            "NICKNAME",
            VcardValue::TextList(VcardTextList(vec![Cow::Borrowed("JD")])),
        ),
        ("URL", VcardValue::Uri(VcardUri::from("http://x"))),
        (
            "BDAY",
            VcardValue::DateAndOrTime(VcardDateAndOrTime::from("1980")),
        ),
        (
            "REV",
            VcardValue::Timestamp(VcardTimestamp::from("20200102T030405Z")),
        ),
        (
            "LANG",
            VcardValue::LanguageTag(VcardLanguageTag::from("en")),
        ),
        ("TZ", VcardValue::UtcOffset(VcardUtcOffset::from("-0500"))),
        (
            "N",
            VcardValue::N(VcardN {
                family: vec![Cow::Borrowed("Doe")],
                given: vec![Cow::Borrowed("John")],
                ..Default::default()
            }),
        ),
        (
            "ADR",
            VcardValue::Adr(VcardAdr {
                street: vec![Cow::Borrowed("Main St")],
                ..Default::default()
            }),
        ),
        (
            "GENDER",
            VcardValue::Gender(VcardGender {
                sex: Cow::Borrowed("M"),
                identity: Cow::Borrowed("man"),
            }),
        ),
        (
            "ORG",
            VcardValue::Org(VcardOrg(vec![Cow::Borrowed("Acme")])),
        ),
        (
            "GEO",
            VcardValue::Geo(VcardGeo {
                latitude: Cow::Borrowed("1.0"),
                longitude: Cow::Borrowed("2.0"),
            }),
        ),
        (
            "CLIENTPIDMAP",
            VcardValue::ClientPidMap(VcardClientPidMap {
                id: Cow::Borrowed("1"),
                uri: Cow::Borrowed("urn:x"),
            }),
        ),
        (
            "PHOTO",
            VcardValue::Binary(VcardBinary::Base64(Cow::Borrowed("Zm9v"))),
        ),
        (
            "KEY",
            VcardValue::Binary(VcardBinary::Uri(Cow::Borrowed("http://x"))),
        ),
        ("X-FOO", VcardValue::Unknown(VcardUnknownValue::default())),
    ];

    let all_params = vec![
        VcardParam::AltId(Cow::Borrowed("1")),
        VcardParam::CalScale(Cow::Borrowed("gregorian")),
        VcardParam::Charset(Cow::Borrowed("UTF-8")),
        VcardParam::Encoding(Cow::Borrowed("8BIT")),
        VcardParam::Geo(Cow::Borrowed("geo:1,2")),
        VcardParam::Label(Cow::Borrowed("addr")),
        VcardParam::Language(Cow::Borrowed("en")),
        VcardParam::MediaType(Cow::Borrowed("text/plain")),
        VcardParam::Pid(vec![Cow::Borrowed("1"), Cow::Borrowed("2")]),
        VcardParam::Pref(Cow::Borrowed("1")),
        VcardParam::SortAs(vec![Cow::Borrowed("Doe")]),
        VcardParam::Type(vec![Cow::Borrowed("home"), Cow::Borrowed("work")]),
        VcardParam::Tz(Cow::Borrowed("-0500")),
        VcardParam::Value(Cow::Borrowed("text")),
        VcardParam::Unknown {
            name: Cow::Borrowed("X-CUSTOM"),
            values: vec![Cow::Borrowed("v")],
        },
    ];

    let mut properties: Vec<VcardProp> = value_props
        .into_iter()
        .map(|(name, value)| VcardProp {
            name: VcardPropName::from(name),
            params: Vec::new(),
            value,
        })
        .collect();

    // One property carrying every parameter, to drive each param encoder.
    properties.push(VcardProp {
        name: VcardPropName::from("NOTE"),
        params: all_params,
        value: VcardValue::Text(VcardText::from("hi")),
    });

    // Encode across versions so the version-specific value shapes (GEO) run.
    for version in [VcardVersion::V2_1, VcardVersion::V3_0, VcardVersion::V4_0] {
        let card = Vcard {
            version,
            properties: properties.clone(),
        };
        let out = card.to_string();
        assert!(out.starts_with("BEGIN:VCARD"));
        assert!(out.trim_end().ends_with("END:VCARD"));

        // The From<Vcard> conversion (distinct from Display's encode path).
        let _ = VcardCst::from(card);
    }
}

#[test]
fn error_types_display() {
    use vcard::tree::error::VcardParseError;

    let parse_errors = [
        VcardParseError::MissingCrlf("x".into()),
        VcardParseError::MissingPropertyColon("x".into()),
        VcardParseError::NonUtf8Header("x".into()),
        VcardParseError::ExpectedBegin("x".into()),
        VcardParseError::MissingEnd("x".into()),
    ];
    for error in parse_errors {
        assert!(!error.to_string().is_empty());
    }

    assert!(
        "bogus"
            .parse::<VcardValueKind>()
            .unwrap_err()
            .to_string()
            .contains("bogus"),
    );
    assert!(
        "bogus"
            .parse::<VcardParamKind>()
            .unwrap_err()
            .to_string()
            .contains("bogus"),
    );
}

#[test]
fn every_property_kind_derefs_and_round_trips() {
    use vcard::prop::{VcardPropKind, VcardPropName};

    for kind in VcardPropKind::ALL {
        let name: &str = &kind;
        assert_eq!(name.parse::<VcardPropKind>().unwrap(), kind);
    }

    // VcardPropName: a known kind and a verbatim unknown name.
    assert_eq!(&*VcardPropName::from("FN"), "FN");
    assert_eq!(&*VcardPropName::from("X-FOO"), "X-FOO");
}

#[test]
fn exercises_every_property_lens_and_bespoke_cursor() {
    use vcard::tree::param::r#type::TYPE;
    use vcard::tree::prop::{
        adr::ADR, agent::AGENT, anniversary::ANNIVERSARY, bday::BDAY, caladruri::CALADRURI,
        caluri::CALURI, categories::CATEGORIES, class::CLASS, client_pid_map::CLIENTPIDMAP,
        email::EMAIL, fburl::FBURL, r#fn::FN, gender::GENDER, geo::GEO, impp::IMPP, key::KEY,
        kind::KIND, label::LABEL, lang::LANG, logo::LOGO, mailer::MAILER, member::MEMBER, n::N,
        name::NAME, nickname::NICKNAME, note::NOTE, org::ORG, photo::PHOTO, prodid::PRODID,
        profile::PROFILE, related::RELATED, rev::REV, role::ROLE, sort_string::SORT_STRING,
        sound::SOUND, source::SOURCE, tel::TEL, title::TITLE, tz::TZ, uid::UID, url::URL, xml::XML,
    };

    let raw = concat!(
        "BEGIN:VCARD\r\nVERSION:4.0\r\n",
        "ADR:;;St;City;;;US\r\n",
        "AGENT:x\r\n",
        "ANNIVERSARY:2020\r\n",
        "BDAY:1980\r\n",
        "CALADRURI:mailto:x\r\n",
        "CALURI:http://x\r\n",
        "CATEGORIES:a,b\r\n",
        "CLASS:PUBLIC\r\n",
        "CLIENTPIDMAP:1;urn:x\r\n",
        "EMAIL:a@b\r\n",
        "FBURL:http://x\r\n",
        "FN:John\r\n",
        "GENDER:M;man\r\n",
        "GEO:geo:1,2\r\n",
        "IMPP:xmpp:x\r\n",
        "KEY:http://x\r\n",
        "KIND:individual\r\n",
        "LABEL:addr\r\n",
        "LANG:en\r\n",
        "LOGO:http://x\r\n",
        "MAILER:x\r\n",
        "MEMBER:urn:x\r\n",
        "N:Doe;John;;;\r\n",
        "NAME:x\r\n",
        "NICKNAME:JD\r\n",
        "NOTE:hi\r\n",
        "ORG:Acme\r\n",
        "PHOTO:http://x\r\n",
        "PRODID:x\r\n",
        "PROFILE:VCARD\r\n",
        "RELATED:urn:x\r\n",
        "REV:20200102T030405Z\r\n",
        "ROLE:Eng\r\n",
        "SORT-STRING:doe\r\n",
        "SOUND:http://x\r\n",
        "SOURCE:http://x\r\n",
        "TEL:123\r\n",
        "TITLE:Boss\r\n",
        "TZ:-0500\r\n",
        "UID:x\r\n",
        "URL:http://x\r\n",
        "XML:<x/>\r\n",
        "END:VCARD\r\n",
    );
    let mut cst = VcardCst::parse(raw).unwrap();

    macro_rules! read {
        ($($m:ty),+ $(,)?) => {{ $( let _ = cst.prop::<$m>(); )+ }};
    }
    macro_rules! edit {
        ($($m:ty),+ $(,)?) => {{ $( let _ = cst.prop_mut::<$m>(); )+ }};
    }

    read!(
        ADR,
        AGENT,
        ANNIVERSARY,
        BDAY,
        CALADRURI,
        CALURI,
        CATEGORIES,
        CLASS,
        CLIENTPIDMAP,
        EMAIL,
        FBURL,
        FN,
        GENDER,
        GEO,
        IMPP,
        KEY,
        KIND,
        LABEL,
        LANG,
        LOGO,
        MAILER,
        MEMBER,
        N,
        NAME,
        NICKNAME,
        NOTE,
        ORG,
        PHOTO,
        PRODID,
        PROFILE,
        RELATED,
        REV,
        ROLE,
        SORT_STRING,
        SOUND,
        SOURCE,
        TEL,
        TITLE,
        TZ,
        UID,
        URL,
        XML,
    );
    edit!(
        AGENT,
        ANNIVERSARY,
        BDAY,
        CALADRURI,
        CALURI,
        CATEGORIES,
        CLASS,
        EMAIL,
        FBURL,
        FN,
        GEO,
        IMPP,
        KEY,
        KIND,
        LABEL,
        LANG,
        LOGO,
        MAILER,
        MEMBER,
        NAME,
        NICKNAME,
        NOTE,
        ORG,
        PHOTO,
        PRODID,
        PROFILE,
        RELATED,
        REV,
        ROLE,
        SORT_STRING,
        SOUND,
        SOURCE,
        TEL,
        TITLE,
        TZ,
        UID,
        URL,
        XML,
    );

    // Bespoke cursors: exercise every named component getter and setter.
    {
        let mut c = cst.prop_mut::<ADR>().unwrap();
        let _ = c.get();
        let _ = c.po_box();
        let _ = c.extended();
        let _ = c.street();
        let _ = c.locality();
        let _ = c.region();
        let _ = c.postal_code();
        let _ = c.country();
        let _ = c.param::<TYPE>();
        c.set_po_box(&["P"]);
        c.set_extended(&["E"]);
        c.set_street(&["S"]);
        c.set_locality(&["L"]);
        c.set_region(&["R"]);
        c.set_postal_code(&["Z"]);
        c.set_country(&["US"]);
    }
    {
        let mut c = cst.prop_mut::<N>().unwrap();
        let _ = c.get();
        let _ = c.family();
        let _ = c.given();
        let _ = c.additional();
        let _ = c.prefixes();
        let _ = c.suffixes();
        let _ = c.param::<TYPE>();
        c.set_family(&["F"]);
        c.set_given(&["G"]);
        c.set_additional(&["A"]);
        c.set_prefixes(&["P"]);
        c.set_suffixes(&["S"]);
    }
    {
        let mut c = cst.prop_mut::<GENDER>().unwrap();
        let _ = c.get();
        let _ = c.sex();
        let _ = c.identity();
        let _ = c.param::<TYPE>();
        c.set_sex("M");
        c.set_identity("man");
    }
    {
        let mut c = cst.prop_mut::<CLIENTPIDMAP>().unwrap();
        let _ = c.get();
        let _ = c.id();
        let _ = c.uri();
        let _ = c.param::<TYPE>();
        c.set_id("1");
        c.set_uri("urn:y");
    }
}

#[test]
fn exercises_every_parameter_lens() {
    use vcard::tree::param::VcardParamLens;
    use vcard::tree::param::{
        altid::ALTID, calscale::CALSCALE, geo::GEO as GEOP, label::LABEL as LABELP,
        language::LANGUAGE, mediatype::MEDIATYPE, pid::PID, pref::PREF, sort_as::SORT_AS,
        r#type::TYPE, tz::TZ as TZP, value::VALUE,
    };
    use vcard::tree::prop::tel::TEL;

    let raw = concat!(
        "BEGIN:VCARD\r\nVERSION:4.0\r\n",
        "TEL;ALTID=1;CALSCALE=gregorian;GEO=x;LABEL=addr;LANGUAGE=en;",
        "MEDIATYPE=text/plain;PID=1,2;PREF=1;SORT-AS=Doe;TYPE=home,work;TZ=-0500;VALUE=text:123\r\n",
        "END:VCARD\r\n",
    );
    let mut cst = VcardCst::parse(raw).unwrap();

    // decode: read each parameter through its lens.
    let c = cst.prop_mut::<TEL>().unwrap();
    let _ = c.param::<ALTID>();
    let _ = c.param::<CALSCALE>();
    let _ = c.param::<GEOP>();
    let _ = c.param::<LABELP>();
    let _ = c.param::<LANGUAGE>();
    let _ = c.param::<MEDIATYPE>();
    let _ = c.param::<PID>();
    let _ = c.param::<PREF>();
    let _ = c.param::<SORT_AS>();
    let _ = c.param::<TYPE>();
    let _ = c.param::<TZP>();
    let _ = c.param::<VALUE>();

    // encode: mint a parameter node from a decoded value through each lens.
    let scalar: Cow<'static, str> = Cow::Borrowed("x");
    let list = vec![Cow::Borrowed("x")];
    let _ = ALTID::encode(&scalar);
    let _ = CALSCALE::encode(&scalar);
    let _ = GEOP::encode(&scalar);
    let _ = LABELP::encode(&scalar);
    let _ = LANGUAGE::encode(&scalar);
    let _ = MEDIATYPE::encode(&scalar);
    let _ = PID::encode(&list);
    let _ = PREF::encode(&scalar);
    let _ = SORT_AS::encode(&list);
    let _ = TYPE::encode(&list);
    let _ = TZP::encode(&scalar);
    let _ = VALUE::encode(&scalar);
}

#[test]
fn decodes_a_card_covering_every_value_kind_and_parameter() {
    // A 2.1 card so GEO is a pair and PHOTO is inline binary; every value kind
    // and (on the NOTE line) every parameter is present, so the whole-card
    // decode dispatch and the parameter decode match both run every arm.
    let raw = concat!(
        "BEGIN:VCARD\r\nVERSION:2.1\r\n",
        "FN:John\r\n",
        "NICKNAME:a,b\r\n",
        "URL:http://x\r\n",
        "BDAY:1980\r\n",
        "REV:20200102T030405Z\r\n",
        "LANG:en\r\n",
        "TZ;VALUE=utc-offset:-0500\r\n",
        "N:Doe;John;;;\r\n",
        "ADR:;;St;City;;;US\r\n",
        "GENDER:M;man\r\n",
        "ORG:Acme;Div\r\n",
        "CLIENTPIDMAP:1;urn:x\r\n",
        "GEO:1.0,2.0\r\n",
        "PHOTO;ENCODING=BASE64:Zm9v\r\n",
        "X-FOO:z\r\n",
        "NOTE;TYPE=home;MEDIATYPE=text/plain;CALSCALE=gregorian;SORT-AS=Doe;GEO=g;",
        "PID=1;PREF=1;ALTID=1;LANGUAGE=en;LABEL=addr;TZ=-0500;VALUE=text:hi\r\n",
        "END:VCARD\r\n",
    );
    let cst = VcardCst::parse(raw).unwrap();
    // Serialize the multi-component values (N, ADR, GEO) through write_bytes.
    let _ = cst.to_bytes();
    let card = cst.decode();
    assert!(card.properties.len() >= 15);
}

#[test]
fn covers_prop_name_conversions_and_errors() {
    use vcard::prop::{VcardPropKind, VcardPropName};

    // ParseVcardPropKindError Display.
    assert!(
        "X-BOGUS"
            .parse::<VcardPropKind>()
            .unwrap_err()
            .to_string()
            .contains("X-BOGUS"),
    );

    // From<&VcardPropKind> for VcardPropName.
    let _ = VcardPropName::from(&VcardPropKind::Fn);
}

#[test]
fn rejects_a_card_without_an_end_line() {
    assert!(VcardCst::parse("BEGIN:VCARD\r\nFN:x\r\n").is_err());
}

#[test]
fn set_bytes_at_pads_missing_components() {
    // Writing a component beyond the current count pads the gap (the raw-bytes
    // structured-value path).
    let mut cst = VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:x\r\nEND:VCARD\r\n").unwrap();
    let seed: &[&[u8]] = &[b"y"];
    cst.props[1].value.set_bytes_at(3, seed);
    assert!(cst.to_bytes().windows(5).any(|w| w == b"x;;;y"));
}

#[test]
fn decodes_a_4_0_date_and_geo_uri() {
    // BDAY decodes to a date-and-or-time, and a 4.0 GEO to a URI whose comma is
    // rejoined (not split), exercising both decode paths.
    let raw = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:X\r\nBDAY:19800101\r\nGEO:geo:1,2\r\nEND:VCARD\r\n";
    let cst = VcardCst::parse(raw).unwrap();
    let card = cst.decode();

    assert!(matches!(
        card.properties[1].value,
        VcardValue::DateAndOrTime(_)
    ));
    assert!(matches!(card.properties[2].value, VcardValue::Uri(_)));
}

#[test]
fn decodes_a_value_node_through_the_value_codec_fallback() {
    use vcard::tree::codec::Codec;
    use vcard::tree::value::VcardValueNode;

    // The VcardValue Codec impl is the liberal raw fallback the divergent lenses
    // inherit; exercise it directly.
    let node = VcardValueNode::parse(b"a;b,c");
    let value = <VcardValue as Codec>::decode(&node);
    assert!(matches!(value, VcardValue::Unknown(_)));
}
