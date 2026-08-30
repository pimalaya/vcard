//! # Instances
//!
//! The unit the merge matches, diffs and replays: one decodable property line
//! of a card, remembering the card it came from.
//!
//! An instance carries the value that tells it from its same-named siblings
//! where vCard gives it one. A repeatable property whose value names a thing
//! outside the card is that thing: the address of an `EMAIL`, the URI of an
//! `IMPP` or `PHOTO`, the entity a `MEMBER` names. Any other value is the
//! datum edited, so keying on it would make every edit a replacement, and a
//! grouped name has none since the group tells it apart.
//!
//! The identity is compared lowercased and written back exactly, so a URI
//! scheme meets the other case it was written in while the line keeps its own
//! bytes. An identity a same-named sibling repeats tells neither of them
//! apart, so both fall back to their positions.
//!
//! `VERSION`, `BEGIN` and `END` are envelope rather than property, so no
//! instance is made of them and none is ever diffed, matched, moved or
//! replayed, the pair a card embedded in a vCard 2.1 `AGENT` carries
//! included: replaying an `END` would close the merged card early and drop
//! the rest.

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::{
    prop::{VcardProp, VcardPropKind},
    tree::{cst::VcardCst, line::VcardLine, merge::VcardPropPath},
};

/// One decodable property line of a card, the unit the matching pairs up.
///
/// It knows the card it belongs to and its index among that card's
/// properties, its position among the same-named ones, its uppercased
/// (group-qualified) wire name as the matching key, the identity that tells
/// it from its siblings, and the decoded property.
pub(super) struct Instance<'a> {
    pub(super) cst: &'a VcardCst<'a>,
    pub(super) line: usize,
    pub(super) nth: usize,
    pub(super) key: String,
    pub(super) identity: Option<String>,
    pub(super) prop: VcardProp<'a>,
}

impl<'a> Instance<'a> {
    /// Decode every property line of a card into a matchable instance.
    pub(super) fn all(cst: &'a VcardCst<'a>) -> Vec<Self> {
        let version = cst.version();
        let mut instances: Vec<Self> = Vec::new();

        for (line, node) in cst.props.iter().enumerate() {
            let name = node.name.get();
            if is_envelope(name) {
                continue;
            }

            let key = name.to_ascii_uppercase();
            let nth = instances
                .iter()
                .filter(|instance| instance.key == key)
                .count();
            let identity = Self::identity_of(&key, node);

            instances.push(Self {
                cst,
                line,
                nth,
                key,
                identity,
                prop: node.decode(version),
            });
        }

        // NOTE: a value written twice names both instances, so it names
        // neither: both fall back to their positions, while a sibling still
        // alone with its value keeps its own.
        let repeated: Vec<usize> = instances
            .iter()
            .enumerate()
            .filter(|(at, instance)| {
                instance.identity.is_some()
                    && instances.iter().enumerate().any(|(index, other)| {
                        index != *at
                            && other.key == instance.key
                            && other.identity == instance.identity
                    })
            })
            .map(|(at, _)| at)
            .collect();

        for at in repeated {
            instances[at].identity = None;
        }

        instances
    }

    /// The raw line the instance was decoded from.
    pub(super) fn node(&self) -> &'a VcardLine<'a> {
        &self.cst.props[self.line]
    }

    /// The path addressing the instance inside its own card.
    pub(super) fn path(&self) -> VcardPropPath<'a> {
        VcardPropPath {
            name: Cow::Borrowed(self.node().name.get()),
            index: self.nth,
            identity: self.identity.clone().map(Cow::Owned),
        }
    }

    /// Whether two instances serialize to the same bytes, line ending
    /// included.
    pub(super) fn line_eq(&self, other: &Self) -> bool {
        let (mut ours, mut theirs) = (Vec::new(), Vec::new());

        self.node().write_bytes(&mut ours);
        other.node().write_bytes(&mut theirs);

        ours == theirs
    }

    /// Whether two instances hold the same property: the same decoded name,
    /// and the same parameters and value on the raw nodes.
    pub(super) fn prop_eq(&self, other: &Self) -> bool {
        let ours = self.node();
        let theirs = other.node();

        self.prop.name == other.prop.name
            && ours.params.len() == theirs.params.len()
            && ours
                .params
                .iter()
                .zip(&theirs.params)
                .all(|(ours, theirs)| {
                    ours.name.get().eq_ignore_ascii_case(theirs.name.get())
                        && ours.same_param_as(theirs)
                })
            && ours.value.same_value_as(&theirs.value)
    }

    /// The value naming what a property is about, lowercased into a match
    /// key, for the properties vCard gives one.
    fn identity_of(key: &str, line: &VcardLine<'_>) -> Option<String> {
        let identified = matches!(
            key.parse::<VcardPropKind>(),
            Ok(VcardPropKind::CalAdrUri
                | VcardPropKind::CalUri
                | VcardPropKind::Email
                | VcardPropKind::FbUrl
                | VcardPropKind::Impp
                | VcardPropKind::Key
                | VcardPropKind::Logo
                | VcardPropKind::Member
                | VcardPropKind::Photo
                | VcardPropKind::Related
                | VcardPropKind::SocialProfile
                | VcardPropKind::Sound
                | VcardPropKind::Source
                | VcardPropKind::Tel
                | VcardPropKind::Url)
        );

        identified.then(|| String::from_utf8_lossy(&line.value.raw_bytes()).to_lowercase())
    }
}

/// Whether a line names the card's structure rather than holding a property.
fn is_envelope(name: &str) -> bool {
    name.eq_ignore_ascii_case("VERSION")
        || name.eq_ignore_ascii_case("BEGIN")
        || name.eq_ignore_ascii_case("END")
}
