//! # CLIENTPIDMAP lens
//!
//! Reading and editing the `CLIENTPIDMAP` property in place through a cursor
//! naming its two components, the source identifier and the client URI.
//!
//! Its RFC contract sits on the marker, [`CLIENTPIDMAP`].

use alloc::borrow::Cow;

use crate::{
    prop::client_pid_map::CLIENTPIDMAP,
    tree::{
        codec::VcardCodec, line::VcardLine, param::lens::VcardParamLens, prop::lens::VcardPropLens,
    },
    value::client_pid_map::VcardClientPidMap,
};

impl VcardPropLens for CLIENTPIDMAP {
    type Target<'v> = VcardClientPidMap<'v>;

    type Cursor<'c, 'a>
        = VcardClientPidMapCursor<'c, 'a>
    where
        'a: 'c;

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardClientPidMapCursor<'c, 'a> {
        VcardClientPidMapCursor { line }
    }
}

/// A typed cursor over a CLIENTPIDMAP line, naming its id and URI.
pub struct VcardClientPidMapCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl VcardClientPidMapCursor<'_, '_> {
    /// The whole decoded value.
    pub fn get(&self) -> VcardClientPidMap<'_> {
        VcardClientPidMap::decode(&self.line.value)
    }

    /// The PID source identifier, decoded.
    pub fn id(&self) -> Cow<'_, str> {
        self.line.value.decode_component(0)
    }

    /// Set the PID source identifier.
    pub fn set_id(&mut self, value: impl AsRef<str>) {
        self.line.value.set_component(0, &[value]);
    }

    /// The client URI, decoded.
    pub fn uri(&self) -> Cow<'_, str> {
        self.line.value.decode_component(1)
    }

    /// Set the client URI.
    pub fn set_uri(&mut self, value: impl AsRef<str>) {
        self.line.value.set_component(1, &[value]);
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}
