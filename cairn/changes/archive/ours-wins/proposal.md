---
cairn: change
id: ours-wins
status: landed
created: 2026-08-30
---

# Hard-code ours over theirs

## Why

`prefer` let a caller say which side wins a collision, apart from which side supplies the baseline bytes. The spec argued the split in as many words: the baseline is a question about bytes and the winner a question about policy, so deciding the second by the first would let a byte-fidelity choice settle a data-loss one.

The argument reads well and nobody ever used it. tCard and neverest both pass `Left`; nothing in the ecosystem passes `Right`. A caller reaches for a merge holding the version it is merging into, and that version is both the one it would rather not churn and the one it means to keep, so the two questions have one honest answer.

Git's vocabulary already names this arrangement and everybody reads it the same way: the side being merged into is `ours` and it wins, the side being merged in is `theirs`. Options with one setting are worse than the convention they obscure. ical-rs made this change first; the two crates state one merge contract, so vcard-rs follows.

## What

Remove `prefer` and `VcardMergeSide`. The left side is `ours`: the merged card is built from its bytes and keeps its value where both sides wrote one into a single field. The right side is `theirs`. Every collision is still reported, so a caller wanting the other value puts it to somebody rather than asking the merge to guess.

Done when the field and the enum are gone, the module header and the spec say the rule rather than the choice, and no caller states it.

## Consequence

One mechanism dies with it. `Merger::replaces` answered whether a right-side action a left one collided with still lands, and its answer was `!scraps && prefer == Right`. With the preference gone it is unconditionally false, so every branch guarded by it is unreachable: the replace-where-it-stood paths for a beaten parameter and for a beaten addition, and `param_position` and `VcardMergeAction::is_removal`, which nothing else called. They are removed rather than left looking live.

The one rule `replaces` never decided is untouched: an update still beats a removal, whichever side it came from. That is settled by the `readded` restore in `apply_pair` and by the `restore` branch of `ParamChanged`, neither of which ever consulted the preference.
