---
cairn: delta
change: parameter-agreement-is-a-set
---

## ADDED Requirements

### Requirement: A change both sides made is never a conflict

Before pairing a right-side action with a colliding left one, the merge SHALL check whether any left action on the same base instance is the same change, and treat that as agreement: nothing to replay, nothing to report.

Two actions are the same change when they are equal, except that a list parameter (`TYPE`, `PID`) compares its items as an unordered set, since RFC 6350 section 5.6 gives them no order.

#### Scenario: A repeated parameter name
- GIVEN a base carrying `TEL;TYPE=work;TYPE=home` and two copies that both rewrote it to `TEL;TYPE=cell;TYPE=fax`
- WHEN they are merged
- THEN the merged card is that copy and nothing is reported

#### Scenario: One type set in two orders
- GIVEN a base whose `TEL` carries no `TYPE`, a left copy adding `TYPE=work,cell` and a right copy adding `TYPE=cell,work`
- WHEN they are merged
- THEN nothing is reported

## MODIFIED Requirements

## REMOVED Requirements
