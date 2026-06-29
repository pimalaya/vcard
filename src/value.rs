//! # Property values
//!
//! The decoded value of a property, one variant per RFC 6350 value kind.
//!
//! [`VcardValue`] is the semantic counterpart of a content line's raw value
//! (the syntactic [`VcardValueNode`](crate::tree::value::VcardValueNode)). Most
//! properties share a small set of value kinds: a single text, a text list, a
//! URI, a date/time, a timestamp, a UTC offset, a language tag. A handful are
//! genuinely structured and get their own bespoke types ([`n::VcardN`],
//! [`adr::VcardAdr`], [`gender::VcardGender`], [`org::VcardOrg`],
//! [`client_pid_map::VcardClientPidMap`]), each in a submodule here. Anything
//! the model does not decode falls back to [`Unknown`](VcardValue::Unknown),
//! which keeps the raw components so it round-trips.
//!
//! These types carry no wire name and no escaping: the property name lives on
//! [`VcardProp::name`](crate::prop::VcardProp::name), and the escaping and
//! framing live on the syntax side ([`crate::tree`]). That keeps the whole
//! decoded model free of any dependency on `tree`, so it can be used on its
//! own.

pub mod adr;
pub mod client_pid_map;
pub mod datetime;
pub mod gender;
pub mod language;
pub mod n;
pub mod org;
pub mod text;
pub mod uri;
pub mod utc_offset;

use alloc::{borrow::Cow, vec::Vec};

use crate::value::{
    adr::VcardAdr,
    client_pid_map::VcardClientPidMap,
    datetime::{VcardDateAndOrTime, VcardTimestamp},
    gender::VcardGender,
    language::VcardLanguageTag,
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
    /// The structured `CLIENTPIDMAP` value.
    ClientPidMap(VcardClientPidMap<'a>),
    /// A date-and-or-time (`BDAY`, `ANNIVERSARY`).
    DateAndOrTime(VcardDateAndOrTime<'a>),
    /// The structured `GENDER` value.
    Gender(VcardGender<'a>),
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
    Unknown(VcardUnknownValue<'a>),
}

/// An undecoded property value: its unescaped components, in source order. The
/// property name lives on [`VcardProp::name`](crate::prop::VcardProp::name).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardUnknownValue<'a> {
    /// The value, as components of values.
    pub components: Vec<Vec<Cow<'a, str>>>,
}
