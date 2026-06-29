//! # ORG lens
//!
//! The `ORG` (organization) property lens. Its value is a `;`-ordered list of
//! units, which maps cleanly onto the generic [`VcardValueCursor`]'s positional
//! component access, so it needs no bespoke cursor.

use crate::v2_1::{
    prop::VCARD_ORG,
    tree::{cursor::VcardValueCursor, line::VcardLine, prop::VcardPropLens, value::VcardValueNode},
    value::org::VcardOrg,
};

/// The `ORG` property lens.
pub struct ORG;

impl VcardPropLens for ORG {
    const NAME: &'static str = VCARD_ORG;

    type Target<'v> = VcardOrg<'v>;

    type Cursor<'c, 'a>
        = VcardValueCursor<'c, 'a>
    where
        'a: 'c;

    fn decode<'v>(line: &'v VcardLine<'_>) -> VcardOrg<'v> {
        VcardOrg::decode(line)
    }

    fn encode(decoded: &VcardOrg<'_>) -> VcardValueNode<'static> {
        decoded.encode()
    }

    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> VcardValueCursor<'c, 'a> {
        VcardValueCursor { line }
    }
}
