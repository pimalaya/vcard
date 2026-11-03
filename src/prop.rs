//! # Properties
//!
//! A decoded property and the RFC 6350 property-name vocabulary.
//!
//! A [`VcardProp`] is a [`VcardPropName`], a list of parameters, and a decoded
//! value. The name is stored explicitly because many properties share one
//! [`VcardValue`] kind: `FN` and `TITLE` both decode to text, so the value
//! alone cannot say which property it is.
//!
//! A known name is the closed [`VcardPropKind`] identity (its wire spelling
//! reached through `Deref` and `FromStr`); an unknown one keeps its verbatim
//! bytes.
//!
//! Build a property directly from its public fields; strict, spec-checked
//! construction is [`VcardPropBuilder`](crate::builder::VcardPropBuilder).
//!
//! This module also owns the per-property contract: one marker module per
//! property the crate knows, each carrying its [`VcardPropSpec`](spec) impl,
//! and the [`spec`] vtable that bridges the open [`VcardPropKind`] back to
//! those static impls. The contract is what the RFC allows, so it is model
//! rather than syntax and needs no parser; the read-and-edit lens on the same
//! marker lives in [`crate::tree::prop`].

pub mod adr;
pub mod agent;
pub mod anniversary;
pub mod bday;
pub mod caladruri;
pub mod caluri;
pub mod cardinality;
pub mod categories;
pub mod class;
pub mod client_pid_map;
pub mod created;
pub mod email;
pub mod fburl;
pub mod r#fn;
pub mod gender;
pub mod geo;
pub mod gramgender;
pub mod impp;
pub mod jsprop;
pub mod key;
pub mod kind;
pub mod label;
pub mod lang;
pub mod language;
pub mod logo;
pub mod mailer;
pub mod member;
pub mod n;
pub mod name;
pub mod nickname;
pub mod note;
pub mod org;
pub mod photo;
pub mod prodid;
pub mod profile;
pub mod pronouns;
pub mod related;
pub mod rev;
pub mod role;
pub mod socialprofile;
pub mod sort_string;
pub mod sound;
pub mod source;
pub mod spec;
pub mod tel;
pub mod title;
pub mod tz;
pub mod uid;
pub mod url;
pub mod xml;

use core::{error, fmt, ops, str};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    param::VcardParam,
    value::{VcardValue, text::VcardText},
};

/// Parse vCard property kind error.
#[derive(Debug)]
pub struct VcardPropKindParseError(
    /// The vCard property that cannot be parsed.
    String,
);

impl fmt::Display for VcardPropKindParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse vCard property `{}`", self.0)
    }
}

impl error::Error for VcardPropKindParseError {}

/// A decoded property: its group, name, parameters and value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VcardProp<'a> {
    /// The group (RFC 6350 3.3), `item1` in `item1.TEL`.
    pub group: Option<Cow<'a, str>>,
    /// The name without its group: a known kind, or an unknown one verbatim.
    pub name: VcardPropName<'a>,
    /// The parameters decorating the property.
    pub params: Vec<VcardParam<'a>>,
    /// The decoded value.
    pub value: VcardValue<'a>,
}

impl VcardProp<'_> {
    /// A property holding nothing, for a decoder to fill in place.
    #[cfg(feature = "parser")]
    pub(crate) const fn empty() -> Self {
        Self {
            group: None,
            name: VcardPropName::Unknown(Cow::Borrowed("")),
            params: Vec::new(),
            value: VcardValue::Unknown(crate::value::VcardValueUnknown {
                components: Vec::new(),
            }),
        }
    }
}

impl<'a> VcardProp<'a> {
    /// A text property with no group: a known kind or an `X-` name, its
    /// parameters and its raw value, escaped when the card is serialized.
    ///
    /// The shape a projection mints when it synthesizes a card from another
    /// representation.
    pub fn text(
        name: impl Into<VcardPropName<'a>>,
        params: Vec<VcardParam<'a>>,
        value: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            group: None,
            name: name.into(),
            params,
            value: VcardValue::Text(VcardText(value.into())),
        }
    }
}

