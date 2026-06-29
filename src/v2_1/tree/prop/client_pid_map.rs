//! # CLIENTPIDMAP lens
//!
//! The `CLIENTPIDMAP` property lens, with a cursor naming its two components (a
//! source id and the URI of the client that produced it).

use alloc::borrow::Cow;

use crate::v2_1::{
    prop::VCARD_CLIENTPIDMAP,
    tree::{line::VcardLine, param::VcardParamLens, prop::VcardPropLens, value::VcardValueNode},
    value::client_pid_map::VcardClientPidMap,
};

/// The `CLIENTPIDMAP` property lens.
pub struct CLIENTPIDMAP;

impl VcardPropLens for CLIENTPIDMAP {
    const NAME: &'static str = VCARD_CLIENTPIDMAP;

    type Target<'v> = VcardClientPidMap<'v>;

    type Cursor<'c, 'a>
        = ClientPidMapCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardClientPidMap<'v> {
        VcardClientPidMap::decode(value)
    }

    fn encode(decoded: &VcardClientPidMap<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> ClientPidMapCursor<'c, 'a> {
        ClientPidMapCursor { line }
    }
}

/// A typed cursor over a CLIENTPIDMAP line, naming its id and URI.
pub struct ClientPidMapCursor<'c, 'a> {
    /// The borrowed content line.
    pub line: &'c mut VcardLine<'a>,
}

impl ClientPidMapCursor<'_, '_> {
    /// The whole decoded value.
    pub fn get(&self) -> VcardClientPidMap<'_> {
        VcardClientPidMap::decode(&self.line.value)
    }

    /// The PID source identifier, decoded.
    pub fn id(&self) -> Cow<'_, str> {
        self.line.value.decode_scalar_at(0)
    }

    /// Set the PID source identifier.
    pub fn set_id(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(0, &[value]);
    }

    /// The client URI, decoded.
    pub fn uri(&self) -> Cow<'_, str> {
        self.line.value.decode_scalar_at(1)
    }

    /// Set the client URI.
    pub fn set_uri(&mut self, value: impl AsRef<str>) {
        self.line.value.set_at(1, &[value]);
    }

    /// The first parameter of type `P` on this line, decoded.
    pub fn param<P: VcardParamLens>(&self) -> Option<P::Target<'_>> {
        self.line.param::<P>()
    }
}
