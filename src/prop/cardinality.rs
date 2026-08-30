//! # Property cardinality
//!
//! How many times a property may appear in a card, per RFC 6350 section 6.

/// The RFC 6350 section 6 property multiplicity: how many times a property may
/// appear in a card. Prop multiplicity, not value structure, so it is not
/// derivable from the value kind (`FN` and `NOTE` are both text but differ).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VcardPropCardinality {
    /// Exactly one (required, single).
    ExactlyOne,
    /// At most one (optional, single).
    AtMostOne,
    /// One or more (required, repeatable).
    OneOrMore,
    /// Any number, including zero (optional, repeatable).
    Any,
}