/// A property name: a known RFC 6350 name, or an unknown one kept verbatim.
///
/// Known names normalise to their canonical [`VcardPropKind`] spelling; unknown
/// names keep their exact bytes so they round-trip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardPropName<'a> {
    /// A name in the closed RFC 6350 vocabulary.
    Kind(VcardPropKind),
    /// Any other kind, kept as written.
    Unknown(Cow<'a, str>),
}

impl ops::Deref for VcardPropName<'_> {
    type Target = str;

    /// The name's wire string: the canonical spelling of a known name, or the
    /// verbatim text of an unknown one.
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Kind(kind) => kind,
            Self::Unknown(name) => name,
        }
    }
}

impl From<VcardPropKind> for VcardPropName<'_> {
    fn from(kind: VcardPropKind) -> Self {
        Self::Kind(kind)
    }
}

impl From<&VcardPropKind> for VcardPropName<'_> {
    fn from(kind: &VcardPropKind) -> Self {
        Self::Kind(*kind)
    }
}

impl<'a> From<Cow<'a, str>> for VcardPropName<'a> {
    fn from(kind: Cow<'a, str>) -> Self {
        match VcardPropKind::from_name(&kind) {
            Some(kind) => Self::Kind(kind),
            None => Self::Unknown(kind),
        }
    }
}

impl<'a> From<&'a str> for VcardPropName<'a> {
    fn from(name: &'a str) -> Self {
        Cow::Borrowed(name).into()
    }
}

/// The closed RFC 6350 property-name vocabulary, one fieldless variant per
/// known property. An identity for dispatch and allowed-sets; the
/// open-vocabulary counterpart that also carries unknown names is
/// [`VcardPropName`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VcardPropKind {
    /// `ADR`: structured delivery address (RFC 6350 6.3.1).
    Adr,
    /// `AGENT`: an associated agent or assistant (RFC 2426 3.5.4).
    Agent,
    /// `ANNIVERSARY`: date of marriage or equivalent (RFC 6350 6.2.6).
    Anniversary,
    /// `BDAY`: date of birth (RFC 6350 6.2.5).
    Bday,
    /// `CALADRURI`: URI for sending a scheduling request (RFC 6350 6.9.2).
    CalAdrUri,
    /// `CALURI`: URI of the associated calendar (RFC 6350 6.9.3).
    CalUri,
    /// `CATEGORIES`: tags or categories for the card (RFC 6350 6.7.1).
    Categories,
    /// `CLASS`: access classification (RFC 2426 3.7.1).
    Class,
    /// `CLIENTPIDMAP`: maps PID source ids to client URIs (RFC 6350 6.7.7).
    ClientPidMap,
    /// `CREATED`: timestamp of the card's creation (RFC 9554).
    Created,
    /// `EMAIL`: email address (RFC 6350 6.4.2).
    Email,
    /// `FBURL`: free/busy URL (RFC 6350 6.9.1).
    FbUrl,
    /// `FN`: formatted display name (RFC 6350 6.2.1).
    Fn,
    /// `GENDER`: sex and gender identity (RFC 6350 6.2.7).
    Gender,
    /// `GEO`: geographic position (RFC 6350 6.5.2).
    Geo,
    /// `GRAMGENDER`: grammatical gender to address the contact by (RFC 9554).
    GramGender,
    /// `IMPP`: instant-messaging and presence URI (RFC 6350 6.4.3).
    Impp,
    /// `JSPROP`: a JSContact property with no vCard counterpart, preserved as
    /// JSON during conversion (RFC 9555).
    JsProp,
    /// `KEY`: public key or certificate (RFC 6350 6.8.1).
    Key,
    /// `KIND`: kind of object the card describes (RFC 6350 6.1.4).
    Kind,
    /// `LABEL`: formatted delivery-address label (RFC 2426 3.2.2; a parameter
    /// in 4.0).
    Label,
    /// `LANG`: language the contact may be addressed in (RFC 6350 6.4.4).
    Lang,
    /// `LANGUAGE`: default language of the card's free-text values (RFC
    /// 9554).
    Language,
    /// `LOGO`: graphic logo of the organization (RFC 6350 6.6.3).
    Logo,
    /// `MAILER`: email program used (RFC 2426 3.3.2).
    Mailer,
    /// `MEMBER`: member of the group this card represents (RFC 6350 6.6.5).
    Member,
    /// `N`: structured name (RFC 6350 6.2.2).
    N,
    /// `NAME`: displayable source name (RFC 2426 3.1.5).
    Name,
    /// `NICKNAME`: nicknames (RFC 6350 6.2.3).
    Nickname,
    /// `NOTE`: free-text note (RFC 6350 6.7.2).
    Note,
    /// `ORG`: organization name and units (RFC 6350 6.6.4).
    Org,
    /// `PHOTO`: photograph of the contact (RFC 6350 6.2.4).
    Photo,
    /// `PRODID`: product that created the card (RFC 6350 6.7.3).
    ProdId,
    /// `PROFILE`: declares the object a vCard profile (RFC 2426 3.1.4).
    Profile,
    /// `PRONOUNS`: pronouns to refer to the contact by (RFC 9554).
    Pronouns,
    /// `RELATED`: relationship to another entity (RFC 6350 6.6.6).
    Related,
    /// `REV`: revision timestamp (RFC 6350 6.7.4).
    Rev,
    /// `ROLE`: role or occupation (RFC 6350 6.6.2).
    Role,
    /// `SOCIALPROFILE`: social-media profile, a URI or a username (RFC 9554).
    SocialProfile,
    /// `SORT-STRING`: string to sort the card by (RFC 2426 3.3.4; the SORT-AS
    /// parameter in 4.0).
    SortString,
    /// `SOUND`: sound, e.g. name pronunciation (RFC 6350 6.7.5).
    Sound,
    /// `SOURCE`: URI the card was fetched from (RFC 6350 6.1.3).
    Source,
    /// `TEL`: telephone number (RFC 6350 6.4.1).
    Tel,
    /// `TITLE`: job title or position (RFC 6350 6.6.1).
    Title,
    /// `TZ`: time zone (RFC 6350 6.5.1).
    Tz,
    /// `UID`: globally unique identifier (RFC 6350 6.7.6).
    Uid,
    /// `URL`: associated URL (RFC 6350 6.7.8).
    Url,
    /// `XML`: extended XML data (RFC 6350 6.1.5).
    Xml,
}

