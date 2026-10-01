---
cairn: log
change: building-a-card-from-another-representation
landed: 2026-10-01
---

# Building a card from another representation

The Microsoft Graph and Google People projections, which synthesize a vCard from a JSON contact, carried four helpers of their own in cardamum: a text property constructor, an RFC 6350 escaper, a splice inserting raw lines before `END:VCARD` in the serialized string, and a birthday normaliser. Moving those projections down into io-msgraph and io-gpeople would have copied the helpers into both, so the generic three moved here instead.

`VcardProp::text` (prop.rs) builds the groupless text property, for a known kind or an `X-` name. Minting the vendor `X-` lines through it makes the hand escaper redundant: the CST escapes on push. `VcardCst::push_raw` (tree/cst.rs) tokenises raw lines with `VcardLine::take` and appends them owned, which the CST serializes byte for byte since it never refolds, so it replaces the string splice with the same output. `VcardDateAndOrTime::full_date` (value/datetime.rs) is the birthday normaliser, unchanged.

The one difference from cardamum's escaper: a `\r` inside a minted value is now kept rather than dropped, as every other escaped value in the crate.

Capability moved: **editing** (building a card from another representation).
