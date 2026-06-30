//! # Properties
//!
//! A decoded property and the RFC 6350 property-name vocabulary.
//!
//! A [`VcardProp`] is a [`VcardPropName`], a list of parameters, and a
//! decoded value. The name is stored explicitly because many properties share
//! one [`VcardValue`] kind: `FN` and `TITLE` both decode to text, so the value
//! alone cannot say which property it is. A known name is held as the closed
//! [`VcardPropKind`] identity (its wire spelling reached through `Deref` and
//! `FromStr`); an unknown one keeps its verbatim bytes. The lens markers in
//! [`crate::tree::prop`] carry the kind to match and build lines, and the decode
//! registry parses a line name onto its value kind.
//!
//! Build a property directly from its public fields; strict, spec-checked
//! construction lives in the syntax layer
//! ([`VcardPropBuilder`](crate::tree::build::VcardPropBuilder)).
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

/// The closed RFC 6350 property-name vocabulary, one fieldless variant per
/// known property. An identity for dispatch and allowed-sets; the
/// open-vocabulary counterpart that also carries unknown names is
/// [`VcardPropName`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VcardPropKind {
    Adr,
    Agent,
    Anniversary,
    Bday,
    CalAdrUri,
    CalUri,
    Categories,
    Class,
    ClientPidMap,
    Email,
    FbUrl,
    Fn,
    Gender,
    Geo,
    Impp,
    Key,
    Kind,
    Label,
    Lang,
    Logo,
    Mailer,
    Member,
    N,
    Name,
    Nickname,
    Note,
    Org,
    Photo,
    ProdId,
    Profile,
    Related,
    Rev,
    Role,
    SortString,
    Sound,
    Source,
    Tel,
    Title,
    Tz,
    Uid,
    Url,
    Xml,
}

impl VcardPropKind {
    /// Every known property kind, for iterating the closed vocabulary (e.g. a
    /// validator checking which required properties are absent).
    pub const ALL: [Self; 42] = [
        Self::Adr,
        Self::Agent,
        Self::Anniversary,
        Self::Bday,
        Self::CalAdrUri,
        Self::CalUri,
        Self::Categories,
        Self::Class,
        Self::ClientPidMap,
        Self::Email,
        Self::FbUrl,
        Self::Fn,
        Self::Gender,
        Self::Geo,
        Self::Impp,
        Self::Key,
        Self::Kind,
        Self::Label,
        Self::Lang,
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
        Self::Related,
        Self::Rev,
        Self::Role,
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
            kind if kind.eq_ignore_ascii_case("EMAIL") => Ok(Self::Email),
            kind if kind.eq_ignore_ascii_case("FBURL") => Ok(Self::FbUrl),
            kind if kind.eq_ignore_ascii_case("FN") => Ok(Self::Fn),
            kind if kind.eq_ignore_ascii_case("GENDER") => Ok(Self::Gender),
            kind if kind.eq_ignore_ascii_case("GEO") => Ok(Self::Geo),
            kind if kind.eq_ignore_ascii_case("IMPP") => Ok(Self::Impp),
            kind if kind.eq_ignore_ascii_case("KEY") => Ok(Self::Key),
            kind if kind.eq_ignore_ascii_case("KIND") => Ok(Self::Kind),
            kind if kind.eq_ignore_ascii_case("LABEL") => Ok(Self::Label),
            kind if kind.eq_ignore_ascii_case("LANG") => Ok(Self::Lang),
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
            kind if kind.eq_ignore_ascii_case("RELATED") => Ok(Self::Related),
            kind if kind.eq_ignore_ascii_case("REV") => Ok(Self::Rev),
            kind if kind.eq_ignore_ascii_case("ROLE") => Ok(Self::Role),
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
            Self::Email => "EMAIL",
            Self::FbUrl => "FBURL",
            Self::Fn => "FN",
            Self::Gender => "GENDER",
            Self::Geo => "GEO",
            Self::Impp => "IMPP",
            Self::Key => "KEY",
            Self::Kind => "KIND",
            Self::Label => "LABEL",
            Self::Lang => "LANG",
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
            Self::Related => "RELATED",
            Self::Rev => "REV",
            Self::Role => "ROLE",
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

impl VcardPropName<'_> {
    /// The name's wire string: the canonical spelling of a known name, or the
    /// verbatim text of an unknown one.
    pub fn as_str(&self) -> &str {
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
        assert_eq!(prop.name.as_str(), "TITLE");
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
        assert_eq!(prop.name.as_str(), "FN");
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
        // Case-insensitive on the way in; unknown names are not in the vocabulary.
        assert_eq!(VcardPropKind::from_str("fn").ok(), Some(VcardPropKind::Fn),);
        assert!(VcardPropKind::from_str("X-CUSTOM").is_err());
    }
}
