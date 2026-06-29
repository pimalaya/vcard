//! # MEMBER lens
//!
//! The `MEMBER` property lens: a URI identifying a member of the group.

use crate::v4_0::{
    prop::VCARD_MEMBER,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::uri::VcardUri,
};

/// The `MEMBER` property lens.
pub struct MEMBER;

impl VcardPropLens for MEMBER {
    const NAME: &'static str = VCARD_MEMBER;

    type Target<'v> = VcardUri<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(value: &'v VcardValueNode<'_>) -> VcardUri<'v> {
        VcardUri::decode(value)
    }

    fn encode(decoded: &VcardUri<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
