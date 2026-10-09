//! # Replay
//!
//! The merged card starts as a clone of the left one, so the left side's
//! edits are present byte for byte, and the right side's actions are then
//! replayed onto it through the byte-preserving edit layer
//! ([`crate::tree::value`]), so every field the right side did not touch
//! keeps its exact bytes.
//!
//! An action whose slot the left side also wrote is a conflict: the left
//! side's outcome stands, except when a removal meets an update, where the
//! update wins at every granularity and whichever side it came from (data
//! survives over silent loss). A change both sides made is no conflict at
//! all, and every conflict is reported so a caller can resolve differently.
//!
//! Two new `REV` stamps are the exception: the later one's line replaces the
//! left one, or the left one stands, and nothing is reported.
//!
//! An edit that keeps the card's line indices happens in place; a removal and
//! an addition are deferred to [`Merger::finish`], where the removals run
//! first on stable indices and each addition then lands after the last line
//! sharing its name.
//!
//! A value replayed from a card of another version is re-encoded for the
//! merged card's escaping mode, so it arrives meaning what it meant, and
//! every line of the merged card but its last is terminated, so the card a
//! caller serializes reads back as itself.

use core::{mem, str};

use alloc::vec::Vec;

use crate::{
    param::VcardParam,
    prop::{VcardPropKind, cardinality::VcardPropCardinality, spec::prop_spec},
    tree::{
        codec::{mode::VcardEscaper, unescape::unescape_param},
        cst::VcardCst,
        leaf::VcardLeaf,
        line::VcardLine,
        merge::{
            VcardMergeAction, VcardMergeConflict, diff::Target, instance::Instance,
            matching::Matching, slot::Slot,
        },
        param::node::VcardParamNode,
        value::node::VcardValueNode,
    },
    value::datetime::VcardTimestamp,
    version::VcardVersion,
};

/// The merge state while the right side's actions replay onto the left clone:
/// the merged card under edit, the recorded conflicts, and the deferred
/// structural changes.
pub(super) struct Merger<'o, 'a> {
    pub(super) escaper: VcardEscaper,
    pub(super) base_instances: &'o [Instance<'a>],
    pub(super) left_instances: &'o [Instance<'a>],
    pub(super) right_instances: &'o [Instance<'a>],
    pub(super) left_matching: &'o Matching,
    pub(super) right_matching: &'o Matching,
    pub(super) left_ops: &'o [(Target, VcardMergeAction<'a>)],
    pub(super) right_ops: &'o [(Target, VcardMergeAction<'a>)],
    pub(super) merged: VcardCst<'a>,
    pub(super) conflicts: Vec<VcardMergeConflict<'a>>,
    pub(super) removals: Vec<usize>,
    pub(super) additions: Vec<VcardLine<'a>>,
    pub(super) readded: Vec<usize>,
}

