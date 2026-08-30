---
cairn: log
change: two-parameters-of-one-name-are-two-fields
landed: 2026-08-30
---

# Two parameters of one name are two fields

The field a parameter action occupied was keyed on the parameter name alone, and the replay resolved the first parameter of that name. RFC 2426 section 4 writes `TEL;TYPE=work;TYPE=voice`, so a side rewriting the first `TYPE` and a side rewriting the second contested one field: the preferred side won it, the other side's edit never reached the merged card, and a conflict was reported where the two sides had touched nothing in common. This is loss, not just noise, and `diff_params` already knew the answer, pairing same-named parameters positionally and then discarding which one it had paired.

`Slot::Param` and `Slot::ParamItems` are now `{ name, at }`, and `param_position`, `param_node_mut`, `right_param_item` and `restore_param` resolve the `at`th parameter of a name. Since the slot is derived from the action alone, the occurrence rides on the action: `ParamAdded`, `ParamRemoved`, `ParamChanged`, `ParamItemAdded` and `ParamItemRemoved` each gained an `index` field, which breaks every exhaustive pattern over them. This is the shape ical-rs already carries.

Spec updated: `merge` (ADDED: each parameter occurrence is a field of its own).
