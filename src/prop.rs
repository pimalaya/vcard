//! # Properties
//!
//! A decoded property and the RFC 6350 property-name vocabulary.
//!
//! A [`VcardProp`] is a [`VcardPropName`], a list of parameters, and a decoded
//! value. The name is stored explicitly because many properties share one
//! [`VcardValue`] kind: `FN` and `TITLE` both decode to text, so the value
//! alone cannot say which property it is. A known name is held as the closed
//! [`VcardPropKind`] identity (its wire spelling reached through `Deref` and
//! `FromStr`); an unknown one keeps its verbatim bytes. The lens markers in
//! [`crate::tree::prop`] carry the kind to match and build lines, and the
//! decode registry parses a line name onto its value kind.
//!
//! Build a property directly from its public fields; strict, spec-checked
//! construction lives in the syntax layer
//! ([`VcardPropBuilder`](crate::tree::vcard::builder::VcardPropBuilder)).
//!
//! This module is pure model: it has no dependency on [`crate::tree`], so the
//! decoded form can be used without the syntax layer.

use core::{error, fmt, ops, str};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::{param::VcardParam, value::VcardValue};

/// Parse vCard property kind error.
#[derive(Debug)]
pub struct ParseVcardPropKindError(
    /// The vCard property that cannot be parsed.
    String,
);

impl fmt::Display for ParseVcardPropKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse vCard property `{}`", self.0)
    }
}

impl error::Error for ParseVcardPropKindError {}

/// A decoded property: its wire name, its parameters, and its decoded value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VcardProp<'a> {
    /// The property name (a known kind, or an unknown name kept verbatim).
    pub name: VcardPropName<'a>,
    /// The parameters decorating the property.
    pub params: Vec<VcardParam<'a>>,
    /// The decoded value.
    pub value: VcardValue<'a>,
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
        match kind.parse().ok() {
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

impl str::FromStr for VcardPropKind {
    type Err = ParseVcardPropKindError;

    /// The known property for a wire name (case-insensitive), or `None`.
    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        match kind {
            kind if kind.eq_ignore_ascii_case("ADR") => Ok(Self::Adr),
            kind if kind.eq_ignore_ascii_case("AGENT") => Ok(Self::Agent),
            kind if kind.eq_ignore_ascii_case("ANNIVERSARY") => Ok(Self::Anniversary),
            kind if kind.eq_ignore_ascii_case("BDAY") => Ok(Self::Bday),
            kind if kind.eq_ignore_ascii_case("CALADRURI") => Ok(Self::CalAdrUri),
            kind if kind.eq_ignore_ascii_case("CALURI") => Ok(Self::CalUri),
            kind if kind.eq_ignore_ascii_case("CATEGORIES") => Ok(Self::Categories),
            kind if kind.eq_ignore_ascii_case("CLASS") => Ok(Self::Class),
            kind if kind.eq_ignore_ascii_case("CLIENTPIDMAP") => Ok(Self::ClientPidMap),
            kind if kind.eq_ignore_ascii_case("CREATED") => Ok(Self::Created),
            kind if kind.eq_ignore_ascii_case("EMAIL") => Ok(Self::Email),
            kind if kind.eq_ignore_ascii_case("FBURL") => Ok(Self::FbUrl),
            kind if kind.eq_ignore_ascii_case("FN") => Ok(Self::Fn),
            kind if kind.eq_ignore_ascii_case("GENDER") => Ok(Self::Gender),
            kind if kind.eq_ignore_ascii_case("GEO") => Ok(Self::Geo),
            kind if kind.eq_ignore_ascii_case("GRAMGENDER") => Ok(Self::GramGender),
            kind if kind.eq_ignore_ascii_case("IMPP") => Ok(Self::Impp),
            kind if kind.eq_ignore_ascii_case("JSPROP") => Ok(Self::JsProp),
            kind if kind.eq_ignore_ascii_case("KEY") => Ok(Self::Key),
            kind if kind.eq_ignore_ascii_case("KIND") => Ok(Self::Kind),
            kind if kind.eq_ignore_ascii_case("LABEL") => Ok(Self::Label),
            kind if kind.eq_ignore_ascii_case("LANG") => Ok(Self::Lang),
            kind if kind.eq_ignore_ascii_case("LANGUAGE") => Ok(Self::Language),
            kind if kind.eq_ignore_ascii_case("LOGO") => Ok(Self::Logo),
            kind if kind.eq_ignore_ascii_case("MAILER") => Ok(Self::Mailer),
            kind if kind.eq_ignore_ascii_case("MEMBER") => Ok(Self::Member),
            kind if kind.eq_ignore_ascii_case("N") => Ok(Self::N),
            kind if kind.eq_ignore_ascii_case("NAME") => Ok(Self::Name),
            kind if kind.eq_ignore_ascii_case("NICKNAME") => Ok(Self::Nickname),
            kind if kind.eq_ignore_ascii_case("NOTE") => Ok(Self::Note),
            kind if kind.eq_ignore_ascii_case("ORG") => Ok(Self::Org),
            kind if kind.eq_ignore_ascii_case("PHOTO") => Ok(Self::Photo),
            kind if kind.eq_ignore_ascii_case("PRODID") => Ok(Self::ProdId),
            kind if kind.eq_ignore_ascii_case("PROFILE") => Ok(Self::Profile),
            kind if kind.eq_ignore_ascii_case("PRONOUNS") => Ok(Self::Pronouns),
            kind if kind.eq_ignore_ascii_case("RELATED") => Ok(Self::Related),
            kind if kind.eq_ignore_ascii_case("REV") => Ok(Self::Rev),
            kind if kind.eq_ignore_ascii_case("ROLE") => Ok(Self::Role),
            kind if kind.eq_ignore_ascii_case("SOCIALPROFILE") => Ok(Self::SocialProfile),
            kind if kind.eq_ignore_ascii_case("SORT-STRING") => Ok(Self::SortString),
            kind if kind.eq_ignore_ascii_case("SOUND") => Ok(Self::Sound),
            kind if kind.eq_ignore_ascii_case("SOURCE") => Ok(Self::Source),
            kind if kind.eq_ignore_ascii_case("TEL") => Ok(Self::Tel),
            kind if kind.eq_ignore_ascii_case("TITLE") => Ok(Self::Title),
            kind if kind.eq_ignore_ascii_case("TZ") => Ok(Self::Tz),
            kind if kind.eq_ignore_ascii_case("UID") => Ok(Self::Uid),
            kind if kind.eq_ignore_ascii_case("URL") => Ok(Self::Url),
            kind if kind.eq_ignore_ascii_case("XML") => Ok(Self::Xml),
            _ => Err(ParseVcardPropKindError(kind.to_string())),
        }
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
mod tests {
    use core::str::FromStr;

    use alloc::{borrow::Cow, vec};

    use crate::{
        param::VcardParam,
        prop::{VcardProp, VcardPropKind, VcardPropName},
        value::{VcardValue, text::VcardText},
    };

    #[test]
    fn names_the_property_and_wraps_the_value() {
        let prop = VcardProp {
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
        // NOTE: Case-insensitive on the way in; unknown names are not in the
        // vocabulary.
        assert_eq!(VcardPropKind::from_str("fn").ok(), Some(VcardPropKind::Fn));
        assert!(VcardPropKind::from_str("X-CUSTOM").is_err());
    }
}
