//! # JSContact
//!
//! The RFC 9555 conversion: the decoded card as an RFC 9553 JSContact Card, and
//! back.
//!
//! [`Vcard::to_jscontact`] writes the decoded model as the JSON Card object
//! JMAP for Contacts (RFC 9610) exchanges; [`Vcard::from_jscontact`] reads one
//! back, borrowing the JSON tree's strings.
//!
//! There is no JSContact model in this crate: the Card is a plain
//! [`serde_json::Value`], and vCard stays the one decoded model.
//!
//! Both directions are lossless through the RFC 9555 escape hatches, and only
//! a non-object root can fail the import.
//!
//! Exporting, a property with no JSContact counterpart (or whose value cannot
//! be represented, like a free-text birthday) is preserved whole in the Card's
//! `vCardProps` member, in jCard syntax through the sibling [`crate::jcard`]
//! codec; a leftover parameter goes to the object's `vCardParams` member.
//!
//! Importing, the mirror hatch applies: a Card member (or nested piece) with no
//! vCard counterpart becomes a `JSPROP` property holding its JSON, located by a
//! `JSPTR` parameter that the export grafts back onto the Card.
//!
//! A `PROP-ID` parameter carries each object's map key across conversions,
//! which is what keeps JMAP patch identity stable; without one, keys are the
//! 1-based source order.
//!
//! Mapped properties: UID, PRODID, REV, KIND, FN and N (with SORT-AS),
//! NICKNAME, ADR (all eighteen components, with LABEL, GEO, TZ), EMAIL, TEL,
//! IMPP, LANG, ORG, TITLE, ROLE, BDAY, ANNIVERSARY, PHOTO, LOGO, SOUND, KEY,
//! CALURI, FBURL, CALADRURI, URL, SOURCE, CATEGORIES, NOTE, MEMBER, RELATED.
//!
//! The RFC 9554 set follows: CREATED, LANGUAGE, GRAMGENDER, PRONOUNS,
//! SOCIALPROFILE and the JSPROP carrier. The TYPE parameter maps to contexts
//! (`home` becomes `private`) and, on TEL, to features (`cell` becomes
//! `mobile`); PREF maps to `pref`. Everything else rides the escape hatches.
//!
//! The conversion is one module per direction, `export` and `import`, over
//! three shared pieces: `params` splits a property's parameters the one way
//! both directions agree on, `date` holds the RFC 9553 date shapes, and
//! `pointer` the RFC 6901 pointers the JSPROP hatch is keyed by.

use core::{error, fmt};

use serde_json::Value;

use crate::{
    jscontact::{export::Card, import::Import},
    vcard::Vcard,
    version::VcardVersion,
};

mod date;
mod export;
mod import;
mod params;
mod pointer;

/// Parse JSContact error.
#[derive(Debug)]
pub struct VcardJscontactParseError;

impl fmt::Display for VcardJscontactParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse JSContact card: the root is not an object")
    }
}

impl error::Error for VcardJscontactParseError {}

impl Vcard<'_> {
    /// Convert the card into an RFC 9553 JSContact Card [`Value`], following
    /// the RFC 9555 mapping.
    ///
    /// Infallible: a property or parameter with no JSContact counterpart is
    /// preserved in the vCardProps / vCardParams escape hatches.
    pub fn to_jscontact(&self) -> Value {
        let mut card = Card::default();

        for prop in &self.properties {
            card.prop(prop);
        }

        card.into_value()
    }

    /// Convert an RFC 9553 JSContact Card [`Value`] into a decoded card,
    /// following the RFC 9555 mapping and borrowing the JSON tree's strings.
    ///
    /// Liberal: only a non-object root errors; a member (or nested piece)
    /// with no vCard counterpart is preserved as a JSPROP property.
    pub fn from_jscontact(jscontact: &Value) -> Result<Vcard<'_>, VcardJscontactParseError> {
        let card = jscontact.as_object().ok_or(VcardJscontactParseError)?;
        let mut import = Import::default();

        for (member, value) in card {
            import.member(member, value);
        }

        Ok(Vcard {
            version: VcardVersion::V4_0,
            properties: import.properties,
        })
    }
}

