//! # Slots
//!
//! The field of a property instance an action occupies, the granularity at
//! which two sides' actions collide.
//!
//! Two item edits merge as a set and never collide; an item edit against a
//! whole-value change does, either way round, since one value has to go. Two
//! parameters of one name are two slots (RFC 2426 section 4), never rivals.
//!
//! A parameter that says what its property's value means collides with every
//! value-level slot, both ways, since what the other side wrote was written
//! for the old meaning: a note one side rewrote in English and the other
//! relabelled as French would otherwise come out as English text labelled
//! French, which neither side wrote.

use alloc::string::String;

use crate::tree::merge::VcardMergeAction;

/// The field of a property instance an action occupies: the whole property,
/// the whole value, one component of a structured value, the items of a list
/// value, one whole parameter, or the items of a list parameter. A parameter
/// is keyed by name and by its position among the property's parameters of
/// that name.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Slot {
    Prop,
    Value,
    Component(usize),
    Items,
    Param { name: String, at: usize },
    ParamItems { name: String, at: usize },
}

impl Slot {
    /// Whether a left action on this slot collides with a right one on
    /// `right`.
    pub(super) fn collides_with(&self, right: &Slot) -> bool {
        match (self, right) {
            (Self::Value, Self::Value | Self::Component(_) | Self::Items) => true,
            (Self::Component(_) | Self::Items, Self::Value) => true,
            (Self::Component(left), Self::Component(right)) => left == right,
            (
                Self::Param {
                    name: left,
                    at: left_at,
                }
                | Self::ParamItems {
                    name: left,
                    at: left_at,
                },
                Self::Param {
                    name: right,
                    at: right_at,
                },
            ) => left == right && left_at == right_at,
            _ => false,
        }
    }

    /// Whether an action on this slot contests one on `other` across the
    /// value and a parameter saying what the value means, either way round.
    pub(super) fn contests_meaning(&self, other: &Slot) -> bool {
        match (self, other) {
            (Self::Param { name, .. }, Self::Value | Self::Component(_) | Self::Items)
            | (Self::Value | Self::Component(_) | Self::Items, Self::Param { name, .. }) => {
                qualifies(name)
            }
            _ => false,
        }
    }
}

impl VcardMergeAction<'_> {
    /// The field the action occupies.
    pub(super) fn slot(&self) -> Slot {
        match self {
            Self::PropAdded { .. } | Self::PropRemoved { .. } => Slot::Prop,
            Self::ValueChanged { .. } => Slot::Value,
            Self::ValueComponentChanged { component, .. } => Slot::Component(*component),
            Self::ValueItemAdded { .. } | Self::ValueItemRemoved { .. } => Slot::Items,
            Self::ParamAdded { index, param, .. } | Self::ParamRemoved { index, param, .. } => {
                Slot::Param {
                    name: param.key(),
                    at: *index,
                }
            }
            Self::ParamChanged { index, new, .. } => Slot::Param {
                name: new.key(),
                at: *index,
            },
            Self::ParamItemAdded { index, param, .. }
            | Self::ParamItemRemoved { index, param, .. } => Slot::ParamItems {
                name: param.to_ascii_uppercase(),
                at: *index,
            },
        }
    }
}

/// Whether a parameter, by its key, says what its property's value means
/// rather than describing the thing the value names, so that changing it
/// contests a concurrent change to the value.
///
/// How the value is read: its type (`VALUE`, RFC 6350 5.2), its language
/// (`LANGUAGE`, 5.1), the calendar its date counts in (`CALSCALE`, 5.8), the
/// media type of what it points at (`MEDIATYPE`, 5.7), the phonetic system
/// and the script it is written in (`PHONETIC` and `SCRIPT`, RFC 9554), and
/// the vCard 2.1 and 3.0 transfer encoding and character set (`ENCODING`,
/// `CHARSET`, and the bare 2.1 `QUOTED-PRINTABLE`).
///
/// Left out as describing the thing rather than the value: `TYPE`, `PREF`,
/// `ALTID`, `PID`, `SORT-AS`, `GEO`, `TZ`, `LABEL` and the other RFC 9554
/// parameters.
fn qualifies(name: &str) -> bool {
    matches!(
        name,
        "VALUE"
            | "LANGUAGE"
            | "CALSCALE"
            | "MEDIATYPE"
            | "PHONETIC"
            | "SCRIPT"
            | "ENCODING"
            | "CHARSET"
            | "QUOTED-PRINTABLE"
    )
}
