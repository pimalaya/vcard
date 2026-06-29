//! # Parameters
//!
//! A decoded parameter and the RFC 6350 parameter-name vocabulary.
//!
//! [`VcardParam`] is a closed set of the parameters defined by RFC 6350, one
//! variant each, plus an [`Unknown`](VcardParam::Unknown) arm so anything else
//! round-trips. Parameters are few and simple (a text, a list, a small integer),
//! so unlike properties each variant carries its value directly rather than
//! through a shared value type; the variant itself names the parameter. The
//! `VCARD_*` consts are the single source of truth for the wire names; the lens
//! markers in [`crate::tree::param`] reference them to match, and the decode
//! registry uses them to dispatch a raw parameter onto its variant.
//!
//! This module is pure model: no dependency on [`crate::tree`].

use alloc::{borrow::Cow, vec::Vec};

pub const VCARD_ALTID: &str = "ALTID";
pub const VCARD_CALSCALE: &str = "CALSCALE";
pub const VCARD_GEO: &str = "GEO";
pub const VCARD_LABEL: &str = "LABEL";
pub const VCARD_LANGUAGE: &str = "LANGUAGE";
pub const VCARD_MEDIATYPE: &str = "MEDIATYPE";
pub const VCARD_PID: &str = "PID";
pub const VCARD_PREF: &str = "PREF";
pub const VCARD_SORT_AS: &str = "SORT-AS";
pub const VCARD_TYPE: &str = "TYPE";
pub const VCARD_TZ: &str = "TZ";
pub const VCARD_VALUE: &str = "VALUE";

/// A decoded parameter: one known kind, or `Unknown` for anything unmodelled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardParam<'a> {
    /// `ALTID`: ties alternative representations of the same logical property.
    AltId(Cow<'a, str>),
    /// `CALSCALE`: the calendar scale of a date/time value.
    CalScale(Cow<'a, str>),
    /// `GEO`: a global positioning value for the property.
    Geo(Cow<'a, str>),
    /// `LABEL`: the formatted text of a delivery address.
    Label(Cow<'a, str>),
    /// `LANGUAGE`: the language of the property value (RFC 5646 tag).
    Language(Cow<'a, str>),
    /// `MEDIATYPE`: the media type of the referenced resource.
    MediaType(Cow<'a, str>),
    /// `PID`: the source identifiers of this property instance.
    Pid(Vec<Cow<'a, str>>),
    /// `PREF`: the preference of this instance among a set (1-100).
    Pref(Cow<'a, str>),
    /// `SORT-AS`: the components to sort the property by.
    SortAs(Vec<Cow<'a, str>>),
    /// `TYPE`: the kinds or contexts of the property (e.g. `work`, `home`).
    Type(Vec<Cow<'a, str>>),
    /// `TZ`: the time zone of the property.
    Tz(Cow<'a, str>),
    /// `VALUE`: the value type the property value is to be read as.
    Value(Cow<'a, str>),

    /// Any parameter the model does not decode.
    Unknown {
        /// The parameter name.
        name: Cow<'a, str>,
        /// The parameter values.
        values: Vec<Cow<'a, str>>,
    },
}
