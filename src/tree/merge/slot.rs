//! # Slots
//!
//! The field of a property instance an action occupies, the granularity at
//! which two sides' actions collide.
//!
//! Two item edits merge as a set and never collide; an item edit against a
//! whole-value change does, either way round, since one value has to go. Two
//! parameters of one name are two slots (RFC 2426 section 4), never rivals.

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