impl VcardPropKind {
    /// Every known property kind, for iterating the closed vocabulary (e.g. a
    /// validator checking which required properties are absent).
    pub const ALL: [Self; 48] = [
        Self::Adr,
        Self::Agent,
        Self::Anniversary,
        Self::Bday,
        Self::CalAdrUri,
        Self::CalUri,
        Self::Categories,
        Self::Class,
        Self::ClientPidMap,
        Self::Created,
        Self::Email,
        Self::FbUrl,
        Self::Fn,
        Self::Gender,
        Self::Geo,
        Self::GramGender,
        Self::Impp,
        Self::JsProp,
        Self::Key,
        Self::Kind,
        Self::Label,
        Self::Lang,
        Self::Language,
        Self::Logo,
        Self::Mailer,
        Self::Member,
        Self::N,
        Self::Name,
        Self::Nickname,
        Self::Note,
        Self::Org,
        Self::Photo,
        Self::ProdId,
        Self::Profile,
        Self::Pronouns,
        Self::Related,
        Self::Rev,
        Self::Role,
        Self::SocialProfile,
        Self::SortString,
        Self::Sound,
        Self::Source,
        Self::Tel,
        Self::Title,
        Self::Tz,
        Self::Uid,
        Self::Url,
        Self::Xml,
    ];
}

impl VcardPropKind {
    /// The known property for a wire name (case-insensitive), or `None`.
    ///
    /// The length and the first byte name one candidate, the second byte
    /// telling apart the few that share both, and the candidate is compared
    /// whole, case aside. The name is never copied: a copy into a buffer, then
    /// read back wider, waits on the copy's stores.
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        let bytes = name.as_bytes();
        let first = bytes.first()?.to_ascii_uppercase();
        let second = bytes.get(1).map_or(0, u8::to_ascii_uppercase);

