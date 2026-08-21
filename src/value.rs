//! # Property values
//!
//! The decoded value of a property, one variant per RFC 6350 value kind.
//!
//! [`VcardValue`] is the semantic counterpart of a content line's raw value
//! (the syntactic
//! [`VcardValueNode`](crate::tree::value::node::VcardValueNode)). Most
//! properties share a small set of kinds: a single text, a text list, a URI, a
//! date/time, a timestamp, a UTC offset, a language tag. The genuinely
//! structured ones get a bespoke type in a submodule here ([`n::VcardN`],
//! [`adr::VcardAdr`], [`gender::VcardGender`], [`org::VcardOrg`],
//! [`client_pid_map::VcardClientPidMap`]). Anything the model does not decode
//! falls back to [`Unknown`](VcardValue::Unknown), which keeps the raw
//! components so it round-trips.
//!
//! These types carry no wire name and no escaping: the name lives on
//! [`VcardProp::name`](crate::prop::VcardProp::name), the escaping and framing
//! on the syntax side, so the decoded model stays free of [`crate::tree`].

pub mod adr;
pub mod binary;
pub mod client_pid_map;
pub mod datetime;
pub mod gender;
pub mod geo;
pub mod language;
pub mod n;
pub mod org;
pub mod text;
pub mod uri;
pub mod utc_offset;

use core::{error, fmt, ops, str};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::value::{
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
};

/// Parse vCard value kind error.
#[derive(Debug)]
pub struct VcardValueKindParseError(
    /// The vCard value type that cannot be parsed.
    String,
);

impl fmt::Display for VcardValueKindParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse vCard value type `{}`", self.0)
    }
}

impl error::Error for VcardValueKindParseError {}

/// The closed RFC 6350 value-type vocabulary, one fieldless variant per value
/// kind. It is the discriminant of [`VcardValue`] (which also has an `Unknown`
/// arm outside this closed set) and the currency of the prop spec's
/// allowed-values sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VcardValueKind {
    /// The structured `ADR` value (RFC 6350 6.3.1).
    Adr,
    /// An inline-base64 or URI-reference binary value (vCard 2.1 / 3.0).
    Binary,
    /// The structured `CLIENTPIDMAP` value (RFC 6350 6.7.7).
    ClientPidMap,
    /// A date-and-or-time value (RFC 6350 4.3.4).
    DateAndOrTime,
    /// The structured `GENDER` value (RFC 6350 6.2.7).
    Gender,
    /// A latitude/longitude pair (vCard 2.1 / 3.0 `GEO`).
    Geo,
    /// A language tag (RFC 6350 4.8).
    LanguageTag,
    /// The structured `N` value (RFC 6350 6.2.2).
    N,
    /// The structured `ORG` value (RFC 6350 6.6.4).
    Org,
    /// A single text value (RFC 6350 4.1).
    Text,
    /// A comma-separated text list (RFC 6350 4.1).
    TextList,
    /// A timestamp (RFC 6350 4.3.5).
    Timestamp,
    /// A URI (RFC 6350 4.2).
    Uri,
    /// A UTC offset (RFC 6350 4.7).
    UtcOffset,
}

impl str::FromStr for VcardValueKind {
    type Err = VcardValueKindParseError;

