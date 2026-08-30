//! # Matching
//!
//! Which instance of one card is which instance of another.
//!
//! Instances of the same name are paired down one ladder, rung by rung, what
//! is left over on either side being an addition or a removal.
//!
//! `PID`, the RFC 6350 section 7 synchronisation identity, comes first,
//! paired with equality so that a card carrying two instances of one name
//! under one `PID` does not break an identical pair and rewrite two lines
//! nobody touched.
//!
//! Then comes the natural identity of a property whose value names a thing
//! outside the card. `PID` sits above it because it is metadata: it survives
//! a value change, so a rename stays a rename, which an identity that is the
//! value cannot do. An instance carrying an identity is never matched with
//! one carrying none.
//!
//! Then exact bytes, so that a card carrying an interchangeable duplicate
//! loses the copy nobody else spells that way rather than one all three
//! copies carry byte for byte, then equality, then position.
//!
//! The position rung is safe because the base card is never mutated: an
//! ordinal counted there names the same instance whenever it is resolved.

use alloc::{borrow::Cow, vec::Vec};

use crate::{param::VcardParam, prop::VcardProp, tree::merge::instance::Instance};

/// The instance pairing between the base card and one side: the matched
/// (base, side) index pairs, the side instances with no base counterpart, and
/// the base instances with no side counterpart.
pub(super) struct Matching {
    pub(super) pairs: Vec<(usize, usize)>,
    pub(super) added: Vec<usize>,
    pub(super) removed: Vec<usize>,
}

impl Matching {
    /// Pair the base instances with one side's, name by name, down the ladder.
    pub(super) fn new(base: &[Instance<'_>], side: &[Instance<'_>]) -> Self {
        let mut keys: Vec<&str> = Vec::new();
        for instance in base.iter().chain(side) {
            if !keys.contains(&instance.key.as_str()) {
                keys.push(&instance.key);
            }
        }

        let mut matching = Self {
            pairs: Vec::new(),
            added: Vec::new(),
            removed: Vec::new(),
        };

        for key in keys {
            let mut pairing = Pairing::new(base, side, key);

            pairing.pair_by(|b, s| {
                base[b].prop.shares_pid(&side[s].prop) && base[b].prop_eq(&side[s])
            });
            pairing.pair_by(|b, s| base[b].prop.shares_pid(&side[s].prop));
            pairing
                .pair_by(|b, s| base[b].identity.is_some() && base[b].identity == side[s].identity);
            pairing.pair_by(|b, s| base[b].line_eq(&side[s]));
            pairing.pair_by(|b, s| base[b].prop_eq(&side[s]));
            pairing.pair_by_position();

            matching.pairs.append(&mut pairing.pairs);
            matching.removed.append(&mut pairing.base_free);
            matching.added.append(&mut pairing.side_free);
        }

        matching
    }
}

/// The instances of one name still free on either side, and the pairs formed
/// among them so far.
struct Pairing<'i, 'a> {
    base: &'i [Instance<'a>],
    side: &'i [Instance<'a>],
    base_free: Vec<usize>,
    side_free: Vec<usize>,
    pairs: Vec<(usize, usize)>,
}

impl<'i, 'a> Pairing<'i, 'a> {
    /// Start with every instance of one name free on both sides.
    fn new(base: &'i [Instance<'a>], side: &'i [Instance<'a>], key: &str) -> Self {
        let indices = |instances: &[Instance<'a>]| -> Vec<usize> {
            instances
                .iter()
                .enumerate()
                .filter(|(_, instance)| instance.key == key)
                .map(|(at, _)| at)
                .collect()
        };

        Self {
            base,
            side,
            base_free: indices(base),
            side_free: indices(side),
            pairs: Vec::new(),
        }
    }

    /// Greedily pair the free instances that `matches`, taking each formed
    /// pair out of the free lists.
    fn pair_by(&mut self, matches: impl Fn(usize, usize) -> bool) {
        let mut b = 0;
        while b < self.base_free.len() {
            let base = self.base_free[b];

            match self.side_free.iter().position(|&s| matches(base, s)) {
                Some(s) => self
                    .pairs
                    .push((self.base_free.remove(b), self.side_free.remove(s))),
                None => b += 1,
            }
        }
    }

    /// Pair what is left over by position, the last rung.
    ///
    /// Position only tells apart properties vCard gives no identity of their
    /// own: an address that matched nothing names an entry that left, never
    /// one the other side renamed.
    fn pair_by_position(&mut self) {
        let (base, side) = (self.base, self.side);
        let mut b = 0;

        while b < self.base_free.len() {
            if base[self.base_free[b]].identity.is_some() {
                b += 1;
                continue;
            }

            match self
                .side_free
                .iter()
                .position(|&s| side[s].identity.is_none())
            {
                Some(s) => self
                    .pairs
                    .push((self.base_free.remove(b), self.side_free.remove(s))),
                None => break,
            }
        }
    }
}

impl<'a> VcardProp<'a> {
    /// Whether two properties share at least one `PID` source identifier.
    pub(super) fn shares_pid(&self, other: &Self) -> bool {
        match (self.pids(), other.pids()) {
            (Some(ours), Some(theirs)) => ours.iter().any(|pid| theirs.contains(pid)),
            _ => false,
        }
    }

    /// The values of the property's `PID` parameter, if it carries one.
    fn pids(&self) -> Option<&[Cow<'a, str>]> {
        self.params.iter().find_map(|param| match param {
            VcardParam::Pid(values) => Some(values.as_slice()),
            _ => None,
        })
    }
}