impl<'a> Merger<'_, 'a> {
    /// Replay one right-side action onto the merged card, or record the
    /// conflict that prevents it.
    pub(super) fn apply(&mut self, target: &Target, action: &VcardMergeAction<'a>) {
        match target {
            Target::Pair { base, side } => self.apply_pair(*base, *side, action),
            Target::Removed(base) => self.apply_removed(*base, action),
            Target::Added(side) => self.apply_added(*side, action),
        }
    }

    /// Run the deferred structural changes and return the merged card with
    /// the recorded conflicts.
    pub(super) fn finish(mut self) -> (VcardCst<'a>, Vec<VcardMergeConflict<'a>>) {
        self.removals.sort_unstable();
        self.removals.dedup();
        for &line in self.removals.iter().rev() {
            self.merged.props.remove(line);
        }

        for line in mem::take(&mut self.additions) {
            // NOTE: only the outer card's lines count here: an embedded card
            // carries its own properties, and with an agent present the last
            // line of a given name is usually the agent's, which is where an
            // addition to the outer card used to land.
            let embedded = self.embedded();

            let after = self
                .merged
                .props
                .iter()
                .enumerate()
                .filter(|(i, _)| !embedded[*i])
                .rev()
                .find(|(_, prop)| prop.name.get().eq_ignore_ascii_case(line.name.get()))
                .map(|(i, _)| i);

            match after {
                Some(i) => self.merged.props.insert(i + 1, line),
                None => self.merged.props.push(line),
            }
        }

        self.merged.terminate_lines();

        (self.merged, self.conflicts)
    }

    /// Replay a right-side edit of a matched property.
    fn apply_pair(&mut self, b: usize, r: usize, action: &VcardMergeAction<'a>) {
        // NOTE: the left side removed the property the right side edited: a
        // remove-update conflict, resolved for the update by restoring the
        // right side's whole line.
        if self.left_matching.removed.contains(&b) {
            if !self.readded.contains(&b) {
                self.readded.push(b);
                let line = self.right_line(&self.right_instances[r]);
                self.additions.push(line);
                self.record(self.left_removed_action(b), action);
            }
            return;
        }

        let line = self.left_line(b);

        // NOTE: both sides restamped the card, so the later revision's line
        // stands whole, its `VALUE` saying how its own stamp reads, and the
        // other side's edits of it go with the stamp they were made beside.
        if let Some(later) = self.settles(b, r) {
            if later {
                self.merged.props[line] = self.right_line(&self.right_instances[r]);
            }
            return;
        }

        match action {
            VcardMergeAction::ValueChanged { .. } => {
                if self.already_made(b, r, action) {
                    return;
                }

                let right_value = &self.right_instances[r].node().value;

                if !self.contest(b, r, &Slot::Value, action).is_empty() {
                    return;
                }

                self.merged.props[line].value = right_value.transcode(self.escaper);
            }

            VcardMergeAction::ValueComponentChanged { component, new, .. } => {
                if self.already_made(b, r, action) {
                    return;
                }

                if !self
                    .contest(b, r, &Slot::Component(*component), action)
                    .is_empty()
                {
                    return;
                }

                self.merged.props[line].value.set_component(*component, new);
            }

            VcardMergeAction::ValueItemAdded { item, .. } => {
                if self.already_made(b, r, action) {
                    return;
                }

                if !self.contest(b, r, &Slot::Items, action).is_empty() {
                    return;
                }

                // NOTE: component zero is named on purpose here, and read the
                // same way it is written: an item is spliced as one leaf, so
                // the leaves it is looked up among must be the same ones. The
                // diff only raises an item action for a one-component value.
                let value = &mut self.merged.props[line].value;
                let present = value
                    .decode_component_list(0)
                    .iter()
                    .any(|value| value.as_ref() == item.as_ref());

                if !present {
                    value.push_value(0, item);
                }
            }

            VcardMergeAction::ValueItemRemoved { item, .. } => {
                // NOTE: a removal is not idempotent when the list holds the
                // item twice, so a side that already dropped it must not drop
                // a second copy neither side wrote off.
                if self.already_made(b, r, action) {
                    return;
                }

                if !self.contest(b, r, &Slot::Items, action).is_empty() {
                    return;
                }

                let value = &mut self.merged.props[line].value;
                let position = value
                    .decode_component_list(0)
                    .iter()
                    .position(|value| value.as_ref() == item.as_ref());

                if let Some(j) = position {
                    value.remove_value_at(0, j);
                }
            }

            VcardMergeAction::ParamAdded { index, param, .. } => {
                if self.already_made(b, r, action) {
                    return;
                }

                // NOTE: an update beats a removal here as it does at property
                // granularity, so the addition still lands when all the left
                // side did was remove the parameter.
                let slot = Slot::Param {
                    name: param.key(),
                    at: *index,
                };

                if !self
                    .contest(b, r, &slot, action)
                    .iter()
                    .all(VcardMergeAction::is_param_removal)
                {
                    return;
                }

                let Some(node) = self.right_param_node(r, param, *index).cloned() else {
                    return;
                };

                self.merged.props[line].params.push(node);
            }

            VcardMergeAction::ParamRemoved { index, param, .. } => {
                if self.already_made(b, r, action) {
                    return;
                }

                let slot = Slot::Param {
                    name: param.key(),
                    at: *index,
                };

                if !self.contest(b, r, &slot, action).is_empty() {
                    return;
                }

                let position = self.merged.props[line]
                    .params
                    .iter()
                    .position(|node| node.decode() == *param);

                if let Some(i) = position {
                    self.merged.props[line].params.remove(i);
                }
            }

            VcardMergeAction::ParamChanged {
                index, old, new, ..
            } => {
                if self.already_made(b, r, action) {
                    return;
                }

                let slot = Slot::Param {
                    name: new.key(),
                    at: *index,
                };

                let colliding = self.contest(b, r, &slot, action);
                let restore = !colliding.is_empty()
                    && colliding.iter().all(VcardMergeAction::is_param_removal);

                if !colliding.is_empty() && !restore {
                    return;
                }

                let position = self.merged.props[line]
                    .params
                    .iter()
                    .position(|node| node.decode() == *old);

                if let Some(node) = self.right_param_node(r, new, *index) {
                    match (position, restore) {
                        (Some(i), _) => self.merged.props[line].params[i] = node.clone(),
                        // NOTE: the left side removed the parameter this
                        // update rewrote, so the update brings it back.
                        (None, true) => self.merged.props[line].params.push(node.clone()),
                        (None, false) => {}
                    }
                }
            }

            VcardMergeAction::ParamItemAdded {
                index, param, item, ..
            } => {
                if self.already_made(b, r, action) {
                    return;
                }

                let leaf = self.right_param_item(r, param, *index, item);

                let Some(node) = self.merged.props[line].param_node_mut(param, *index) else {
                    self.restore_param(b, r, param, *index, action);
                    return;
                };

                let present = node
                    .values
                    .iter()
                    .any(|value| unescape_param(value.get(), node.escaper) == item.as_ref());

                if let Some(leaf) = leaf
                    && !present
                {
                    node.values.push(leaf);
                }
            }

            VcardMergeAction::ParamItemRemoved {
                index, param, item, ..
            } => {
                // NOTE: as above, a parameter may hold one item twice, and
                // `TYPE=work,,` is exactly that.
                if self.already_made(b, r, action) {
                    return;
                }

                let Some(node) = self.merged.props[line].param_node_mut(param, *index) else {
                    self.restore_param(b, r, param, *index, action);
                    return;
                };

                let position = node
                    .values
                    .iter()
                    .position(|value| unescape_param(value.get(), node.escaper) == item.as_ref());

                if let Some(i) = position {
                    node.values.remove(i);
                }
            }

            // NOTE: prop-level actions carry Removed / Added targets.
            VcardMergeAction::PropAdded { .. } | VcardMergeAction::PropRemoved { .. } => {}
        }
    }

    /// Replay a right-side property removal.
    fn apply_removed(&mut self, b: usize, action: &VcardMergeAction<'a>) {
        // NOTE: both sides removed it: already gone from the left clone.
        if self.left_matching.removed.contains(&b) {
            return;
        }

        // NOTE: the left side edited what the right side removed: an
        // update-remove conflict, resolved for the update by keeping the left
        // line as edited.
        let colliding = self.left_ops_on(b).next().map(|(_, op)| op.clone());
        if let Some(colliding) = colliding {
            self.record(colliding, action);
            return;
        }

        let line = self.left_line(b);
        self.removals.push(line);
    }

    /// Replay a right-side property addition.
    fn apply_added(&mut self, s: usize, action: &VcardMergeAction<'a>) {
        let VcardMergeAction::PropAdded { .. } = action else {
            return;
        };
        let instance = &self.right_instances[s];

        let both_added = self.left_matching.added.iter().any(|&l| {
            let left = &self.left_instances[l];
            left.key == instance.key && left.prop_eq(instance)
        });
        if both_added {
            return;
        }

        // NOTE: both sides wrote over one slot: a name allowed at most once,
        // or an instance both sides took away and each replaced. The left
        // side's copy is the one the card keeps, and the loser does not join
        // it on the card.
        let contested = self.paired_arrival(&instance.key, s).or_else(|| {
            let single = instance
                .key
                .parse::<VcardPropKind>()
                .is_ok_and(|kind| kind.at_most_one(self.merged.version()));

            match single {
                true => self
                    .left_matching
                    .added
                    .iter()
                    .copied()
                    .find(|l| self.left_instances[*l].key == instance.key),
                false => None,
            }
        });

        if let Some(l) = contested {
            let ours = &self.left_instances[l];

            // NOTE: both sides stamped a card whose base carried no revision:
            // the later stamp's line stands, as over a base one.
            if ours.key == "REV"
                && let Some(later) = later_revision(ours.node(), instance.node())
            {
                if later {
                    self.merged.props[ours.line] = self.right_line(instance);
                }
                return;
            }

            let colliding = self
                .left_ops
                .iter()
                .find(|(target, _)| matches!(target, Target::Added(x) if *x == l))
                .map(|(_, action)| action.clone());

            if let Some(colliding) = colliding {
                self.record(colliding, action);
                return;
            }
        }

        self.additions.push(self.right_line(instance));
    }

    /// The left arrival standing over the same departed base instance as `s`.
    ///
    /// A property whose identity is its own value cannot be seen to change, so
    /// an edit is a departure plus an arrival: two arrivals over one departure
    /// both sides agreed on are one instance edited twice, which collides.
    fn paired_arrival(&self, key: &str, s: usize) -> Option<usize> {
        let gone = |matching: &Matching| {
            matching
                .removed
                .iter()
                .copied()
                .filter(|b| self.base_instances[*b].key == key)
                .collect::<Vec<_>>()
        };
        let new = |matching: &Matching, instances: &[Instance<'a>]| {
            matching
                .added
                .iter()
                .copied()
                .filter(|a| instances[*a].key == key)
                .collect::<Vec<_>>()
        };

        let rank = new(self.right_matching, self.right_instances)
            .iter()
            .position(|&a| a == s)?;
        let base = *gone(self.right_matching).get(rank)?;
        let ours = gone(self.left_matching).iter().position(|&b| b == base)?;

        new(self.left_matching, self.left_instances)
            .get(ours)
            .copied()
    }

    /// Which of the merged card's lines belong to an embedded card, its
    /// `BEGIN` and `END` included.
    fn embedded(&self) -> Vec<bool> {
        let mut out = Vec::with_capacity(self.merged.props.len());
        let mut depth = 0usize;

        for line in &self.merged.props {
            let name = line.name.get();

            if name.eq_ignore_ascii_case("BEGIN") {
                depth += 1;
                out.push(true);
            } else if name.eq_ignore_ascii_case("END") {
                depth = depth.saturating_sub(1);
                out.push(true);
            } else {
                out.push(depth > 0);
            }
        }

        out
    }

    /// The left instance a base instance the left side kept was matched with.
    fn left_instance(&self, b: usize) -> &Instance<'a> {
        let pair = self
            .left_matching
            .pairs
            .iter()
            .find(|(base, _)| *base == b)
            .expect("a left-matched base instance");

        &self.left_instances[pair.1]
    }

    /// The merged (= left) line index of a base instance the left side kept.
    fn left_line(&self, b: usize) -> usize {
        self.left_instance(b).line
    }

    /// The left-side actions targeting a base instance.
    fn left_ops_on(&self, b: usize) -> impl Iterator<Item = &(Target, VcardMergeAction<'a>)> {
        targeting(self.left_ops, b)
    }

    /// Whether the left side already made the very same change, so the merged
    /// card holds it and the right action needs neither a replay nor a report.
    fn already_made(&self, b: usize, r: usize, action: &VcardMergeAction<'a>) -> bool {
        self.left_ops_on(b)
            .any(|(_, left)| left.same_change_as(action))
            && self.wrote_alike(b, r, action)
    }

    /// Whether the two sides put the same bytes on the wire for one change.
    ///
    /// A change that only takes something away wrote no bytes, and what it
    /// names lives in the base both sides share, so the change itself settles
    /// it.
    fn wrote_alike(&self, b: usize, r: usize, action: &VcardMergeAction<'a>) -> bool {
        let ours = self.left_instance(b).node();
        let theirs = self.right_instances[r].node();

        match action {
            VcardMergeAction::PropAdded { .. } | VcardMergeAction::PropRemoved { .. } => false,
            VcardMergeAction::ValueItemRemoved { .. }
            | VcardMergeAction::ParamRemoved { .. }
            | VcardMergeAction::ParamItemRemoved { .. } => true,
            VcardMergeAction::ValueChanged { .. } => {
                ours.value.raw_bytes() == theirs.value.raw_bytes()
            }
            VcardMergeAction::ValueComponentChanged { component, .. } => {
                ours.value.raw_component_list(*component)
                    == theirs.value.raw_component_list(*component)
            }
            VcardMergeAction::ValueItemAdded { item, .. } => {
                ours.value.same_item_bytes_as(&theirs.value, item)
            }
            VcardMergeAction::ParamAdded { index, param, .. }
            | VcardMergeAction::ParamChanged {
                index, new: param, ..
            } => ours.same_param_bytes_as(theirs, param, *index),
            VcardMergeAction::ParamItemAdded {
                index, param, item, ..
            } => {
                let ours = ours.raw_param_item(param, *index, item);

                ours.is_some() && ours == theirs.raw_param_item(param, *index, item)
            }
        }
    }

    /// Whether the right side made the very same change as a left action.
    fn right_made(&self, b: usize, r: usize, left: &VcardMergeAction<'a>) -> bool {
        targeting(self.right_ops, b).any(|(_, right)| right.same_change_as(left))
            && self.wrote_alike(b, r, left)
    }

    /// Record a conflict for each left-side action on the same base instance
    /// that a right action on `right` collides with, and return them: the
    /// first one on its own field, and every one contesting what the value
    /// means.
    ///
    /// Two meaning actions contest nothing. One the other side made too: both
    /// sides stood on it, so what was written beside it was written for it.
    /// And a parameter removal the other side's update of that parameter
    /// beats: the update is the meaning that stands.
    fn contest(
        &mut self,
        b: usize,
        r: usize,
        right: &Slot,
        action: &VcardMergeAction<'a>,
    ) -> Vec<VcardMergeAction<'a>> {
        let field = self
            .left_ops_on(b)
            .map(|(_, left)| left)
            .find(|left| left.slot().collides_with(right));

        let beaten = gives_way(action, self.left_ops, b);

        // NOTE: each meaning pair is its own report, so swapping the sides
        // reports as many: one parameter against two components is two pairs
        // read either way round.
        let meaning = self.left_ops_on(b).map(|(_, left)| left).filter(|left| {
            !beaten
                && left.slot().contests_meaning(right)
                && !self.right_made(b, r, left)
                && !gives_way(left, self.right_ops, b)
        });

        let colliding: Vec<VcardMergeAction<'a>> =
            field.into_iter().chain(meaning).cloned().collect();

        for left in &colliding {
            self.record(left.clone(), action);
        }

        colliding
    }

    /// Whether the right side's revision stands over the left side's, when
    /// both sides wrote a new `REV` value on one base instance.
    ///
    /// `None` where either side did not, or where [`later_revision`] cannot
    /// tell, which leaves the pair to the ordinary rules.
    fn settles(&self, b: usize, r: usize) -> Option<bool> {
        let ours = self.left_instance(b);
        let wrote = |ops| {
            targeting(ops, b)
                .any(|(_, action)| matches!(action, VcardMergeAction::ValueChanged { .. }))
        };

        if ours.key != "REV" || !wrote(self.left_ops) || !wrote(self.right_ops) {
            return None;
        }

        later_revision(ours.node(), self.right_instances[r].node())
    }

    /// The left side's removal action of a base instance.
    fn left_removed_action(&self, b: usize) -> VcardMergeAction<'a> {
        self.left_ops
            .iter()
            .find(|(target, _)| matches!(target, Target::Removed(base) if *base == b))
            .map(|(_, action)| action.clone())
            .expect("a left removal action")
    }

    /// One line of the right card, ready to land in the merged one: its value
    /// re-encoded for the merged card's escaping mode when the two differ.
    fn right_line(&self, instance: &Instance<'a>) -> VcardLine<'a> {
        let mut line = instance.node().clone();
        line.value = line.value.transcode(self.escaper);
        line
    }

    /// The right card's raw leaf for one item of the `index`th list parameter.
    ///
    /// The decoded text is no wire form: an item holds a real line break where
    /// the wire holds `^n`, so writing it back decoded would cut the line in
    /// two. The item was decoded from that node, so the leaf is there.
    fn right_param_item(
        &self,
        r: usize,
        param: &str,
        index: usize,
        item: &str,
    ) -> Option<VcardLeaf<'a>> {
        let node = self.right_instances[r].node().param_node(param, index)?;

        node.values
            .iter()
            .find(|value| unescape_param(value.get(), node.escaper) == item)
            .cloned()
    }

    /// The right card's raw parameter node, for byte-faithful replay.
    ///
    /// Addressed by name and ordinal, which is how the action was raised: a
    /// decoded parameter is no key, two same-named ones differing past their
    /// first value decoding alike.
    fn right_param_node(
        &self,
        r: usize,
        param: &VcardParam<'_>,
        index: usize,
    ) -> Option<&'a VcardParamNode<'a>> {
        self.right_instances[r]
            .node()
            .param_node(&param.key(), index)
    }

    /// Replay a right item edit whose parameter the left side removed: the
    /// update beats the removal, so the right side's whole parameter comes
    /// back and the collision is reported. Without a left culprit there is
    /// nothing to restore over, and the edit is dropped.
    fn restore_param(
        &mut self,
        b: usize,
        r: usize,
        param: &str,
        index: usize,
        action: &VcardMergeAction<'a>,
    ) {
        let key = param.to_ascii_uppercase();
        let culprit = self
            .left_ops_on(b)
            .find(|(_, action)| {
                matches!(
                    action.slot(),
                    Slot::Param { name, at } | Slot::ParamItems { name, at }
                        if name == key && at == index,
                )
            })
            .map(|(_, action)| action.clone());

        let Some(culprit) = culprit else {
            return;
        };

        let node = self.right_instances[r]
            .node()
            .param_node(param, index)
            .cloned();

        if let Some(node) = node {
            let line = self.left_line(b);
            self.merged.props[line].params.push(node);
        }

        self.record(culprit, action);
    }

    /// Record one conflict pair.
    fn record(&mut self, left: VcardMergeAction<'a>, right: &VcardMergeAction<'a>) {
        self.conflicts.push(VcardMergeConflict {
            left,
            right: right.clone(),
        });
    }
}

