//! # Parameters
//!
//! A decoded parameter and the RFC 2426 parameter-name vocabulary.
//!
//! [`VcardParam`] is the closed set of parameters defined by RFC 2426, one
//! variant each, plus an [`Unknown`](VcardParam::Unknown) arm so anything else
//! round-trips. Parameters are few and simple (a single text, or the
//! `,`-separated [`Type`](VcardParam::Type) list), so unlike properties each
//! variant carries its value directly rather than through a shared value type;
//! the variant itself names the parameter. The `VCARD_*` consts are the single
//! source of truth for the wire names; the lens markers in
//! [`crate::v3_0::tree::param`] reference them to match, and the decode registry
//! uses them to dispatch a raw parameter onto its variant.
//!
//! This module is pure model: no dependency on [`crate::v3_0::tree`].

use alloc::{borrow::Cow, vec::Vec};

pub const VCARD_ENCODING: &str = "ENCODING";
pub const VCARD_LANGUAGE: &str = "LANGUAGE";
pub const VCARD_TYPE: &str = "TYPE";
pub const VCARD_VALUE: &str = "VALUE";

/// A decoded parameter: one known kind, or `Unknown` for anything unmodelled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardParam<'a> {
    /// `ENCODING`: the inline encoding of the value (`b` for base64).
    Encoding(Cow<'a, str>),
    /// `LANGUAGE`: the language of the property value (RFC 1766 tag).
    Language(Cow<'a, str>),
    /// `TYPE`: the kinds or contexts of the property (e.g. `work`, `home`,
    /// `pref`).
    Type(Vec<Cow<'a, str>>),
    /// `VALUE`: the value type the property value is to be read as (e.g. `uri`,
    /// `text`).
    Value(Cow<'a, str>),

    /// Any parameter the model does not decode.
    Unknown {
        /// The parameter name.
        name: Cow<'a, str>,
        /// The parameter values.
        values: Vec<Cow<'a, str>>,
    },
}
