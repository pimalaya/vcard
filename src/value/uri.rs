//! # URI value
//!
//! The decoded URI value kind.
//!
//! Backs every RFC 6350 property whose value is a URI: `SOURCE`, `PHOTO`,
//! `IMPP`, `LOGO`, `MEMBER`, `RELATED`, `SOUND`, `UID`, `KEY`, `GEO`, `URL`,
//! `FBURL`, `CALADRURI`, `CALURI`. The reference is kept verbatim as a string;
//! the crate does not parse or validate it. Pure data with no escaping
//! knowledge, like every other type in [`crate::value`]; the owning property's
//! wire name lives on [`crate::prop::VcardProp::name`].

use alloc::borrow::Cow;

/// A decoded URI value, kept verbatim.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardUri<'a>(pub Cow<'a, str>);
