//! # Property values
//!
//! The decoded value of a property, one variant per vCard 2.1 value kind.
//!
//! [`VcardValue`] is the semantic counterpart of a content line's raw value (the
//! syntactic [`VcardValueNode`](crate::v21::tree::value::VcardValueNode)). Most
//! properties are a single text or a URI; a few are structured ([`n::VcardN`],
//! [`adr::VcardAdr`], [`org::VcardOrg`], [`geo::VcardGeo`]) or binary
//! ([`binary::VcardBinary`]). Anything the model does not decode falls back to
//! [`Unknown`](VcardValue::Unknown), which keeps the raw `;`-components so it
//! round-trips.
//!
//! These types carry no wire name and no escaping: the property name lives on
//! [`VcardProp::name`](crate::v21::prop::VcardProp::name), and the escaping and
//! framing live on the syntax side ([`crate::v21::tree`]). That keeps the whole
//! decoded model free of any dependency on `tree`, so it can be used on its own.

pub mod adr;
pub mod binary;
pub mod datetime;
pub mod geo;
pub mod n;
pub mod org;
pub mod text;
pub mod uri;

use alloc::{borrow::Cow, vec::Vec};

use crate::v21::value::{
    adr::VcardAdr,
    binary::VcardBinary,
    datetime::{VcardDateAndOrTime, VcardTimestamp},
    geo::VcardGeo,
    n::VcardN,
    org::VcardOrg,
    text::VcardText,
    uri::VcardUri,
};

/// A decoded property value: one known kind, or `Unknown` (raw) for anything the
/// model does not decode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardValue<'a> {
    /// A single text value (`FN`, `TITLE`, `NOTE`, `TEL`, `EMAIL`, ...).
    Text(VcardText<'a>),
    /// A URI (`URL`).
    Uri(VcardUri<'a>),
    /// A date-and-or-time (`BDAY`).
    DateAndOrTime(VcardDateAndOrTime<'a>),
    /// A timestamp (`REV`).
    Timestamp(VcardTimestamp<'a>),
    /// The structured `N` value.
    N(VcardN<'a>),
    /// The structured `ADR` value.
    Adr(VcardAdr<'a>),
    /// The structured `ORG` value.
    Org(VcardOrg<'a>),
    /// The structured `GEO` value.
    Geo(VcardGeo<'a>),
    /// A binary value, inline or by URI (`PHOTO`, `LOGO`, `SOUND`, `KEY`).
    Binary(VcardBinary<'a>),
    /// Any value the model does not decode, kept as its raw components so it
    /// round-trips.
    Unknown(VcardUnknownValue<'a>),
}

/// An undecoded property value: its `;`-separated components, in source order
/// (2.1 components are single-valued). The property name lives on
/// [`VcardProp::name`](crate::v21::prop::VcardProp::name).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardUnknownValue<'a> {
    /// The value, as its `;`-separated components.
    pub components: Vec<Cow<'a, str>>,
}