        let kind = match (bytes.len(), first) {
            (1, b'N') => Self::N,
            (2, b'F') => Self::Fn,
            (2, b'T') => Self::Tz,
            (3, b'A') => Self::Adr,
            (3, b'G') => Self::Geo,
            (3, b'K') => Self::Key,
            (3, b'O') => Self::Org,
            (3, b'R') => Self::Rev,
            (3, b'T') => Self::Tel,
            (3, b'U') if second == b'I' => Self::Uid,
            (3, b'U') => Self::Url,
            (3, b'X') => Self::Xml,
            (4, b'B') => Self::Bday,
            (4, b'I') => Self::Impp,
            (4, b'K') => Self::Kind,
            (4, b'L') if second == b'A' => Self::Lang,
            (4, b'L') => Self::Logo,
            (4, b'N') if second == b'A' => Self::Name,
            (4, b'N') => Self::Note,
            (4, b'R') => Self::Role,
            (5, b'A') => Self::Agent,
            (5, b'C') => Self::Class,
            (5, b'E') => Self::Email,
            (5, b'F') => Self::FbUrl,
            (5, b'L') => Self::Label,
            (5, b'P') => Self::Photo,
            (5, b'S') => Self::Sound,
            (5, b'T') => Self::Title,
            (6, b'C') => Self::CalUri,
            (6, b'G') => Self::Gender,
            (6, b'J') => Self::JsProp,
            (6, b'M') if second == b'A' => Self::Mailer,
            (6, b'M') => Self::Member,
            (6, b'P') => Self::ProdId,
            (6, b'S') => Self::Source,
            (7, b'C') => Self::Created,
            (7, b'P') => Self::Profile,
            (7, b'R') => Self::Related,
            (8, b'L') => Self::Language,
            (8, b'N') => Self::Nickname,
            (8, b'P') => Self::Pronouns,
            (9, b'C') => Self::CalAdrUri,
            (10, b'C') => Self::Categories,
            (10, b'G') => Self::GramGender,
            (11, b'A') => Self::Anniversary,
            (11, b'S') => Self::SortString,
            (12, b'C') => Self::ClientPidMap,
            (13, b'S') => Self::SocialProfile,
            _ => return None,
        };

        kind.eq_ignore_ascii_case(name).then_some(kind)
    }
}

impl str::FromStr for VcardPropKind {
    type Err = VcardPropKindParseError;

    /// The known property for a wire name (case-insensitive), or `None`.
    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        Self::from_name(kind).ok_or_else(|| VcardPropKindParseError(kind.to_string()))
    }
}

impl ops::Deref for VcardPropKind {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Adr => "ADR",
            Self::Agent => "AGENT",
            Self::Anniversary => "ANNIVERSARY",
            Self::Bday => "BDAY",
            Self::CalAdrUri => "CALADRURI",
            Self::CalUri => "CALURI",
            Self::Categories => "CATEGORIES",
            Self::Class => "CLASS",
            Self::ClientPidMap => "CLIENTPIDMAP",
            Self::Created => "CREATED",
            Self::Email => "EMAIL",
            Self::FbUrl => "FBURL",
            Self::Fn => "FN",
            Self::Gender => "GENDER",
            Self::Geo => "GEO",
            Self::GramGender => "GRAMGENDER",
            Self::Impp => "IMPP",
            Self::JsProp => "JSPROP",
            Self::Key => "KEY",
            Self::Kind => "KIND",
            Self::Label => "LABEL",
            Self::Lang => "LANG",
            Self::Language => "LANGUAGE",
            Self::Logo => "LOGO",
            Self::Mailer => "MAILER",
            Self::Member => "MEMBER",
            Self::N => "N",
            Self::Name => "NAME",
            Self::Nickname => "NICKNAME",
            Self::Note => "NOTE",
            Self::Org => "ORG",
            Self::Photo => "PHOTO",
            Self::ProdId => "PRODID",
            Self::Profile => "PROFILE",
            Self::Pronouns => "PRONOUNS",
            Self::Related => "RELATED",
            Self::Rev => "REV",
            Self::Role => "ROLE",
            Self::SocialProfile => "SOCIALPROFILE",
            Self::SortString => "SORT-STRING",
            Self::Sound => "SOUND",
            Self::Source => "SOURCE",
            Self::Tel => "TEL",
            Self::Title => "TITLE",
            Self::Tz => "TZ",
            Self::Uid => "UID",
            Self::Url => "URL",
            Self::Xml => "XML",
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use core::str::FromStr;

