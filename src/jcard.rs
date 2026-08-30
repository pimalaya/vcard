//! # jCard
//!
//! The RFC 7095 jCard codec: the decoded card as JSON, and back.
//!
//! jCard is the JSON spelling of the vCard model: a card is a
//! `["vcard", [...]]` array of `[name, {params}, type, value...]` property
//! entries (RFC 7095 3).
//!
//! [`Vcard::to_jcard`] writes the decoded model as a [`serde_json::Value`];
//! [`Vcard::from_jcard`] reads one back, borrowing the JSON tree's strings and
//! resolving each value kind through the same spec vtable as the wire decoder,
//! so a jCard and the vCard it came from decode to the same model.
//!
//! The codec keeps the crate's Postel stance, one direction per submodule:
//! `export` follows RFC 7095 to the letter, `import` accepts anything, and
//! `datetime` re-spells a date, a time and a UTC offset between the extended
//! form the RFC writes and the basic one RFC 6350 reads.
//!
//! Two deliberate, lossless departures from the letter of the RFC. A card of
//! any version is written, carrying its own version in the `version` entry,
//! where RFC 7095 defines jCard for vCard 4.0 only.
//!
//! And an undecoded value is written structurally, its semicolon and comma
//! components mirrored as a string, an array or an array of arrays, rather than
//! as the RFC's one re-escaped string: the decoded model holds unescaped
//! components, and re-escaping belongs to the wire codec.
//!
//! Round-tripping normalizes rather than preserves: parameter order is lost to
//! the JSON object, a declared `VALUE` equal to the property's default is
//! dropped, and names come back in their canonical spelling. Byte fidelity is
//! the syntax tree's job; jCard is a projection of the decoded model.

use core::{error, fmt};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use serde_json::{Value, json};

use crate::{prop::VcardProp, vcard::Vcard, version::VcardVersion};

pub(crate) mod datetime;
pub(crate) mod export;
mod import;

/// Parse jCard error.
#[derive(Debug)]
pub enum VcardJcardParseError {
    /// The root is not a two-element `["vcard", [...]]` array.
    InvalidCard,
    /// A property entry is not a `[name, {params}, type, ...]` array.
    InvalidProp(String),
}

impl fmt::Display for VcardJcardParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCard => {
                write!(f, "Cannot parse jCard: the root is not [\"vcard\", [...]]")
            }
            Self::InvalidProp(prop) => write!(f, "Cannot parse jCard property `{prop}`"),
        }
    }
}

impl error::Error for VcardJcardParseError {}

impl Vcard<'_> {
    /// Write the card as its RFC 7095 jCard [`Value`].
    pub fn to_jcard(&self) -> Value {
        let mut entries = Vec::with_capacity(self.properties.len() + 1);
        entries.push(json!(["version", {}, "text", &*self.version]));
        entries.extend(self.properties.iter().map(VcardProp::to_jcard));
        json!(["vcard", entries])
    }

    /// Read a card from its RFC 7095 jCard [`Value`], borrowing its strings.
    ///
    /// Liberal: only a structurally broken root or property entry errors;
    /// unknown names, parameters and type slots are kept.
    pub fn from_jcard(jcard: &Value) -> Result<Vcard<'_>, VcardJcardParseError> {
        let root = jcard
            .as_array()
            .filter(|root| root.len() == 2)
            .ok_or(VcardJcardParseError::InvalidCard)?;

        let tag = root[0].as_str().ok_or(VcardJcardParseError::InvalidCard)?;
        if !tag.eq_ignore_ascii_case("vcard") {
            return Err(VcardJcardParseError::InvalidCard);
        }

        let entries = root[1]
            .as_array()
            .ok_or(VcardJcardParseError::InvalidCard)?;

        // NOTE: the version is the card's indicator, not a free property,
        // mirroring the wire decoder; an unreadable one normalises to 4.0.
        let version = entries
            .iter()
            .filter_map(Value::as_array)
            .find(|entry| entry_is_version(entry))
            .and_then(|entry| entry.get(3))
            .and_then(Value::as_str)
            .and_then(|version| version.parse().ok())
            .unwrap_or(VcardVersion::V4_0);

        let mut properties = Vec::new();

        for entry in entries {
            let invalid = || VcardJcardParseError::InvalidProp(entry.to_string());
            let entry = entry
                .as_array()
                .filter(|entry| entry.len() >= 3)
                .ok_or_else(invalid)?;

            if entry_is_version(entry) {
                continue;
            }

            let name = entry[0].as_str().ok_or_else(invalid)?;
            let params = entry[1].as_object().ok_or_else(invalid)?;
            let slot = entry[2].as_str().ok_or_else(invalid)?;

            properties.push(VcardProp::from_jcard(
                name,
                params,
                slot,
                &entry[3..],
                version,
            ));
        }

        Ok(Vcard {
            version,
            properties,
        })
    }
}

