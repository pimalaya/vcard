#![cfg(feature = "parser")]
//! Coverage-oriented exercises for the pure model surface: the value and
//! parameter vocabularies (round-trip and `Deref`), every value/parameter
//! variant's `kind`, the model newtype conversions, error `Display`, and a
//! maximal encode that drives every value kind and parameter through the codec.

use std::borrow::Cow;

use vcard::{
    param::{VcardParam, VcardParamKind},
    prop::{VcardProp, VcardPropName},
    tree::codec::mode::VcardEscaper,
    tree::cst::VcardCst,
    value::{
        VcardValue, VcardValueKind, VcardValueUnknown,
        adr::VcardAdr,
        binary::VcardBinary,
        client_pid_map::VcardClientPidMap,
        datetime::{VcardDateAndOrTime, VcardTimestamp},
        gender::VcardGender,
        geo::VcardGeo,
        language::VcardLanguageTag,
        n::VcardN,
        org::VcardOrg,
        text::{VcardText, VcardTextList},
        uri::VcardUri,
        utc_offset::VcardUtcOffset,
    },
    vcard::Vcard,
    version::VcardVersion,
};

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

const PARAM_KINDS: [VcardParamKind; 24] = [
    VcardParamKind::AltId,
    VcardParamKind::Author,
    VcardParamKind::AuthorName,
    VcardParamKind::CalScale,
    VcardParamKind::Charset,
    VcardParamKind::Created,
    VcardParamKind::Derived,
    VcardParamKind::Encoding,
    VcardParamKind::Geo,
    VcardParamKind::Jsptr,
    VcardParamKind::Label,
    VcardParamKind::Language,
    VcardParamKind::MediaType,
    VcardParamKind::Phonetic,
    VcardParamKind::Pid,
    VcardParamKind::Pref,
    VcardParamKind::PropId,
    VcardParamKind::Script,
    VcardParamKind::ServiceType,
    VcardParamKind::SortAs,
    VcardParamKind::Type,
    VcardParamKind::Tz,
    VcardParamKind::Username,
    VcardParamKind::Value,
];

/// The wire name each parameter kind is spelled with, stated here rather than
/// read back from the kind's own `Deref`.
///
/// The match is exhaustive on purpose: a kind added to the vocabulary and left
/// out of `PARAM_KINDS` fails to compile rather than escaping every sweep.
fn param_wire_name(kind: VcardParamKind) -> &'static str {
    match kind {
        VcardParamKind::AltId => "ALTID",
        VcardParamKind::Author => "AUTHOR",
        VcardParamKind::AuthorName => "AUTHOR-NAME",
        VcardParamKind::CalScale => "CALSCALE",
        VcardParamKind::Charset => "CHARSET",
        VcardParamKind::Created => "CREATED",
        VcardParamKind::Derived => "DERIVED",
        VcardParamKind::Encoding => "ENCODING",
        VcardParamKind::Geo => "GEO",
        VcardParamKind::Jsptr => "JSPTR",
        VcardParamKind::Label => "LABEL",
        VcardParamKind::Language => "LANGUAGE",
        VcardParamKind::MediaType => "MEDIATYPE",
        VcardParamKind::Phonetic => "PHONETIC",
        VcardParamKind::Pid => "PID",
        VcardParamKind::Pref => "PREF",
        VcardParamKind::PropId => "PROP-ID",
        VcardParamKind::Script => "SCRIPT",
        VcardParamKind::ServiceType => "SERVICE-TYPE",
        VcardParamKind::SortAs => "SORT-AS",
        VcardParamKind::Type => "TYPE",
        VcardParamKind::Tz => "TZ",
        VcardParamKind::Username => "USERNAME",
        VcardParamKind::Value => "VALUE",
    }
}

