//! # CLIENTPIDMAP lens
//!
//! The `CLIENTPIDMAP` property lens, mapping a PID source identifier to the URI
//! of the client that produced it, with a cursor naming those two components.
//!
//! See RFC 6350 6.7.7.

use alloc::borrow::Cow;

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        codec::VcardCodec,
        line::VcardLine,
        param::lens::VcardParamLens,
        prop::{lens::VcardPropLens, spec::VcardPropSpec},
    },
    value::{VcardValueKind, client_pid_map::VcardClientPidMap},
    version::VcardVersion,
};

/// The `CLIENTPIDMAP` property lens.
pub struct CLIENTPIDMAP;

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

impl VcardPropSpec for CLIENTPIDMAP {
    const KIND: VcardPropKind = VcardPropKind::ClientPidMap;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::ClientPidMap]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[]
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