/// The conversion reads and writes the decoded model alone, so it holds in a
/// build carrying no parser.
#[cfg(test)]
mod model_tests {
    use alloc::{borrow::Cow, vec};

    use crate::{
        prop::VcardProp, value::VcardValue, value::text::VcardText, vcard::Vcard,
        version::VcardVersion,
    };

    #[test]
    fn round_trips_a_hand_built_card() {
        let card = Vcard {
            version: VcardVersion::V4_0,
            properties: vec![VcardProp {
                name: "FN".into(),
                params: vec![],
                value: VcardValue::Text(VcardText(Cow::Borrowed("John Doe"))),
            }],
        };

        let jscontact = card.to_jscontact();
        assert_eq!(jscontact["name"]["full"], "John Doe");

        let back = Vcard::from_jscontact(&jscontact).expect("a Card object");
        assert_eq!(&*back.properties[0].name, "FN");
    }
}

#[cfg(all(test, feature = "parser"))]
mod tests {
    use alloc::{borrow::Cow, vec, vec::Vec};

    use serde_json::json;

    use crate::{prop::VcardPropName, tree::cst::VcardCst, vcard::Vcard};

    #[test]
    fn exports_a_minimal_card() {
        let cst =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nEND:VCARD\r\n").unwrap();

        assert_eq!(
            cst.decode().to_jscontact(),
            json!({
                "@type": "Card",
                "version": "1.0",
                "name": { "@type": "Name", "full": "John Doe" },
            }),
        );
    }

    #[test]
    fn exports_a_full_card() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "UID:urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1\r\n",
            "FN:Simon Perreault\r\n",
            "N:Perreault;Simon;;;ing. jr,M.Sc.\r\n",
            "ORG;SORT-AS=Viagenie:Viagenie;IT\r\n",
            "TITLE:Director\r\n",
            "EMAIL;TYPE=work;PREF=1:simon@example.com\r\n",
            "TEL;TYPE=work,cell,voice:tel:+1-418-262-6501\r\n",
            "ADR;TYPE=home;LABEL=The full label:;;2875 boul. Laurier;Quebec;QC;G1V 2M2;Canada\r\n",
            "LANG;PREF=2:fr\r\n",
            "BDAY:--0203\r\n",
            "ANNIVERSARY:20090808T143000Z\r\n",
            "CATEGORIES:developer,ietf\r\n",
            "NOTE:Hello\r\n",
            "URL:https://example.com\r\n",
            "KEY;MEDIATYPE=application/pgp-keys:https://example.com/key.asc\r\n",
            "REV:20240101T000000Z\r\n",
            "GENDER:M\r\n",
            "X-FOO;X-BAR=1:baz\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();