/// Whether a jCard property entry is the `version` pseudo-property.
fn entry_is_version(entry: &[Value]) -> bool {
    matches!(
        entry.first().and_then(Value::as_str),
        Some(name) if name.eq_ignore_ascii_case("version"),
    )
}

/// The codec reads and writes the decoded model alone, so it holds in a build
/// carrying no parser.
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

        let jcard = card.to_jcard();
        let back = Vcard::from_jcard(&jcard).expect("a well-formed jCard");

        assert_eq!(back.version, VcardVersion::V4_0);
        assert_eq!(&*back.properties[0].name, "FN");
        assert_eq!(
            back.properties[0].value,
            VcardValue::Text(VcardText(Cow::Borrowed("John Doe"))),
        );
    }
}

#[cfg(all(test, feature = "parser"))]
mod tests {
    use alloc::{borrow::Cow, vec};

    use serde_json::json;

    use crate::{
        jcard::{
            VcardJcardParseError,
            datetime::{basic_to_extended, extended_str_to_basic},
        },
        param::VcardParam,
        prop::VcardPropName,
        tree::cst::VcardCst,
        value::{
            VcardValue, VcardValueUnknown, client_pid_map::VcardClientPidMap, n::VcardN,
            text::VcardText,
        },
        vcard::Vcard,
        version::VcardVersion,
    };

    #[test]
    fn exports_a_simple_card_with_its_version_entry() {
        let cst =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nEND:VCARD\r\n").unwrap();

        assert_eq!(
            cst.decode().to_jcard(),
            json!([
                "vcard",
                [
                    ["version", {}, "text", "4.0"],
                    ["fn", {}, "text", "John Doe"],
                ],
            ]),
        );
    }

    #[test]
    fn moves_the_value_param_into_the_type_slot_and_back() {
        let cst = VcardCst::parse(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nBDAY;VALUE=text:circa 1800\r\nEND:VCARD\r\n",
        )
        .unwrap();
        let jcard = cst.decode().to_jcard();

        assert_eq!(
            jcard,
            json!([
                "vcard",
                [
                    ["version", {}, "text", "4.0"],
                    ["bday", {}, "text", "circa 1800"],
                ],
            ]),
        );

        // NOTE: text is not BDAY's default kind, so the declared VALUE comes
        // back as a parameter and the value stays text.
        let card = Vcard::from_jcard(&jcard).unwrap();
        let prop = &card.properties[0];
        assert_eq!(prop.params, vec![VcardParam::Value(Cow::Borrowed("text"))]);
        assert_eq!(
            prop.value,
            VcardValue::Text(VcardText(Cow::Borrowed("circa 1800"))),
        );
    }

