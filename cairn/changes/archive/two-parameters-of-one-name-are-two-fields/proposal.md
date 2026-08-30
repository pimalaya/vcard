---
cairn: change
id: two-parameters-of-one-name-are-two-fields
status: landed
created: 2026-08-30
---

# Two parameters of one name are two fields, not one

## Why

`VcardMergeAction::slot` keys a parameter action on the parameter name alone, and `param_position` resolves the first parameter of that name. RFC 2426 section 4 writes a repeated name explicitly, `TEL;TYPE=work;TYPE=voice`, and the corpus carries it.

A side rewriting the first `TYPE` and a side rewriting the second therefore contest one field: the preferred side wins it, the other side's edit never reaches the merged card, and a conflict is reported where the two sides touched nothing in common. `diff_params` already pairs same-named parameters positionally, so the occurrence each action addresses is known at diff time and thrown away.

## What

`Slot::Param` and `Slot::ParamItems` become `{ name, at }`, and the replay resolves the `at`th parameter of that name rather than the first. The five parameter-carrying actions carry the occurrence in a new `index` field, since `slot` derives the slot from the action alone.

This mirrors ical-rs, whose `Slot::Param { name, at }` already addresses the occurrence.

## Judgement calls, for review

**The public enum breaks.** `ParamAdded`, `ParamRemoved`, `ParamChanged`, `ParamItemAdded` and `ParamItemRemoved` gain a field, so every exhaustive struct-variant pattern over them breaks. The crate is pre-1.0 and the alternative, carrying the occurrence beside the action in a private structure the way ical-rs does, would hide from a caller which of two same-named parameters an action addresses, which is the very thing the report has to say.

**The item actions carry the occurrence too, and it is always `0`.** Item-level diffing only runs when both sides hold exactly one parameter of that name, so `ParamItemAdded` and `ParamItemRemoved` never name a later occurrence. They carry the field anyway: it makes the collision rule uniform, and it keeps an item edit of the one `TYPE` a property started with from contesting a whole `TYPE` the other side appended beside it.
