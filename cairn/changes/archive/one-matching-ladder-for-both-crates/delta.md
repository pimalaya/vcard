---
cairn: delta
change: one-matching-ladder-for-both-crates
---

## ADDED Requirements
### Requirement: The matching ladder

A property instance SHALL be matched across the three copies down one ladder, per name, and the rungs SHALL be consulted in this order:

1. The `PID` synchronisation identity (RFC 6350 section 7), with equality first, so two instances sharing one `PID` do not break the pair that needs no change, then `PID` alone.
2. The natural identity of a property that may occur more than once and whose value names a thing outside the card.
3. Identical serialized bytes, then equality alone, so among instances that decode alike the one that needs no rewrite is chosen.
4. Position.

A property whose value names a thing outside the card SHALL be identified by that value, the whole of it and as written: `EMAIL` by its address, `TEL` by its number, `IMPP`, `URL`, `SOURCE`, `FBURL`, `CALURI`, `CALADRURI`, `PHOTO`, `LOGO`, `SOUND`, `KEY` and `SOCIALPROFILE` by their URI, `MEMBER` and `RELATED` by the entity theirs names. Every other property SHALL carry no identity, since a property that may occur only once is already named by its name, and a property whose value is the datum would turn every edit into a replacement. A grouped name carries none either, the group being part of what tells the instance apart already.

`PID` SHALL stay above the natural identity. `PID` is metadata, so it survives a value change and a rename stays a rename, which an identity that is the value cannot do.

An identity SHALL tell an instance from its same-named siblings or it is not one: where two of them carry the same value, both fall back to their positions, and a sibling still alone with its value keeps its own. An instance carrying an identity SHALL NOT be matched with one carrying none, since the two are told apart differently and a position on one side does not answer for an identity on the other.

The identity SHALL be reported with the action, so a caller is told which member of a group is contested.

Where the identity is the value, changing the value changes the identity, so an edited address is one instance leaving and another arriving rather than a rename. Two sides that each replaced one therefore leave two instances, since neither renamed the base one and the name may repeat.

#### Scenario: Reordered instances
- GIVEN two copies whose `TEL` instances carry `PID` and appear in different orders
- WHEN they are merged
- THEN each instance is matched by its `PID`, not by its position

#### Scenario: Two instances under one PID
- GIVEN a base carrying two `TEL` instances under one `PID`, and a copy that edited one of them
- WHEN they are merged
- THEN the untouched instance keeps its bytes and only the edited one changes

#### Scenario: An interchangeable duplicate spelled two ways
- GIVEN a base carrying one property three times, one of them with a different line ending, and a copy carrying only the two identical ones
- WHEN they are merged
- THEN the copy the other two carry byte for byte survives

#### Scenario: A side that only reordered and replaced an address
- GIVEN a base holding two `EMAIL` instances, an untouched copy, and one that reordered them and replaced one address
- WHEN they are merged
- THEN the merged card is that copy, byte for byte, and nothing is reported

#### Scenario: An edit against a replacement
- GIVEN a copy that replaced Ada's `EMAIL` with Bob's and a copy that set a parameter on Ada's
- WHEN they are merged
- THEN Ada's parameter is never written onto Bob's line

#### Scenario: One address on two instances
- GIVEN a card carrying one `EMAIL` address twice, edited once
- WHEN it is merged with itself against the original
- THEN the edit lands, and nothing is reported

#### Scenario: A rename under a PID
- GIVEN a base `EMAIL;PID=1`, a copy adding a parameter and a copy changing the address
- WHEN they are merged
- THEN the card carries one `EMAIL`, with the new address and the new parameter

### Requirement: Matching normalises, writing is exact

An identity SHALL be compared normalised and written back exactly. The comparison lowercases, so a URI scheme (RFC 3986 section 3.1) and a mail host meet whichever case they were written in. What goes back on the wire is the bytes the side that wrote them wrote, never a normalised form the merge chose.

The two halves are one rule. Comparing raw bytes misses a match that is there; writing the normalised form loses the byte fidelity the whole crate is for.

A case difference in a value is still a change, since only matching normalises: a side that rewrote the case of a scheme rewrote the value, and that change lands like any other.

#### Scenario: One address in two cases
- GIVEN a base `IMPP:XMPP:ada@x.test`, a copy adding a parameter and a copy that lowercased the scheme
- WHEN they are merged
- THEN the card carries one `IMPP`, holding both parameters, spelled as the side that rewrote it spelled it

### Requirement: The base card is never mutated

The merge SHALL leave the base card untouched, and every position it carries SHALL be counted in that card. That is what makes the ladder's last rung safe: an ordinal read off the base names the same instance whenever it is resolved, however the merged card has moved under it.

The merged card is a clone of the left one and every edit lands on that clone, so nothing the merge does can renumber what a base ordinal names. A merge that mutated the base, or that numbered an action in the merged card, would need the ordinal translation ical-rs carries, which vcard-rs deliberately does not.

#### Scenario: An ordinal read off the base
- GIVEN a base carrying several same-named properties and a copy that removed one of them and edited another
- WHEN they are merged
- THEN the edit lands on the property its base ordinal names

## MODIFIED Requirements
### Requirement: Values are compared on the raw node

The merge SHALL decide whether two copies hold the same value by comparing their raw value nodes component by component, over every component of the value, never through the decoded projection, which reads only a value's first `;`-component. An identity read off a value SHALL be read from the same raw node, for the same reason.

Two components agree when they decode to the same list of items, so a difference in escaping is a difference. An absent component and an all-empty one agree, so a trailing empty component is not a change.

#### Scenario: A photo payload past the first semicolon
- GIVEN a base card carrying `PHOTO:data:image/png;base64,AAAA` and a copy carrying a different payload
- WHEN they are merged
- THEN the change is reported and lands, as one photo leaving and another arriving

#### Scenario: A title past the first semicolon
- GIVEN a base `TITLE:boss;of;nothing` and two copies rewriting what follows the second `;`
- WHEN they are merged
- THEN the divergence is reported

## REMOVED Requirements
### Requirement: Instance matching is PID, then equality, then position