    use alloc::{
        borrow::Cow,
        format,
        string::{String, ToString},
        vec,
        vec::Vec,
    };

    use crate::{
        param::VcardParam,
        prop::{VcardProp, VcardPropKind, VcardPropName},
        value::{VcardValue, text::VcardText},
    };

    #[test]
    fn names_the_property_and_wraps_the_value() {
        let prop = VcardProp {
            group: None,
            name: VcardPropKind::Title.into(),
            params: [].into(),
            value: VcardValue::Text(VcardText(Cow::Borrowed("Developer"))),
        };
        assert_eq!(prop.name, VcardPropName::Kind(VcardPropKind::Title));
        assert_eq!(&*prop.name, "TITLE");
        assert_eq!(
            prop.value,
            VcardValue::Text(VcardText(Cow::Borrowed("Developer"))),
        );
        assert!(prop.params.is_empty());
    }

    #[test]
    fn carries_the_given_parameters() {
        let prop = VcardProp {
            group: None,
            name: VcardPropKind::Fn.into(),
            params: [VcardParam::Pref(Cow::Borrowed("1"))].into(),
            value: VcardValue::Text(VcardText(Cow::Borrowed("John"))),
        };
        assert_eq!(&*prop.name, "FN");
        assert_eq!(prop.params, vec![VcardParam::Pref(Cow::Borrowed("1"))]);
    }

    #[test]
    fn round_trips_every_kind_through_its_wire_name() {
        for kind in [
            VcardPropKind::Fn,
            VcardPropKind::ClientPidMap,
            VcardPropKind::SortString,
            VcardPropKind::CalAdrUri,
        ] {
            assert_eq!(VcardPropKind::from_str(&kind).ok(), Some(kind));
        }
        assert_eq!(VcardPropKind::from_str("fn").ok(), Some(VcardPropKind::Fn));
        assert!(VcardPropKind::from_str("X-CUSTOM").is_err());
    }

    /// Names a lookup could confuse with `name`: recased, one byte off, cut
    /// short and grown.
    pub(crate) fn near_names(name: &str) -> Vec<String> {
        let mut names = vec![
            name.to_string(),
            name.to_ascii_lowercase(),
            name[..name.len() - 1].to_string(),
            format!("{name}X"),
            format!("X{name}"),
        ];

        for i in 0..name.len() {
            for flip in [0x01, 0x20] {
                let mut bytes = name.as_bytes().to_vec();
                bytes[i] ^= flip;
                names.extend(String::from_utf8(bytes));
            }
        }

        names
    }

    /// The lookup picks one candidate by length and first bytes, then compares
    /// it whole, so it must find what a search of every kind finds.
    #[test]
    fn finds_what_a_search_of_every_kind_finds() {
        let search = |name: &str| {
            VcardPropKind::ALL
                .into_iter()
                .find(|kind| kind.eq_ignore_ascii_case(name))
        };

        for kind in VcardPropKind::ALL {
            for name in near_names(&kind) {
                assert_eq!(VcardPropKind::from_name(&name), search(&name), "{name:?}");
            }
        }

        assert_eq!(VcardPropKind::from_name(""), None);
    }

    #[test]
    fn text_builds_a_groupless_text_prop_for_known_and_unknown_names() {
        let known = VcardProp::text(VcardPropKind::Note, vec![], "a, b");
        let unknown = VcardProp::text("X-VENDOR-FIELD", vec![], "c");

        assert_eq!(known.group, None);
        assert_eq!(known.name, VcardPropName::Kind(VcardPropKind::Note));
        assert_eq!(
            unknown.name,
            VcardPropName::Unknown(Cow::Borrowed("X-VENDOR-FIELD"))
        );
    }
}
