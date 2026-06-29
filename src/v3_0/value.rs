//! # Property values
//!
//! The decoded value of a property, one variant per RFC 2426 value kind.
//!
//! [`VcardValue`] is the semantic counterpart of a content line's raw value
//! (the syntactic [`VcardValueNode`](crate::v3_0::tree::value::VcardValueNode)).
//! Most properties share a small set of value kinds: a single text, a text
//! list, a URI, a date/time, a timestamp, a UTC offset. The structured ones get
//! their own bespoke types ([`n::VcardN`], [`adr::VcardAdr`], [`org::VcardOrg`],
//! [`geo::VcardGeo`]), each in a submodule here, and the properties that carry
//! binary data (`PHOTO`, `LOGO`, `SOUND`, `KEY`) decode to
//! [`binary::VcardBinary`]. Anything the model does not decode falls back to
//! [`Unknown`](VcardValue::Unknown), which keeps the raw components so it
//! round-trips.
//!
//! These types carry no wire name and no escaping: the property name lives on
//! [`VcardProp::name`](crate::v3_0::prop::VcardProp::name), and the escaping and
//! framing live on the syntax side ([`crate::v3_0::tree`]). That keeps the whole
//! decoded model free of any dependency on `tree`, so it can be used on its
//! own.

pub mod adr;
pub mod binary;
pub mod datetime;
pub mod geo;
pub mod n;
pub mod org;
pub mod text;
pub mod uri;
pub mod utc_offset;

use alloc::{borrow::Cow, vec::Vec};

use crate::v3_0::value::{
    adr::VcardAdr,
    binary::VcardBinary,
    datetime::{VcardDateAndOrTime, VcardTimestamp},
    geo::VcardGeo,
    n::VcardN,
    org::VcardOrg,
    text::{VcardText, VcardTextList},
    uri::VcardUri,
    utc_offset::VcardUtcOffset,
};

/// A decoded property value: one known kind, or `Unknown` (raw) for anything
/// the model does not decode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardValue<'a> {
    /// The structured `ADR` value.
    Adr(VcardAdr<'a>),
    /// A binary value, inline or by URI (`PHOTO`, `LOGO`, `SOUND`, `KEY`).
    Binary(VcardBinary<'a>),
    /// A date-and-or-time (`BDAY`).
    DateAndOrTime(VcardDateAndOrTime<'a>),
    /// The structured `GEO` value.
    Geo(VcardGeo<'a>),
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
    /// A URI (`SOURCE`, `URL`).
    Uri(VcardUri<'a>),
    /// A UTC offset (`TZ`).
    UtcOffset(VcardUtcOffset<'a>),

    /// Any value the model does not decode, kept as its raw components so it
    /// round-trips.
    Unknown(VcardUnknownValue<'a>),
}

/// An undecoded property value: its unescaped components, in source order. The
/// property name lives on [`VcardProp::name`](crate::v3_0::prop::VcardProp::name).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardUnknownValue<'a> {
    /// The value, as components of values.
    pub components: Vec<Vec<Cow<'a, str>>>,
}
