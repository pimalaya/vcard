//! # Parameters
//!
//! A decoded parameter and the vCard 2.1 parameter-name vocabulary.
//!
//! [`VcardParam`] is the closed set of parameters defined by vCard 2.1, one
//! variant each, plus an [`Unknown`](VcardParam::Unknown) arm. 2.1 type values
//! are written bare (`TEL;HOME;VOICE`), so they are collected into
//! [`Type`](VcardParam::Type); the rest are `name=value`. The `VCARD_*` consts
//! are the single source of truth for the wire names; the lens markers in
//! [`crate::v2_1::tree::param`] reference them to match, and the decode registry
//! uses them to dispatch a raw parameter onto its variant.
//!
//! This module is pure model: no dependency on [`crate::v2_1::tree`].

use alloc::{borrow::Cow, vec::Vec};

pub const VCARD_CHARSET: &str = "CHARSET";
pub const VCARD_ENCODING: &str = "ENCODING";
pub const VCARD_LANGUAGE: &str = "LANGUAGE";
pub const VCARD_TYPE: &str = "TYPE";
pub const VCARD_VALUE: &str = "VALUE";

/// A decoded parameter: one known kind, or `Unknown` for anything unmodelled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardParam<'a> {
    /// `CHARSET`: the character set of the property value (default ASCII).
    Charset(Cow<'a, str>),
    /// `ENCODING`: the value encoding (`7BIT`, `8BIT`, `QUOTED-PRINTABLE`,
    /// `BASE64`; default `7BIT`).
    Encoding(Cow<'a, str>),
    /// `LANGUAGE`: the language of the property value.
    Language(Cow<'a, str>),
    /// `TYPE`: the kinds or contexts of the property, written bare in 2.1 (e.g.
    /// `HOME`, `WORK`, `VOICE`).
    Type(Vec<Cow<'a, str>>),
    /// `VALUE`: the value type the property value is to be read as (e.g. `URL`).
    Value(Cow<'a, str>),

    /// Any parameter the model does not decode.
    Unknown {
        /// The parameter name.
        name: Cow<'a, str>,
        /// The parameter values.
        values: Vec<Cow<'a, str>>,
    },
}