    #[test]
    fn moves_a_group_prefix_into_the_group_parameter_and_back() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "ITEM1.X-ABLABEL:Nickname\r\n",
            "END:VCARD\r\n",
        );
        let jcard = VcardCst::parse(input).unwrap().decode().to_jcard();

        assert_eq!(
            jcard,
            json!([
                "vcard",
                [
                    ["version", {}, "text", "4.0"],
                    ["x-ablabel", { "group": "item1" }, "unknown", "Nickname"],
                ],
            ]),
        );

        let card = Vcard::from_jcard(&jcard).unwrap();
        let prop = &card.properties[0];
        assert_eq!(
            prop.name,
            VcardPropName::Unknown(Cow::Borrowed("item1.X-ABLABEL")),
        );
        assert_eq!(
            prop.value,
            VcardValue::Unknown(VcardValueUnknown {
                components: vec![vec![Cow::Borrowed("Nickname")]],
            }),
        );
    }

    #[test]
    fn converts_dates_between_basic_and_extended() {
        assert_eq!(basic_to_extended("19850412"), "1985-04-12");
        assert_eq!(basic_to_extended("1985-04"), "1985-04");
        assert_eq!(basic_to_extended("--0412"), "--04-12");
        assert_eq!(basic_to_extended("---12"), "---12");
        assert_eq!(basic_to_extended("T102200"), "T10:22:00");
        assert_eq!(basic_to_extended("T-2200"), "T-22:00");
        assert_eq!(
            basic_to_extended("19961022T140000-0500"),
            "1996-10-22T14:00:00-05:00",
        );
        assert_eq!(
            basic_to_extended("19961022T140000Z"),
            "1996-10-22T14:00:00Z"
        );
        assert_eq!(basic_to_extended("circa 1800"), "circa 1800");

        assert_eq!(
            extended_str_to_basic("1996-10-22T14:00:00-05:00").as_deref(),
            Some("19961022T140000-0500"),
        );
        assert_eq!(extended_str_to_basic("--04-12").as_deref(), Some("--0412"));
        assert_eq!(extended_str_to_basic("1985-04"), None);
        assert_eq!(extended_str_to_basic("circa 1800"), None);
    }

    #[test]
    fn merges_a_repeated_parameter_and_keeps_a_list_one() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "TEL;TYPE=work,home;TYPE=voice:tel:123\r\n",
            "END:VCARD\r\n",
        );
        let jcard = VcardCst::parse(input).unwrap().decode().to_jcard();

        assert_eq!(
            jcard,
            json!([
                "vcard",
                [
                    ["version", {}, "text", "4.0"],
                    ["tel", { "type": ["work", "home", "voice"] }, "text", "tel:123"],
                ],
            ]),
        );

        let card = Vcard::from_jcard(&jcard).unwrap();
        assert_eq!(
            card.properties[0].params,
            vec![VcardParam::Type(vec![
                Cow::Borrowed("work"),
                Cow::Borrowed("home"),
                Cow::Borrowed("voice"),
            ])],
        );
    }

    #[test]
    fn round_trips_a_card_through_jcard() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John Doe\r\n",
            "N:Doe;John;;Dr.;\r\n",
            "EMAIL;TYPE=work:john@example.com\r\n",
            "CATEGORIES:friend,colleague\r\n",
            "BDAY:19850412\r\n",
            "LANG;PREF=1:fr\r\n",
            "REV:19961022T140000Z\r\n",
            "TZ;VALUE=utc-offset:-0500\r\n",
            "UID:urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1\r\n",
            // NOTE: The structured and binary values below each take their own
            // branch in both directions, so a card without them leaves half of
            // the conversion unwalked.
            "GENDER:F;grrrl\r\n",
            "ORG:Example;Research;Optics\r\n",
            "ORG:Solo\r\n",
            "CLIENTPIDMAP:1;urn:uuid:3df67951-1932-4fc6-9d54-8b4c0e0ba0b2\r\n",
            "GEO:geo:37.386013\r\n",
            "PHOTO:data:image/jpeg;base64,Zm9v\r\n",
            "KEY;ENCODING=b:Zm9v\r\n",
            "X-SKI:snowboard\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode();
        let jcard = card.to_jcard();
        let reimported = Vcard::from_jcard(&jcard).unwrap();

        assert_eq!(reimported.version, VcardVersion::V4_0);
        assert_eq!(reimported, card);
    }

    /// An RFC 7095 section 2.1-style example: a number where text is expected,
    /// an uppercase tag, no version entry.
    #[test]
    fn imports_a_foreign_jcard_liberally() {
        let jcard = json!([
            "VCARD",
            [
                ["fn", {}, "text", "SimonPerreault"],
                ["clientpidmap", {}, "text", [1, "urn:uuid:x"]],
                [
                    "n",
                    {},
                    "text",
                    ["Perreault", "Simon", "", "", ["ing.jr", "M.Sc."]]
                ],
            ],
        ]);
        let card = Vcard::from_jcard(&jcard).unwrap();

        assert_eq!(card.version, VcardVersion::V4_0);
        assert_eq!(card.properties.len(), 3);
        assert_eq!(
            card.properties[1].value,
            VcardValue::ClientPidMap(VcardClientPidMap {
                id: Cow::Borrowed("1"),
                uri: Cow::Borrowed("urn:uuid:x"),
            }),
        );
        assert_eq!(
            card.properties[2].value,
            VcardValue::N(VcardN {
                family: vec![Cow::Borrowed("Perreault")],
                given: vec![Cow::Borrowed("Simon")],
                additional: vec![Cow::Borrowed("")],
                prefixes: vec![Cow::Borrowed("")],
                suffixes: vec![Cow::Borrowed("ing.jr"), Cow::Borrowed("M.Sc.")],
            }),
        );
    }

    #[test]
    fn errors_on_a_broken_root_or_property_entry() {
        for jcard in [json!({}), json!(["vcalendar", []]), json!(["vcard", [], 3])] {
            assert!(matches!(
                Vcard::from_jcard(&jcard),
                Err(VcardJcardParseError::InvalidCard),
            ));
        }

        let jcard = json!(["vcard", [["fn", {}]]]);
        assert!(matches!(
            Vcard::from_jcard(&jcard),
            Err(VcardJcardParseError::InvalidProp(_)),
        ));
    }
}