    /// The value kind named by a `VALUE` parameter (case-insensitive). Liberal:
    /// it maps every wire spelling (and a few aliases) onto a model kind,
    /// leaving membership checks to a later validation tier.
    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        match kind {
            kind if kind.eq_ignore_ascii_case("ADR") => Ok(Self::Adr),
            kind if kind.eq_ignore_ascii_case("B") => Ok(Self::Binary),
            kind if kind.eq_ignore_ascii_case("BINARY") => Ok(Self::Binary),
            kind if kind.eq_ignore_ascii_case("CLIENTPIDMAP") => Ok(Self::ClientPidMap),
            kind if kind.eq_ignore_ascii_case("DATE") => Ok(Self::DateAndOrTime),
            kind if kind.eq_ignore_ascii_case("DATE-AND-OR-TIME") => Ok(Self::DateAndOrTime),
            kind if kind.eq_ignore_ascii_case("DATE-TIME") => Ok(Self::DateAndOrTime),
            kind if kind.eq_ignore_ascii_case("GENDER") => Ok(Self::Gender),
            kind if kind.eq_ignore_ascii_case("GEO") => Ok(Self::Geo),
            kind if kind.eq_ignore_ascii_case("LANGUAGE-TAG") => Ok(Self::LanguageTag),
            kind if kind.eq_ignore_ascii_case("N") => Ok(Self::N),
            kind if kind.eq_ignore_ascii_case("ORG") => Ok(Self::Org),
            kind if kind.eq_ignore_ascii_case("TEXT") => Ok(Self::Text),
            kind if kind.eq_ignore_ascii_case("TEXT-LIST") => Ok(Self::TextList),
            kind if kind.eq_ignore_ascii_case("TIME") => Ok(Self::DateAndOrTime),
            kind if kind.eq_ignore_ascii_case("TIMESTAMP") => Ok(Self::Timestamp),
            kind if kind.eq_ignore_ascii_case("URI") => Ok(Self::Uri),
            kind if kind.eq_ignore_ascii_case("URL") => Ok(Self::Uri),
            kind if kind.eq_ignore_ascii_case("UTC-OFFSET") => Ok(Self::UtcOffset),
            _ => Err(VcardValueKindParseError(kind.to_string())),
        }
    }
}

impl ops::Deref for VcardValueKind {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Adr => "ADR",
            Self::Binary => "BINARY",
            Self::ClientPidMap => "CLIENTPIDMAP",
            Self::DateAndOrTime => "DATE-AND-OR-TIME",
            Self::Gender => "GENDER",
            Self::Geo => "GEO",
            Self::LanguageTag => "LANGUAGE-TAG",
            Self::N => "N",
            Self::Org => "ORG",
            Self::Text => "TEXT",
            Self::TextList => "TEXT-LIST",
            Self::Timestamp => "TIMESTAMP",
            Self::Uri => "URI",
            Self::UtcOffset => "UTC-OFFSET",
        }
    }
}

