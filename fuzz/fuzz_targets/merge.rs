#![no_main]

//! Coverage-guided fuzz target for the three-way merge.
//!
//! The oracles are the merge's algebraic laws, which hold for any three cards:
//! the merged card reparses to a byte-stable fixpoint unless it is empty or a
//! degenerate record, an untouched side contributes nothing, and two identical
//! edits are not a disagreement.
//!
//! A line all three copies carry keeps its bytes, as long as the card's
//! instances can be told apart. Each exemption is explained at the site that
//! carries it.
//!
//! The three cards are carved from one input, so the mutator naturally
//! produces related copies rather than three unrelated cards, which is where
//! reconciliation actually happens.

use std::collections::BTreeMap;

use libfuzzer_sys::fuzz_target;
use vcard::tree::{
    cst::VcardCst,
    line::VcardLine,
    merge::{VcardMerge, VcardMergeReport},
};

/// Merge three cards, the shape every law below is stated at: the left side
/// is `ours` and wins a collision, the right side is `theirs`.
fn merge<'a>(
    base: &'a VcardCst<'a>,
    left: &'a VcardCst<'a>,
    right: &'a VcardCst<'a>,
) -> VcardMergeReport<'a> {
    VcardMerge { base, left, right }.merge()
}

/// Whether a card carries two interchangeable instances of one property.
///
/// Which of several copies survives a removal is not something the merge can
/// promise: the three matchings may pair them differently, every pairing
/// preserving the content. The byte-preservation law skips such cards.
fn repeats_an_instance(cst: &VcardCst<'_>) -> bool {
    let content = |line: &VcardLine<'_>| {
        line.to_string()
            .trim_end_matches(['\r', '\n'])
            .to_ascii_uppercase()
    };

    let mut seen: Vec<String> = Vec::new();

    cst.props.iter().any(|line| {
        let line = content(line);
        let repeated = seen.contains(&line);
        seen.push(line);
        repeated
    })
}

/// Whether a bare record carries an envelope line among its properties.
///
/// Degenerate: its bytes read as an enveloped card the moment a `BEGIN` line
/// becomes the first one, so removing an earlier property changes what the
/// same bytes describe. The fixpoint law skips it.
fn is_degenerate(cst: &VcardCst<'_>) -> bool {
    cst.begin.is_none()
        && cst.props.iter().any(|line| {
            matches!(
                line.name.get().to_ascii_uppercase().as_str(),
                "BEGIN" | "END",
            )
        })
}

/// How many times each logical line occurs in a card, as exact bytes.
fn lines(cst: &VcardCst<'_>) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();

    for line in &cst.props {
        *out.entry(line.to_string()).or_insert(0) += 1;
    }

    out
}

fuzz_target!(|cards: (&[u8], &[u8], &[u8])| {
    let (Ok(base), Ok(left), Ok(right)) = (
        VcardCst::parse(cards.0),
        VcardCst::parse(cards.1),
        VcardCst::parse(cards.2),
    ) else {
        return;
    };

    let report = merge(&base, &left, &right);

    // Whatever the merge builds must parse again, to the same bytes, unless
    // the right side removed every line there was (nothing is not a card) or
    // the result is a bare record carrying an envelope line, whose bytes
    // describe a different document as soon as one becomes the first line.
    let bytes = report.merged.to_bytes();

    if !bytes.is_empty() && !is_degenerate(&report.merged) {
        let reparsed = VcardCst::parse(&bytes).expect("the merged card must reparse");
        assert_eq!(
            reparsed.to_bytes(),
            bytes,
            "the merged card is not a fixpoint"
        );
    }

    // A line all three copies carry, nobody touched: it keeps its bytes,
    // unless a card carries interchangeable copies of one property, where no
    // pairing of them is more right than another.
    let interchangeable = [&base, &left, &right]
        .iter()
        .any(|cst| repeats_an_instance(cst));

    let (b, l, r, m) = (
        lines(&base),
        lines(&left),
        lines(&right),
        lines(&report.merged),
    );

    for (line, count) in b.iter().filter(|_| !interchangeable) {
        let kept = (*count)
            .min(l.get(line).copied().unwrap_or(0))
            .min(r.get(line).copied().unwrap_or(0));

        let mut held = m.get(line).copied().unwrap_or(0);

        // NOTE: a line a source file left unterminated gains the default
        // ending when it stops being last, which is framing, not content.
        if !line.ends_with('\n') {
            held += m.get(&format!("{line}\r\n")).copied().unwrap_or(0);
        }

        assert!(held >= kept, "an untouched line lost its bytes: {line:?}");
    }

    // An untouched right side contributes nothing.
    let untouched = merge(&base, &left, &base);
    assert_eq!(untouched.merged.to_bytes(), left.to_bytes());
    assert!(untouched.conflicts.is_empty());
    assert!(untouched.right.is_empty());

    // Two identical edits are not a disagreement.
    let twin = merge(&base, &left, &left);
    assert_eq!(twin.merged.to_bytes(), left.to_bytes());
    assert!(twin.conflicts.is_empty());

    // Merging a card with itself against itself is the identity.
    let same = merge(&base, &base, &base);
    assert_eq!(same.merged.to_bytes(), base.to_bytes());
    assert!(same.left.is_empty());
    assert!(same.right.is_empty());
    assert!(same.conflicts.is_empty());
});
