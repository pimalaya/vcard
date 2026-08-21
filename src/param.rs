//! # Parameters
//!
//! A decoded parameter and the RFC 6350 parameter-name vocabulary.
//!
//! [`VcardParam`] is a closed set of the parameters the RFCs define, one
//! variant each, plus an [`Unknown`](VcardParam::Unknown) arm so anything else
//! round-trips. Parameters are few and simple (a text, a list, a small
//! integer), so unlike properties each variant carries its value directly
//! rather than through a shared value type; the variant itself names the
//! parameter. A known name is the closed [`VcardParamKind`], reached through
//! `FromStr` and `Deref`. Pure model, no [`crate::tree`] dependency.

use core::{error, fmt, ops, str};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

/// Parse vCard parameter kind error.
#[derive(Debug)]
pub struct VcardParamKindParseError(
    /// The vCard parameter that cannot be parsed.
    String,
);

impl fmt::Display for VcardParamKindParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse vCard parameter `{}`", self.0)
    }
}

impl error::Error for VcardParamKindParseError {}

/// The closed RFC 6350 parameter-name vocabulary, one fieldless variant per
/// known parameter. An identity for dispatch and allowed-sets; the open
/// counterpart that carries the value (and unknown names) is [`VcardParam`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VcardParamKind {
    /// `ALTID`: ties alternative representations together (RFC 6350 5.4).
    AltId,
    /// `AUTHOR`: URI of the author of the value (RFC 9554).
    Author,
    /// `AUTHOR-NAME`: name of the author of the value (RFC 9554).
    AuthorName,
    /// `CALSCALE`: calendar scale of a date/time value (RFC 6350 5.8).
    CalScale,
    /// `CHARSET`: character set of the value (vCard 2.1).
    Charset,
    /// `CREATED`: timestamp of the property's creation (RFC 9554).
    Created,
    /// `DERIVED`: whether the value derives from other properties (RFC 9554).
    Derived,
    /// `ENCODING`: inline encoding of the value (vCard 2.1 / 3.0).
    Encoding,
    /// `GEO`: global position of the property (RFC 6350 5.10).
    Geo,
    /// `JSPTR`: JSON pointer locating a preserved JSContact property (RFC
    /// 9555).
    Jsptr,
    /// `LABEL`: formatted delivery-address label (RFC 6350 6.3.1).
    Label,
    /// `LANGUAGE`: language of the value (RFC 6350 5.1).
    Language,
    /// `MEDIATYPE`: media type of the referenced resource (RFC 6350 5.7).
    MediaType,
    /// `PHONETIC`: phonetic system the value is written in (RFC 9554).
    Phonetic,
    /// `PID`: source identifiers of the property instance (RFC 6350 5.5).
    Pid,
    /// `PREF`: preference among a set of instances (RFC 6350 5.3).
    Pref,
    /// `PROP-ID`: identity of the property instance across conversions (RFC
    /// 9554).
    PropId,
    /// `SCRIPT`: script the value is written in (RFC 9554).
    Script,
    /// `SERVICE-TYPE`: online service the property points at (RFC 9554).
    ServiceType,
    /// `SORT-AS`: components to sort the property by (RFC 6350 5.9).
    SortAs,
    /// `TYPE`: kinds or contexts of the property (RFC 6350 5.6).
    Type,
    /// `TZ`: time zone of the property (RFC 6350 5.11).
    Tz,
    /// `USERNAME`: username on the online service (RFC 9554).
    Username,
    /// `VALUE`: value type the value is to be read as (RFC 6350 5.2).
    Value,
}

impl str::FromStr for VcardParamKind {
    type Err = VcardParamKindParseError;

