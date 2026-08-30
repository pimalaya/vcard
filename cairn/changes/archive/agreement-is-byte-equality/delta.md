---
cairn: delta
change: agreement-is-byte-equality
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A change both sides made is never a conflict

Before pairing a right-side action with a colliding left one, the merge SHALL check whether any left action on the same base instance is the same change, and treat that as agreement: nothing to replay, nothing to report.

Two sides SHALL be held to have made one change only where they wrote the same bytes. An unescape is not injective, since `\N` and `\n` both read as a line break (RFC 6350 section 3.4), so two changes that decode alike may say different things on the wire, and reading those as one change drops the difference without a word. What is weighed is what the change itself wrote: the value it wrote, the `;`-component it wrote, the item a list gained, the parameter it wrote. A change that only takes something away wrote no bytes, and what it names lives in the base both sides share, so the change itself settles it.

The one exception SHALL be a list parameter the specification gives no order, `TYPE` (RFC 6350 section 5.6) and `PID` (section 7), whose items compare as a set both decoded and raw.

#### Scenario: A repeated parameter name
- GIVEN a base carrying `TEL;TYPE=work;TYPE=home` and two copies that both rewrote it to `TEL;TYPE=cell;TYPE=fax`
- WHEN they are merged
- THEN the merged card is that copy and nothing is reported

#### Scenario: One type set in two orders
- GIVEN a base whose `TEL` carries no `TYPE`, a left copy adding `TYPE=work,cell` and a right copy adding `TYPE=cell,work`
- WHEN they are merged
- THEN nothing is reported

#### Scenario: One value spelled two ways
- GIVEN a base holding `FN:Ada`, a left copy holding `FN:Ada\nLovelace` and a right copy holding `FN:Ada\NLovelace`
- WHEN they are merged
- THEN the divergence is reported and the merged card is the left copy's bytes

## REMOVED Requirements