impl<'a> VcardValueNode<'a> {
    /// Re-encode the node for `escaper`, the escaping its new card reads.
    ///
    /// vCard 2.1 escapes only `;` while the later versions also escape a
    /// backslash, a comma and a newline, so copying across the bytes of a
    /// value replayed from another version's card would change what it means.
    fn transcode(&self, escaper: VcardEscaper) -> Self {
        if self.escaper == escaper {
            return self.clone();
        }

        let mut out = Self::from_components(Vec::new(), escaper);

        for i in 0..self.component_count() {
            out.set_component(i, &self.decode_component_list(i));
        }

        out
    }
}

impl VcardCst<'_> {
    /// Give every line of the card but its last a line ending.
    ///
    /// A card read without a trailing break leaves its final line with an
    /// empty ending, right only while that line stays last: a line serializes
    /// with nothing between its parts, so anything after it would land in its
    /// value.
    fn terminate_lines(&mut self) {
        let mut lines: Vec<&mut VcardLine<'_>> = self
            .begin
            .iter_mut()
            .chain(self.props.iter_mut())
            .chain(self.end.iter_mut())
            .collect();

        let Some((_, rest)) = lines.split_last_mut() else {
            return;
        };

        for line in rest {
            if line.eol.get().is_empty() {
                line.eol = VcardLeaf::from("\r\n");
            }
        }
    }
}