    /// The known parameter for a wire name (case-insensitive).
    fn from_str(kind: &str) -> Result<Self, Self::Err> {
        let kind = match kind {
            kind if kind.eq_ignore_ascii_case("ALTID") => Self::AltId,
            kind if kind.eq_ignore_ascii_case("AUTHOR") => Self::Author,
            kind if kind.eq_ignore_ascii_case("AUTHOR-NAME") => Self::AuthorName,
            kind if kind.eq_ignore_ascii_case("CALSCALE") => Self::CalScale,
            kind if kind.eq_ignore_ascii_case("CHARSET") => Self::Charset,
            kind if kind.eq_ignore_ascii_case("CREATED") => Self::Created,
            kind if kind.eq_ignore_ascii_case("DERIVED") => Self::Derived,
            kind if kind.eq_ignore_ascii_case("ENCODING") => Self::Encoding,
            kind if kind.eq_ignore_ascii_case("GEO") => Self::Geo,
            kind if kind.eq_ignore_ascii_case("JSPTR") => Self::Jsptr,
            kind if kind.eq_ignore_ascii_case("LABEL") => Self::Label,
            kind if kind.eq_ignore_ascii_case("LANGUAGE") => Self::Language,
            kind if kind.eq_ignore_ascii_case("MEDIATYPE") => Self::MediaType,
            kind if kind.eq_ignore_ascii_case("PHONETIC") => Self::Phonetic,
            kind if kind.eq_ignore_ascii_case("PID") => Self::Pid,
            kind if kind.eq_ignore_ascii_case("PREF") => Self::Pref,
            kind if kind.eq_ignore_ascii_case("PROP-ID") => Self::PropId,
            kind if kind.eq_ignore_ascii_case("SCRIPT") => Self::Script,
            kind if kind.eq_ignore_ascii_case("SERVICE-TYPE") => Self::ServiceType,
            kind if kind.eq_ignore_ascii_case("SORT-AS") => Self::SortAs,
            kind if kind.eq_ignore_ascii_case("TYPE") => Self::Type,
            kind if kind.eq_ignore_ascii_case("TZ") => Self::Tz,
            kind if kind.eq_ignore_ascii_case("USERNAME") => Self::Username,
            kind if kind.eq_ignore_ascii_case("VALUE") => Self::Value,
            _ => return Err(VcardParamKindParseError(kind.to_string())),
        };

        Ok(kind)
    }
}

impl ops::Deref for VcardParamKind {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::AltId => "ALTID",
            Self::Author => "AUTHOR",
            Self::AuthorName => "AUTHOR-NAME",
            Self::CalScale => "CALSCALE",
            Self::Charset => "CHARSET",
            Self::Created => "CREATED",
            Self::Derived => "DERIVED",
            Self::Encoding => "ENCODING",
            Self::Geo => "GEO",
            Self::Jsptr => "JSPTR",
            Self::Label => "LABEL",
            Self::Language => "LANGUAGE",
            Self::MediaType => "MEDIATYPE",
            Self::Phonetic => "PHONETIC",
            Self::Pid => "PID",
            Self::Pref => "PREF",
            Self::PropId => "PROP-ID",
            Self::Script => "SCRIPT",
            Self::ServiceType => "SERVICE-TYPE",
            Self::SortAs => "SORT-AS",
            Self::Type => "TYPE",
            Self::Tz => "TZ",
            Self::Username => "USERNAME",
            Self::Value => "VALUE",
        }
    }
}