        assert_eq!(
            cst.decode().to_jscontact(),
            json!({
                "@type": "Card",
                "version": "1.0",
                "uid": "urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1",
                "updated": "2024-01-01T00:00:00Z",
                "name": {
                    "@type": "Name",
                    "full": "Simon Perreault",
                    "components": [
                        { "@type": "NameComponent", "kind": "surname", "value": "Perreault" },
                        { "@type": "NameComponent", "kind": "given", "value": "Simon" },
                        { "@type": "NameComponent", "kind": "credential", "value": "ing. jr" },
                        { "@type": "NameComponent", "kind": "credential", "value": "M.Sc." },
                    ],
                },
                "organizations": {
                    "1": {
                        "@type": "Organization",
                        "name": "Viagenie",
                        "units": [{ "@type": "OrgUnit", "name": "IT" }],
                        "sortAs": "Viagenie",
                    },
                },
                "titles": { "1": { "@type": "Title", "kind": "title", "name": "Director" } },
                "emails": {
                    "1": {
                        "@type": "EmailAddress",
                        "address": "simon@example.com",
                        "contexts": { "work": true },
                        "pref": 1,
                    },
                },
                "phones": {
                    "1": {
                        "@type": "Phone",
                        "number": "tel:+1-418-262-6501",
                        "contexts": { "work": true },
                        "features": { "mobile": true, "voice": true },
                    },
                },
                "addresses": {
                    "1": {
                        "@type": "Address",
                        "components": [
                            { "@type": "AddressComponent", "kind": "name", "value": "2875 boul. Laurier" },
                            { "@type": "AddressComponent", "kind": "locality", "value": "Quebec" },
                            { "@type": "AddressComponent", "kind": "region", "value": "QC" },
                            { "@type": "AddressComponent", "kind": "postcode", "value": "G1V 2M2" },
                            { "@type": "AddressComponent", "kind": "country", "value": "Canada" },
                        ],
                        "contexts": { "private": true },
                        "full": "The full label",
                    },
                },
                "preferredLanguages": {
                    "1": { "@type": "LanguagePref", "language": "fr", "pref": 2 },
                },
                "anniversaries": {
                    "1": {
                        "@type": "Anniversary",
                        "kind": "birth",
                        "date": { "@type": "PartialDate", "month": 2, "day": 3 },
                    },
                    "2": {
                        "@type": "Anniversary",
                        "kind": "wedding",
                        "date": { "@type": "Timestamp", "utc": "2009-08-08T14:30:00Z" },
                    },
                },
                "keywords": { "developer": true, "ietf": true },
                "notes": { "1": { "@type": "Note", "note": "Hello" } },
                "links": { "1": { "@type": "Link", "uri": "https://example.com" } },
                "cryptoKeys": {
                    "1": {
                        "@type": "CryptoKey",
                        "uri": "https://example.com/key.asc",
                        "mediaType": "application/pgp-keys",
                    },
                },
                "vCardProps": [
                    ["gender", {}, "text", "M"],
                    ["x-foo", { "x-bar": "1" }, "unknown", "baz"],
                ],
            }),
        );
    }

    #[test]
    fn tags_resource_objects_with_their_rfc_type_names() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John\r\n",
            "PHOTO:https://example.com/photo.png\r\n",
            "KEY:https://example.com/key.asc\r\n",
            "CALURI:https://example.com/cal.ics\r\n",
            "CALADRURI:mailto:john@example.com\r\n",
            "URL:https://example.com\r\n",
            "SOURCE:https://example.com/john.vcf\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode().to_jscontact();

        // NOTE: RFC 9553 2.6 registers these names; the pre-RFC drafts spelled
        // them `MediaResource`, `CryptoResource` and so on, and a strict
        // server (Fastmail) rejects the draft spelling outright.
        assert_eq!(card["media"]["1"]["@type"], json!("Media"));
        assert_eq!(card["cryptoKeys"]["1"]["@type"], json!("CryptoKey"));
        assert_eq!(card["calendars"]["1"]["@type"], json!("Calendar"));
        assert_eq!(
            card["schedulingAddresses"]["1"]["@type"],
            json!("SchedulingAddress")
        );
        assert_eq!(card["links"]["1"]["@type"], json!("Link"));
        assert_eq!(card["directories"]["1"]["@type"], json!("Directory"));
    }

    #[test]
    fn reads_back_a_resource_written_with_a_draft_type_name() {
        let card = json!({
            "@type": "Card",
            "version": "1.0",
            "links": { "1": { "@type": "LinkResource", "uri": "https://example.com" } },
        });
        let vcard = Vcard::from_jscontact(&card).unwrap();

        // NOTE: import ignores `@type`, so a Card written by an older version
        // still converts back to URL rather than falling into JSPROP.
        let names: Vec<&str> = vcard.properties.iter().map(|prop| &*prop.name).collect();
        assert_eq!(names, ["URL"]);
    }

    #[test]
    fn uses_prop_id_as_the_map_key() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John\r\n",
            "EMAIL;PROP-ID=e99:john@example.com\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode().to_jscontact();

        assert_eq!(card["emails"]["e99"]["address"], json!("john@example.com"),);
    }

    #[test]
    fn keeps_unconverted_params_in_vcard_params() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John\r\n",
            "EMAIL;ALTID=1;X-CUSTOM=y:john@example.com\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode().to_jscontact();

        assert_eq!(
            card["emails"]["1"]["vCardParams"],
            json!({ "altid": "1", "x-custom": "y" }),
        );
    }

    /// A free-text birthday fits neither Timestamp nor PartialDate, a
    /// parameterized CATEGORIES cannot ride the keywords boolean map, and a
    /// grouped property keeps its group in the escaped jCard entry.
    #[test]
    fn escapes_what_cannot_be_represented() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John\r\n",
            "BDAY;VALUE=text:circa 1800\r\n",
            "CATEGORIES;PREF=1:vip\r\n",
            "ITEM1.X-ABLABEL:Nickname\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode().to_jscontact();

        assert_eq!(
            card["vCardProps"],
            json!([
                ["bday", {}, "text", "circa 1800"],
                ["categories", { "pref": "1" }, "text", "vip"],
                ["x-ablabel", { "group": "item1" }, "unknown", "Nickname"],
            ]),
        );
        assert!(card.get("anniversaries").is_none());
        assert!(card.get("keywords").is_none());
    }

    #[test]
    fn maps_group_kind_members_and_relations() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "KIND:group\r\n",
            "FN:The Does\r\n",
            "MEMBER:urn:uuid:john\r\n",
            "MEMBER:urn:uuid:jane\r\n",
            "RELATED;TYPE=friend,met:urn:uuid:jimmy\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode().to_jscontact();

        assert_eq!(card["kind"], json!("group"));
        assert_eq!(
            card["members"],
            json!({ "urn:uuid:john": true, "urn:uuid:jane": true }),
        );
        assert_eq!(
            card["relatedTo"],
            json!({
                "urn:uuid:jimmy": {
                    "@type": "Relation",
                    "relation": { "friend": true, "met": true },
                },
            }),
        );
    }

    #[test]
    fn exports_the_rfc_9554_props_first_class() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John\r\n",
            "CREATED:20240101T000000Z\r\n",
            "LANGUAGE:fr\r\n",
            "GRAMGENDER:Masculine\r\n",
            "PRONOUNS;PREF=1;PROP-ID=p1:he/him\r\n",
            "SOCIALPROFILE;SERVICE-TYPE=Mastodon:https://example.social/@john\r\n",
            "JSPROP;JSPTR=/foo/bar:{\"baz\":42}\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();

        assert_eq!(
            cst.decode().to_jscontact(),
            json!({
                "@type": "Card",
                "version": "1.0",
                "created": "2024-01-01T00:00:00Z",
                "language": "fr",
                "name": { "@type": "Name", "full": "John" },
                "speakToAs": {
                    "@type": "SpeakToAs",
                    "grammaticalGender": "masculine",
                    "pronouns": {
                        "p1": { "@type": "Pronouns", "pronouns": "he/him", "pref": 1 },
                    },
                },
                "onlineServices": {
                    "1": {
                        "@type": "OnlineService",
                        "uri": "https://example.social/@john",
                        "service": "Mastodon",
                    },
                },
                "foo": { "bar": { "baz": 42 } },
            }),
        );
    }

    #[test]
    fn imports_a_jscontact_card() {
        use alloc::borrow::ToOwned;

        use crate::{param::VcardParam, prop::VcardPropKind, value::VcardValue};

        let jscontact = json!({
            "@type": "Card",
            "version": "1.0",
            "name": {
                "full": "Jane Doe",
                "components": [
                    { "kind": "surname", "value": "Doe" },
                    { "kind": "given", "value": "Jane" },
                ],
            },
            "emails": {
                "e1": {
                    "address": "jane@example.com",
                    "contexts": { "private": true },
                    "pref": 2,
                },
            },
            "anniversaries": {
                "a1": { "kind": "birth", "date": { "month": 4, "day": 12 } },
            },
            "onlineServices": {
                "o1": { "user": "@jane", "service": "Mastodon" },
            },
            "x-custom": { "hello": "world" },
        });
        let card = Vcard::from_jscontact(&jscontact).unwrap();

        // NOTE: members convert in their (alphabetical) JSON order.
        let names: Vec<&str> = card.properties.iter().map(|prop| &*prop.name).collect();
        assert_eq!(
            names,
            ["BDAY", "EMAIL", "FN", "N", "SOCIALPROFILE", "JSPROP"],
        );

        let bday = &card.properties[0];
        assert_eq!(bday.params, vec![VcardParam::PropId(Cow::Borrowed("a1"))]);
        assert!(matches!(&bday.value, VcardValue::DateAndOrTime(date) if date.0 == "--0412"),);

        let email = &card.properties[1];
        assert_eq!(
            email.params,
            vec![
                VcardParam::PropId(Cow::Borrowed("e1")),
                VcardParam::Pref(Cow::Owned("2".to_owned())),
                VcardParam::Type(vec![Cow::Borrowed("home")]),
            ],
        );

        let profile = &card.properties[4];
        assert_eq!(
            profile.name,
            VcardPropName::Kind(VcardPropKind::SocialProfile)
        );
        assert!(
            profile
                .params
                .contains(&VcardParam::ServiceType(Cow::Borrowed("Mastodon"))),
        );
        assert!(matches!(&profile.value, VcardValue::Text(user) if user.0 == "@jane"));

        let jsprop = &card.properties[5];
        assert_eq!(
            jsprop.params,
            vec![VcardParam::Jsptr(Cow::Borrowed("/x-custom"))],
        );
        assert!(
            matches!(&jsprop.value, VcardValue::Text(json) if json.0 == "{\"hello\":\"world\"}"),
        );
    }

    #[test]
    fn export_import_export_is_a_fixpoint() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "KIND:group\r\n",
            "UID:urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1\r\n",
            "FN:Simon Perreault\r\n",
            "N;SORT-AS=\"Perreault,Simon\":Perreault;Simon;;;ing. jr,M.Sc.\r\n",
            "ORG;SORT-AS=Viagenie:Viagenie;IT\r\n",
            "TITLE:Director\r\n",
            "ROLE:Project lead\r\n",
            "NICKNAME:Sim\r\n",
            "EMAIL;TYPE=work;PREF=1:simon@example.com\r\n",
            "TEL;TYPE=work,cell,voice:tel:+1-418-262-6501\r\n",
            "IMPP:xmpp:simon@example.com\r\n",
            "ADR;TYPE=home;LABEL=The full label:;;2875 boul. Laurier;Quebec;QC;G1V 2M2;Canada\r\n",
            "LANG;PREF=2:fr\r\n",
            "LANGUAGE:fr\r\n",
            "CREATED:20200101T000000Z\r\n",
            "GRAMGENDER:masculine\r\n",
            "PRONOUNS;PROP-ID=p1:he/him\r\n",
            "SOCIALPROFILE;SERVICE-TYPE=Mastodon:https://example.social/@simon\r\n",
            "BDAY:--0203\r\n",
            "ANNIVERSARY:20090808T143000Z\r\n",
            "CATEGORIES:developer,ietf\r\n",
            "NOTE:Hello\r\n",
            "URL:https://example.com\r\n",
            "SOURCE:https://directory.example.com/simon.vcf\r\n",
            "KEY;MEDIATYPE=application/pgp-keys:https://example.com/key.asc\r\n",
            "CALURI:https://example.com/cal.ics\r\n",
            "FBURL:https://example.com/fb.ifb\r\n",
            "CALADRURI:mailto:simon@example.com\r\n",
            "PHOTO;MEDIATYPE=image/jpeg:https://example.com/photo.jpg\r\n",
            "MEMBER:urn:uuid:john\r\n",
            "RELATED;TYPE=friend:urn:uuid:jimmy\r\n",
            "REV:20240101T000000Z\r\n",
            "GENDER:M\r\n",
            "JSPROP;JSPTR=/foo/bar:{\"baz\":42}\r\n",
            "X-FOO;X-BAR=1:baz\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let exported = cst.decode().to_jscontact();

        let reimported = Vcard::from_jscontact(&exported).unwrap();
        assert_eq!(reimported.to_jscontact(), exported);
    }

    #[test]
    fn converts_the_rfc_9554_address_components() {
        // NOTE: an 18-component ADR: the RFC 9554 slots map to their
        // JSContact component kinds, and the pair aliases resolve (street
        // name over street, apartment over extended address).
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:Simon\r\n",
            "ADR:;;;Quebec;QC;G1V 2M2;Canada;8th wing;;2;2875;boul. Laurier;;;;;;\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let exported = cst.decode().to_jscontact();

        assert_eq!(
            exported["addresses"]["1"]["components"],
            json!([
                { "@type": "AddressComponent", "kind": "name", "value": "boul. Laurier" },
                { "@type": "AddressComponent", "kind": "locality", "value": "Quebec" },
                { "@type": "AddressComponent", "kind": "region", "value": "QC" },
                { "@type": "AddressComponent", "kind": "postcode", "value": "G1V 2M2" },
                { "@type": "AddressComponent", "kind": "country", "value": "Canada" },
                { "@type": "AddressComponent", "kind": "room", "value": "8th wing" },
                { "@type": "AddressComponent", "kind": "floor", "value": "2" },
                { "@type": "AddressComponent", "kind": "number", "value": "2875" },
            ]),
        );

        let reimported = Vcard::from_jscontact(&exported).unwrap();
        assert_eq!(reimported.to_jscontact(), exported);

        // NOTE: a card filling both a legacy slot and its RFC 9554 alias
        // cannot pick a side, so it is escaped whole.
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:Simon\r\n",
            "ADR:;;2875 boul. Laurier;;;;;;;;;boul. Laurier;;;;;;\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let exported = cst.decode().to_jscontact();

        assert!(exported.get("addresses").is_none());
        assert_eq!(exported["vCardProps"][0][0], json!("adr"));
    }

    #[test]
    fn errors_only_on_a_non_object_root() {
        assert!(Vcard::from_jscontact(&json!([])).is_err());
        assert!(Vcard::from_jscontact(&json!("card")).is_err());

        let object = json!({});
        let empty = Vcard::from_jscontact(&object).unwrap();
        assert!(empty.properties.is_empty());
    }

    #[test]
    fn converts_partial_dates_and_utc_timestamps() {
        use crate::jscontact::date::{partial_date, utc_timestamp};

        assert_eq!(
            partial_date("19850412"),
            Some((Some(1985), Some(4), Some(12)))
        );
        assert_eq!(partial_date("1985-04"), Some((Some(1985), Some(4), None)));
        assert_eq!(partial_date("1985"), Some((Some(1985), None, None)));
        assert_eq!(partial_date("--0412"), Some((None, Some(4), Some(12))));
        assert_eq!(partial_date("--04"), Some((None, Some(4), None)));
        assert_eq!(partial_date("---12"), Some((None, None, Some(12))));
        assert_eq!(partial_date("circa 1800"), None);

        assert_eq!(
            utc_timestamp("20090808T143000Z").as_deref(),
            Some("2009-08-08T14:30:00Z"),
        );
        assert_eq!(
            utc_timestamp("2009-08-08T14:30:00Z").as_deref(),
            Some("2009-08-08T14:30:00Z"),
        );
        assert_eq!(utc_timestamp("20090808T143000-0500"), None);
        assert_eq!(utc_timestamp("20090808"), None);
    }
}