impl VcardPropKind {
    /// Whether the property may appear at most once in a card of the given
    /// version.
    fn at_most_one(self, version: VcardVersion) -> bool {
        matches!(
            (prop_spec(self).cardinality)(version),
            VcardPropCardinality::ExactlyOne | VcardPropCardinality::AtMostOne,
        )
    }
}

impl VcardMergeAction<'_> {
    /// Whether the action takes a parameter away, the one a parameter update
    /// beats.
    fn is_param_removal(&self) -> bool {
        matches!(self, Self::ParamRemoved { .. })
    }
}

/// The actions of one side that target a base instance.
fn targeting<'o, 'a>(
    ops: &'o [(Target, VcardMergeAction<'a>)],
    b: usize,
) -> impl Iterator<Item = &'o (Target, VcardMergeAction<'a>)> {
    ops.iter().filter(move |(target, _)| {
        matches!(target, Target::Pair { base, .. } | Target::Removed(base) if *base == b)
    })
}

/// Whether `theirs` stamps a later revision than `ours`.
///
/// `REV` records when the card was last revised (RFC 6350 6.7.4), so the
/// later stamp is the one standing, read as an instant whichever form the
/// version wrote it in. `None` where both wrote the same value, which is
/// agreement rather than two stamps, or where either does not read as a
/// timestamp, which leaves the two to the ordinary rules. A tie is no later
/// stamp, so the left side's stands.
fn later_revision(ours: &VcardLine<'_>, theirs: &VcardLine<'_>) -> Option<bool> {
    let (ours, theirs) = (ours.value.raw_bytes(), theirs.value.raw_bytes());

    if ours == theirs {
        return None;
    }

    let read = |raw: &[u8]| VcardTimestamp::from(str::from_utf8(raw).ok()?).to_unix_nanos();

    Some(read(&theirs)? > read(&ours)?)
}

/// Whether an action is a parameter removal the other side's update of that
/// parameter beats, so the removal is not the meaning that stands.
fn gives_way(
    action: &VcardMergeAction<'_>,
    theirs: &[(Target, VcardMergeAction<'_>)],
    b: usize,
) -> bool {
    if !action.is_param_removal() {
        return false;
    }

    let slot = action.slot();

    targeting(theirs, b).any(|(_, other)| {
        matches!(other, VcardMergeAction::ParamChanged { .. }) && other.slot() == slot
    })
}
