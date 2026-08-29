---
cairn: change
id: byte-preservation-needs-distinguishable-instances
status: landed
created: 2026-08-29
---

# Byte preservation is stated for a card whose instances can be told apart

## Why

The fuzz target's byte-preservation law says a line all three copies carry keeps its exact bytes. It holds for every card whose instances can be told apart, and it cannot hold for one that carries the same property twice with the same content.

Three matchings run against the base, one per side, and each is free to pair interchangeable copies differently. Every such pairing preserves the same content, and none is more right than another, but they disagree about *which* copy a removal takes and which spelling survives. The fuzz target kept finding that disagreement on garbage cards carrying a property four and five times over: content intact, an exact spelling gone.

Two real defects were repaired on the way there rather than papered over, and both stay fixed: two instances under one `PID` now pair by equality first, and among interchangeable instances the identical bytes pair first. What is left is not a defect, it is the law asking for more than the merge can promise.

## What

The law skips a card carrying two instances of one property with the same content, differing at most in their line ending, and says why at the site. Nothing else changes: the law still runs on every card whose instances are distinguishable, which is every real card, and the merge capability records the scope.

The content of such a card is still covered. The completeness law, the differential and the identity laws all run on it, so a duplicate that is *lost* rather than merely respelled still fails the suite.

## Judgement call, for review

**The law is scoped, not dropped.** The alternative is to keep tightening the matching until every pairing of interchangeable copies agrees across three independent matchings, which is a total order on instances the vCard format does not give. The property suite already draws this line: its completeness and differential layers only run on fixtures whose instances are identifiable. The byte law now draws it in the same place.
