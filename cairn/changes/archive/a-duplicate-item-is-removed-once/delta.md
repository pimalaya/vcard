---
cairn: delta
change: a-duplicate-item-is-removed-once
---

## ADDED Requirements

### Requirement: A removal both sides made takes one copy

A right-side item removal the left side already made SHALL not run again, so a list holding one item twice loses the one copy the two sides dropped rather than both.

#### Scenario: A repeated item both sides trimmed
- GIVEN a base carrying `NICKNAME:a,a` and two copies that both trimmed it to `NICKNAME:a`
- WHEN they are merged
- THEN the merged card is that copy

## MODIFIED Requirements

## REMOVED Requirements
