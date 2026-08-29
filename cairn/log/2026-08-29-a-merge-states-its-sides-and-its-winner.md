---
cairn: log
change: a-merge-states-its-sides-and-its-winner
landed: 2026-08-29
---

# A merge states its sides, and its winner

The entry point is now `VcardMerge { base, left, right, prefer }` with a `merge(self)` method, the shape ical-rs already has. The free `merge(base, left, right)` stays as a deprecated shim over it, keeping the left preference, so its callers keep building until they migrate.

Three positional arguments could not grow. That is not a hypothetical: `prefer` is the option this crate was missing, and adding it to a free function would have broken every caller for the sake of a value most of them would have written `Left`. As a public field on a struct literal it is a breaking addition too, which is why the shim is there and why it is the shim rather than the struct that is deprecated.

`VcardMergeSide` mirrors `IcalMergeSide`, `Left` by default. What it decides is one branch: a right-side action that collided with a left-side one on the same field. Everything around that branch is untouched, and the boundary is what the six new unit tests state. A field one side alone touched still comes from that side. An untouched line still comes out byte for byte. An update still beats a removal whichever side it came from, and the preference cannot invert that, because losing data silently is not the caller's to choose. Stating `Left` gives, byte for byte, what saying nothing gives.

Two places needed more than a branch. A parameter the right side wins is written over the one it beat rather than pushed beside it, and a property added under a name the version allows once replaces the left side's addition rather than joining it. A merge that emitted both would produce a card `validate` refuses, which is a worse outcome than either side's.

`right_speaks_for` is deliberately not here. It is RFC 5546 organiser authority, and vCard has no organiser, so there is nothing for a side to speak for. That is written into the campaign note so nobody aligns it later.

The property laws now run under both preferences rather than under the default alone, and one new law says the preference changes no report: the same actions on each side, the same collided fields, the same number of conflicts. The completeness law and the differential still run under the left preference, since the naive reference implements that rule and teaching it the other one would be reimplementing the merge twice.

Capabilities moved: merge ("The winning side is chosen, not implied" ADDED; "Conflicts are reported, never silently resolved away" and "The merge obeys its algebraic laws" MODIFIED).
