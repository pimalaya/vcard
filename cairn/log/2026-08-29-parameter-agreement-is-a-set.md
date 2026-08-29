---
cairn: log
change: parameter-agreement-is-a-set
landed: 2026-08-29
---

# The whole-parameter path honours identity and set semantics

Two false conflicts, one cause: the replay asked for one left action on the collided slot and compared it with `==`. A property carrying two parameters of one name yields two actions on one slot, so the right side's second action was matched against the left side's first and a card was reported as disagreeing with itself; and two sides adding one `TYPE` set in two orders were reported as disagreeing, because a whole-parameter addition compares the `Vec` inside `VcardParam::Type` in order.

The replay now asks whether any left action on the same base instance is the same change before looking for a colliding one, and a list parameter's items compare as a set. Neither lost data, but a synchronisation engine surfaced a conflict to a person for two replicas that agree, which for a multi-`TYPE` phone number is the common case.

The identity laws in tests/merge.rs and in the fuzz target now run over cards repeating a parameter name, which they used to skip.

Spec updated: `merge` (ADDED: a change both sides made is never a conflict).