#[test]
fn every_kind_derefs_to_its_wire_name_and_round_trips() {
    for kind in VALUE_KINDS {
        let name: &str = &kind;
        assert_eq!(name.parse::<VcardValueKind>().unwrap(), kind);
    }
    for kind in PARAM_KINDS {
        let name: &str = &kind;
        assert_eq!(name, param_wire_name(kind), "{name} is spelled wrong");
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
        (VcardValue::Unknown(VcardValueUnknown::default()), None),
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
        ("X-FOO", VcardValue::Unknown(VcardValueUnknown::default())),
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

    // Encode across versions so the version-specific value shapes (GEO) run:
    // 2.1 writes the coordinate pair `lat,long`, 3.0 writes `lat;long`, and 4.0
    // carries GEO as a URI instead (RFC 6350 6.5.2), which is why the value
    // itself, not just its encoding, changes with the version.
    // NOTE: The 4.0 URI is comma-free on purpose. A URI carrying a comma, which
    // a `geo:` URI does, comes back out escaped as `geo:1.0\,2.0`: the encoder
    // runs a URI through the text escaper, which RFC 6350 4.2 does not call
    // for. Pinning that here would settle a question that is still open.
    let cases = [
        (VcardVersion::V2_1, "2.1", "GEO:1.0,2.0\r\n"),
        (VcardVersion::V3_0, "3.0", "GEO:1.0;2.0\r\n"),
        (
            VcardVersion::V4_0,
            "4.0",
            "GEO:https://example.com/where\r\n",
        ),
    ];

    for (version, wire_version, geo) in cases {
        let properties: Vec<VcardProp> = properties
            .iter()
            .cloned()
            .map(|prop| match (version, &*prop.name) {
                (VcardVersion::V4_0, "GEO") => VcardProp {
                    value: VcardValue::Uri(VcardUri::from("https://example.com/where")),
                    ..prop
                },
                _ => prop,
            })
            .collect();

        let card = Vcard {
            version,
            properties,
        };

        // NOTE: The whole card, not a BEGIN/END sanity check: every value kind
        // above encodes to a shape this pins, so a value that silently encoded
        // to nothing would still have passed before.
        let expected = alloc_expected(wire_version, geo);
        assert_eq!(card.to_string(), expected, "{wire_version} does not encode");

        // The From<Vcard> conversion (distinct from Display's encode path).
        assert_eq!(VcardCst::from(card).to_string(), expected);
    }
}

/// The wire text [`encodes_a_card_with_every_value_kind_and_parameter`]
/// expects, differing between versions only in the `VERSION` and `GEO` lines.
fn alloc_expected(version: &str, geo: &str) -> String {
    let head = concat!(
        "FN:John\r\n",
        "NICKNAME:JD\r\n",
        "URL:http://x\r\n",
        "BDAY:1980\r\n",
        "REV:20200102T030405Z\r\n",
        "LANG:en\r\n",
        "TZ:-0500\r\n",
        "N:Doe;John;;;\r\n",
        "ADR:;;Main St;;;;\r\n",
        "GENDER:M;man\r\n",
        "ORG:Acme\r\n",
    );
    let tail = concat!(
        "CLIENTPIDMAP:1;urn:x\r\n",
        "PHOTO:Zm9v\r\n",
        "KEY:http://x\r\n",
        "X-FOO:\r\n",
        "NOTE;ALTID=1;CALSCALE=gregorian;CHARSET=UTF-8;ENCODING=8BIT;GEO=geo:1,2;",
        "LABEL=addr;LANGUAGE=en;MEDIATYPE=text/plain;PID=1,2;PREF=1;SORT-AS=Doe;",
        "TYPE=home,work;TZ=-0500;VALUE=text;X-CUSTOM=v:hi\r\n",
        "END:VCARD\r\n",
    );

    format!("BEGIN:VCARD\r\nVERSION:{version}\r\n{head}{geo}{tail}")
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
    use vcard::prop::{
        adr::ADR, agent::AGENT, anniversary::ANNIVERSARY, bday::BDAY, caladruri::CALADRURI,
        caluri::CALURI, categories::CATEGORIES, class::CLASS, client_pid_map::CLIENTPIDMAP,
        created::CREATED, email::EMAIL, fburl::FBURL, r#fn::FN, gender::GENDER, geo::GEO,
        gramgender::GRAMGENDER, impp::IMPP, jsprop::JSPROP, key::KEY, kind::KIND, label::LABEL,
        lang::LANG, language::LANGUAGE, logo::LOGO, mailer::MAILER, member::MEMBER, n::N,
        name::NAME, nickname::NICKNAME, note::NOTE, org::ORG, photo::PHOTO, prodid::PRODID,
        profile::PROFILE, pronouns::PRONOUNS, related::RELATED, rev::REV, role::ROLE,
        socialprofile::SOCIALPROFILE, sort_string::SORT_STRING, sound::SOUND, source::SOURCE,
        tel::TEL, title::TITLE, tz::TZ, uid::UID, url::URL, xml::XML,
    };
    use vcard::tree::param::r#type::TYPE;

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
        // The properties RFC 9554 and RFC 9555 added.
        "CREATED:20260101T000000Z\r\n",
        "GRAMGENDER:neuter\r\n",
        "JSPROP;JSPTR=x:{}\r\n",
        "LANGUAGE:en\r\n",
        "PRONOUNS:they/them\r\n",
        "SOCIALPROFILE;SERVICE-TYPE=Mastodon:https://example.com/@ann\r\n",
        "END:VCARD\r\n",
    );
    let mut cst = VcardCst::parse(raw).unwrap();

    // NOTE: A lens pointing at the wrong property name reads `None` on a card
    // that carries every property, so the assertion, not the call, is what
    // proves the marker is wired to its own line.
    macro_rules! read {
        ($($m:ty),+ $(,)?) => {{ $(
            assert!(
                cst.prop::<$m>().is_some(),
                "{} does not read its own property",
                stringify!($m),
            );
        )+ }};
    }
    macro_rules! edit {
        ($($m:ty),+ $(,)?) => {{ $(
            assert!(
                cst.prop_mut::<$m>().is_some(),
                "{} does not reach its own property",
                stringify!($m),
            );
        )+ }};
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
        CREATED,
        EMAIL,
        FBURL,
        FN,
        GENDER,
        GEO,
        GRAMGENDER,
        IMPP,
        JSPROP,
        KEY,
        KIND,
        LABEL,
        LANG,
        LANGUAGE,
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
        PRONOUNS,
        RELATED,
        REV,
        ROLE,
        SOCIALPROFILE,
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
    use vcard::prop::tel::TEL;
    use vcard::tree::param::{
        altid::ALTID, calscale::CALSCALE, geo::GEO as GEOP, label::LABEL as LABELP,
        language::LANGUAGE, lens::VcardParamLens, mediatype::MEDIATYPE, pid::PID, pref::PREF,
        sort_as::SORT_AS, r#type::TYPE, tz::TZ as TZP, value::VALUE,
    };

    let raw = concat!(
        "BEGIN:VCARD\r\nVERSION:4.0\r\n",
        "TEL;ALTID=1;CALSCALE=gregorian;GEO=x;LABEL=addr;LANGUAGE=en;",
        "MEDIATYPE=text/plain;PID=1,2;PREF=1;SORT-AS=Doe;TYPE=home,work;TZ=-0500;VALUE=text:123\r\n",
        "END:VCARD\r\n",
    );
    let mut cst = VcardCst::parse(raw).unwrap();

    // decode: each lens reads the value its own parameter carries on the line.
    let c = cst.prop_mut::<TEL>().unwrap();
    assert_eq!(c.param::<ALTID>().as_deref(), Some("1"));
    assert_eq!(c.param::<CALSCALE>().as_deref(), Some("gregorian"));
    assert_eq!(c.param::<GEOP>().as_deref(), Some("x"));
    assert_eq!(c.param::<LABELP>().as_deref(), Some("addr"));
    assert_eq!(c.param::<LANGUAGE>().as_deref(), Some("en"));
    assert_eq!(c.param::<MEDIATYPE>().as_deref(), Some("text/plain"));
    assert_eq!(
        c.param::<PID>(),
        Some(vec![Cow::Borrowed("1"), Cow::Borrowed("2")])
    );
    assert_eq!(c.param::<PREF>().as_deref(), Some("1"));
    assert_eq!(c.param::<SORT_AS>(), Some(vec![Cow::Borrowed("Doe")]));
    assert_eq!(
        c.param::<TYPE>(),
        Some(vec![Cow::Borrowed("home"), Cow::Borrowed("work")]),
    );
    assert_eq!(c.param::<TZP>().as_deref(), Some("-0500"));
    assert_eq!(c.param::<VALUE>().as_deref(), Some("text"));

    // encode: each lens mints a node under the name it is named after. The
    // names are spelled out rather than read back from the lens' own KIND, so a
    // marker pointing at the wrong kind has somewhere to fail.
    let scalar: Cow<'static, str> = Cow::Borrowed("x");
    let list = vec![Cow::Borrowed("x")];
    assert_eq!(
        ALTID::encode(&scalar, VcardEscaper::V4_0).to_string(),
        "ALTID=x"
    );
    assert_eq!(
        CALSCALE::encode(&scalar, VcardEscaper::V4_0).to_string(),
        "CALSCALE=x"
    );
    assert_eq!(
        GEOP::encode(&scalar, VcardEscaper::V4_0).to_string(),
        "GEO=x"
    );
    assert_eq!(
        LABELP::encode(&scalar, VcardEscaper::V4_0).to_string(),
        "LABEL=x"
    );
    assert_eq!(
        LANGUAGE::encode(&scalar, VcardEscaper::V4_0).to_string(),
        "LANGUAGE=x"
    );
    assert_eq!(
        MEDIATYPE::encode(&scalar, VcardEscaper::V4_0).to_string(),
        "MEDIATYPE=x"
    );
    assert_eq!(PID::encode(&list, VcardEscaper::V4_0).to_string(), "PID=x");
    assert_eq!(
        PREF::encode(&scalar, VcardEscaper::V4_0).to_string(),
        "PREF=x"
    );
    assert_eq!(
        SORT_AS::encode(&list, VcardEscaper::V4_0).to_string(),
        "SORT-AS=x"
    );
    assert_eq!(
        TYPE::encode(&list, VcardEscaper::V4_0).to_string(),
        "TYPE=x"
    );
    assert_eq!(TZP::encode(&scalar, VcardEscaper::V4_0).to_string(), "TZ=x");
    assert_eq!(
        VALUE::encode(&scalar, VcardEscaper::V4_0).to_string(),
        "VALUE=x"
    );
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

    // VcardPropKindParseError Display.
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
fn set_component_bytes_pads_missing_components() {
    // Writing a component beyond the current count pads the gap (the raw-bytes
    // structured-value path).
    let mut cst = VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nNOTE:x\r\nEND:VCARD\r\n").unwrap();
    let seed: &[&[u8]] = &[b"y"];
    cst.props[1].value.set_component_bytes(3, seed);
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
    use vcard::tree::{codec::VcardCodec, value::node::VcardValueNode};

    // NOTE: the VcardValue VcardCodec impl is the liberal raw fallback the
    // divergent lenses inherit, exercised directly here.
    let node = VcardValueNode::parse(b"a;b,c");
    let value = <VcardValue as VcardCodec>::decode(&node);
    assert!(matches!(value, VcardValue::Unknown(_)));
}

/// The parameters RFC 9554 and RFC 9555 added, wire to model and back.
///
/// Every value here is free of colons and quotes: a quoted parameter value is
/// not yet read as RFC 6350 3.3 defines it, so a card built around one would
/// pin a bug rather than the parameters.
#[test]
fn the_newer_parameters_decode_onto_their_own_variants_and_encode_back() {
    let raw = concat!(
        "BEGIN:VCARD\r\nVERSION:4.0\r\n",
        "NOTE;AUTHOR=urn-ann;AUTHOR-NAME=Ann;CREATED=20260101T000000Z;DERIVED=TRUE;",
        "JSPTR=addresses/a1;PHONETIC=ipa;PROP-ID=n1;SCRIPT=Latn;SERVICE-TYPE=Mastodon;",
        "USERNAME=ann:hi\r\n",
        "END:VCARD\r\n",
    );

    let cst = VcardCst::parse(raw).unwrap();
    let card = cst.decode();
    let params = &card.properties[0].params;

    let expected = [
        (VcardParam::Author(Cow::Borrowed("urn-ann")), "AUTHOR"),
        (VcardParam::AuthorName(Cow::Borrowed("Ann")), "AUTHOR-NAME"),
        (
            VcardParam::Created(Cow::Borrowed("20260101T000000Z")),
            "CREATED",
        ),
        (VcardParam::Derived(Cow::Borrowed("TRUE")), "DERIVED"),
        (VcardParam::Jsptr(Cow::Borrowed("addresses/a1")), "JSPTR"),
        (VcardParam::Phonetic(Cow::Borrowed("ipa")), "PHONETIC"),
        (VcardParam::PropId(Cow::Borrowed("n1")), "PROP-ID"),
        (VcardParam::Script(Cow::Borrowed("Latn")), "SCRIPT"),
        (
            VcardParam::ServiceType(Cow::Borrowed("Mastodon")),
            "SERVICE-TYPE",
        ),
        (VcardParam::Username(Cow::Borrowed("ann")), "USERNAME"),
    ];

    assert_eq!(params.len(), expected.len(), "a parameter went missing");

    for (decoded, (want, name)) in params.iter().zip(expected) {
        assert_eq!(decoded, &want, "{name} decodes onto the wrong variant");
        assert_eq!(
            decoded.kind().map(|kind| param_wire_name(kind)),
            Some(name),
            "{name} reports the wrong kind",
        );
    }

    // The decoded card writes the same line back out.
    assert_eq!(VcardCst::from(card).to_string(), raw);
}