/// A decoded property value: one known kind, or `Unknown` (raw) for anything
/// the model does not decode.
// NOTE: the 18-component VcardAdr dominates the enum size; values are
// decoded on demand, not stored in bulk, so plain variants beat boxing.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardValue<'a> {
    /// The structured `ADR` value.
    Adr(VcardAdr<'a>),
    /// A binary value (2.1 / 3.0 `PHOTO`, `LOGO`, `SOUND`, `KEY`): a URI
    /// reference or inline base64.
    Binary(VcardBinary<'a>),
    /// The structured `CLIENTPIDMAP` value.
    ClientPidMap(VcardClientPidMap<'a>),
    /// A date-and-or-time (`BDAY`, `ANNIVERSARY`).
    DateAndOrTime(VcardDateAndOrTime<'a>),
    /// The structured `GENDER` value.
    Gender(VcardGender<'a>),
    /// A `GEO` latitude/longitude pair (2.1 / 3.0; 4.0 uses a URI).
    Geo(VcardGeo<'a>),
    /// A language tag (`LANG`).
    LanguageTag(VcardLanguageTag<'a>),
    /// The structured `N` value.
    N(VcardN<'a>),
    /// The structured `ORG` value.
    Org(VcardOrg<'a>),
    /// A single text value (`FN`, `TITLE`, `NOTE`, ...).
    Text(VcardText<'a>),
    /// A comma-separated text list (`NICKNAME`, `CATEGORIES`).
    TextList(VcardTextList<'a>),
    /// A timestamp (`REV`).
    Timestamp(VcardTimestamp<'a>),
    /// A URI (`PHOTO`, `URL`, `KEY`, ...).
    Uri(VcardUri<'a>),
    /// A UTC offset (one form of `TZ`).
    UtcOffset(VcardUtcOffset<'a>),
    /// Any value the model does not decode, kept as its raw components so it
    /// round-trips.
    Unknown(VcardValueUnknown<'a>),
}

impl VcardValue<'_> {
    /// The closed [`VcardValueKind`] of this value, or `None` for
    /// [`Unknown`](VcardValue::Unknown) (which is outside the vocabulary).
    pub fn kind(&self) -> Option<VcardValueKind> {
        match self {
            Self::Adr(_) => Some(VcardValueKind::Adr),
            Self::Binary(_) => Some(VcardValueKind::Binary),
            Self::ClientPidMap(_) => Some(VcardValueKind::ClientPidMap),
            Self::DateAndOrTime(_) => Some(VcardValueKind::DateAndOrTime),
            Self::Gender(_) => Some(VcardValueKind::Gender),
            Self::Geo(_) => Some(VcardValueKind::Geo),
            Self::LanguageTag(_) => Some(VcardValueKind::LanguageTag),
            Self::N(_) => Some(VcardValueKind::N),
            Self::Org(_) => Some(VcardValueKind::Org),
            Self::Text(_) => Some(VcardValueKind::Text),
            Self::TextList(_) => Some(VcardValueKind::TextList),
            Self::Timestamp(_) => Some(VcardValueKind::Timestamp),
            Self::Uri(_) => Some(VcardValueKind::Uri),
            Self::UtcOffset(_) => Some(VcardValueKind::UtcOffset),
            Self::Unknown(_) => None,
        }
    }

    /// An empty value of the given kind: the inverse of [`kind`](Self::kind),
    /// so `empty(k).kind() == Some(k)`. Every component is blank (an empty
    /// string, list, or structured value); a binary is an empty URI
    /// reference. Used to mint a placeholder for a required property that is
    /// otherwise absent (see [`VcardCst::fill_required`]).
    ///
    /// [`VcardCst::fill_required`]: crate::tree::cst::VcardCst::fill_required
    pub fn empty(kind: VcardValueKind) -> VcardValue<'static> {
        match kind {
            VcardValueKind::Adr => VcardValue::Adr(VcardAdr::default()),
            VcardValueKind::Binary => VcardValue::Binary(VcardBinary::Uri(Cow::Borrowed(""))),
            VcardValueKind::ClientPidMap => VcardValue::ClientPidMap(VcardClientPidMap::default()),
            VcardValueKind::DateAndOrTime => {
                VcardValue::DateAndOrTime(VcardDateAndOrTime::default())
            }
            VcardValueKind::Gender => VcardValue::Gender(VcardGender::default()),
            VcardValueKind::Geo => VcardValue::Geo(VcardGeo::default()),
            VcardValueKind::LanguageTag => VcardValue::LanguageTag(VcardLanguageTag::default()),
            VcardValueKind::N => VcardValue::N(VcardN::default()),
            VcardValueKind::Org => VcardValue::Org(VcardOrg::default()),
            VcardValueKind::Text => VcardValue::Text(VcardText::default()),
            VcardValueKind::TextList => VcardValue::TextList(VcardTextList::default()),
            VcardValueKind::Timestamp => VcardValue::Timestamp(VcardTimestamp::default()),
            VcardValueKind::Uri => VcardValue::Uri(VcardUri::default()),
            VcardValueKind::UtcOffset => VcardValue::UtcOffset(VcardUtcOffset::default()),
        }
    }
}

/// An undecoded property value: its unescaped components, in source order. The
/// property name lives on [`VcardProp::name`](crate::prop::VcardProp::name).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardValueUnknown<'a> {
    /// The value, as components of values.
    pub components: Vec<Vec<Cow<'a, str>>>,
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::value::{VcardValue, VcardValueKind, VcardValueUnknown, text::VcardText};

    #[test]
    fn empty_is_the_inverse_of_kind() {
        use crate::value::VcardValueKind::*;

        for kind in [
            Adr,
            Binary,
            ClientPidMap,
            DateAndOrTime,
            Gender,
            Geo,
            LanguageTag,
            N,
            Org,
            Text,
            TextList,
            Timestamp,
            Uri,
            UtcOffset,
        ] {
            assert_eq!(VcardValue::empty(kind).kind(), Some(kind));
        }
    }

    #[test]
    fn reports_the_kind_of_a_value_and_none_for_unknown() {
        assert_eq!(
            VcardValue::Text(VcardText::default()).kind(),
            Some(VcardValueKind::Text),
        );
        assert_eq!(
            VcardValue::Unknown(VcardValueUnknown::default()).kind(),
            None,
        );
    }

    #[test]
    fn maps_value_param_strings_liberally_and_case_insensitively() {
        assert_eq!("URI".parse().ok(), Some(VcardValueKind::Uri));
        assert_eq!("date".parse().ok(), Some(VcardValueKind::DateAndOrTime));
        assert!(VcardValueKind::from_str("bogus").is_err());
    }
}