/// A decoded parameter: one known kind, or `Unknown` for anything unmodelled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardParam<'a> {
    /// `ALTID`: ties alternative representations of the same logical property.
    AltId(Cow<'a, str>),
    /// `AUTHOR`: the URI of the author of the value.
    Author(Cow<'a, str>),
    /// `AUTHOR-NAME`: the name of the author of the value.
    AuthorName(Cow<'a, str>),
    /// `CALSCALE`: the calendar scale of a date/time value.
    CalScale(Cow<'a, str>),
    /// `CHARSET`: the character set of the value (vCard 2.1).
    Charset(Cow<'a, str>),
    /// `CREATED`: the timestamp of the property's creation.
    Created(Cow<'a, str>),
    /// `DERIVED`: whether the value derives from other properties.
    Derived(Cow<'a, str>),
    /// `ENCODING`: the inline encoding of the value (vCard 2.1 / 3.0).
    Encoding(Cow<'a, str>),
    /// `GEO`: a global positioning value for the property.
    Geo(Cow<'a, str>),
    /// `JSPTR`: the JSON pointer locating a preserved JSContact property.
    Jsptr(Cow<'a, str>),
    /// `LABEL`: the formatted text of a delivery address.
    Label(Cow<'a, str>),
    /// `LANGUAGE`: the language of the property value (RFC 5646 tag).
    Language(Cow<'a, str>),
    /// `MEDIATYPE`: the media type of the referenced resource.
    MediaType(Cow<'a, str>),
    /// `PHONETIC`: the phonetic system the value is written in.
    Phonetic(Cow<'a, str>),
    /// `PID`: the source identifiers of this property instance.
    Pid(Vec<Cow<'a, str>>),
    /// `PREF`: the preference of this instance among a set (1-100).
    Pref(Cow<'a, str>),
    /// `PROP-ID`: the identity of this property instance across conversions.
    PropId(Cow<'a, str>),
    /// `SCRIPT`: the script the value is written in.
    Script(Cow<'a, str>),
    /// `SERVICE-TYPE`: the online service the property points at.
    ServiceType(Cow<'a, str>),
    /// `SORT-AS`: the components to sort the property by.
    SortAs(Vec<Cow<'a, str>>),
    /// `TYPE`: the kinds or contexts of the property (e.g. `work`, `home`).
    Type(Vec<Cow<'a, str>>),
    /// `TZ`: the time zone of the property.
    Tz(Cow<'a, str>),
    /// `USERNAME`: the username on the online service.
    Username(Cow<'a, str>),
    /// `VALUE`: the value type the property value is to be read as.
    Value(Cow<'a, str>),
    /// Any parameter the model does not decode: its name and its values.
    Unknown {
        /// The verbatim parameter name, as it was spelled on the wire.
        name: Cow<'a, str>,
        /// The `,`-separated raw values, empty when the parameter carries none.
        values: Vec<Cow<'a, str>>,
    },
}

impl VcardParam<'_> {
    /// The closed [`VcardParamKind`] of this parameter, or `None` for
    /// [`Unknown`](VcardParam::Unknown) (which is outside the vocabulary).
    pub fn kind(&self) -> Option<VcardParamKind> {
        match self {
            Self::AltId(_) => Some(VcardParamKind::AltId),
            Self::Author(_) => Some(VcardParamKind::Author),
            Self::AuthorName(_) => Some(VcardParamKind::AuthorName),
            Self::CalScale(_) => Some(VcardParamKind::CalScale),
            Self::Charset(_) => Some(VcardParamKind::Charset),
            Self::Created(_) => Some(VcardParamKind::Created),
            Self::Derived(_) => Some(VcardParamKind::Derived),
            Self::Encoding(_) => Some(VcardParamKind::Encoding),
            Self::Geo(_) => Some(VcardParamKind::Geo),
            Self::Jsptr(_) => Some(VcardParamKind::Jsptr),
            Self::Label(_) => Some(VcardParamKind::Label),
            Self::Language(_) => Some(VcardParamKind::Language),
            Self::MediaType(_) => Some(VcardParamKind::MediaType),
            Self::Phonetic(_) => Some(VcardParamKind::Phonetic),
            Self::Pid(_) => Some(VcardParamKind::Pid),
            Self::Pref(_) => Some(VcardParamKind::Pref),
            Self::PropId(_) => Some(VcardParamKind::PropId),
            Self::Script(_) => Some(VcardParamKind::Script),
            Self::ServiceType(_) => Some(VcardParamKind::ServiceType),
            Self::SortAs(_) => Some(VcardParamKind::SortAs),
            Self::Type(_) => Some(VcardParamKind::Type),
            Self::Tz(_) => Some(VcardParamKind::Tz),
            Self::Username(_) => Some(VcardParamKind::Username),
            Self::Value(_) => Some(VcardParamKind::Value),
            Self::Unknown { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::param::VcardParamKind;

    #[test]
    fn round_trips_every_kind_through_its_wire_name() {
        for kind in [
            VcardParamKind::Type,
            VcardParamKind::SortAs,
            VcardParamKind::MediaType,
        ] {
            assert_eq!(VcardParamKind::from_str(&kind).ok(), Some(kind));
        }
        // NOTE: Case-insensitive on the way in; unknown names are not in the
        // vocabulary.
        assert_eq!(
            VcardParamKind::from_str("type").ok(),
            Some(VcardParamKind::Type),
        );
        assert!(VcardParamKind::from_str("X-CUSTOM").is_err());
    }
}
