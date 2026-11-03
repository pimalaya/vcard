//! # Wire shape
//!
//! What a content line looked like on the wire, kept beside the logical line it
//! parsed into.
//!
//! A real card folds at 75 octets, blank-lines its properties apart and, under
//! vCard 2.1, breaks a `QUOTED-PRINTABLE` value across physical lines. Every
//! layer above the parser wants the *logical* line, so
//! [`VcardLine::take`](crate::tree::line::VcardLine::take) resolves all three.
//!
//! [`VcardWire`] is what makes that resolution reversible: a list of byte
//! offsets into the logical line, each holding the bytes the wire carried
//! there, so serialization reproduces the input exactly rather than a
//! normalised paraphrase of it.
//!
//! ## Offsets are logical, and checked
//!
//! An offset indexes the line's logical bytes (its name, its parameters and
//! its value, exactly as `VcardLine::write_bytes` lays them out, line ending
//! excluded, which is why a blank line before the line is an insertion at
//! offset 0).
//!
//! The logical length is recorded with them, and a shape whose length no
//! longer matches is dropped rather than applied: an edit that changes a
//! value's length moves every byte after it, so the old fold points would land
//! in the wrong places.
//!
//! ## Edited and built lines fold
//!
//! An edit through a cursor cuts the shape at the first byte it changed: the
//! pieces before it are kept, the rest of the line is refolded at 75 octets
//! (RFC 6350 3.2) with the line's own whitespace and break. A built line, or
//! one whose shape went stale, is folded whole.
//!
//! A vCard 2.1 line is never refolded: 2.1 unfolds as RFC 822 does, keeping the
//! whitespace, so a fold this crate adds would change the value for a 2.1
//! reader. Nor is a `QUOTED-PRINTABLE` line, a fold after one of its `=`
//! reading back as a soft break.

use alloc::{borrow::Cow, vec::Vec};

use crate::tree::leaf::put;

/// The octets a physical line holds at most, its line ending excluded.
///
/// Refs: <https://datatracker.ietf.org/doc/html/rfc6350#section-3.2>
pub(crate) const FOLD_OCTETS: usize = 75;

/// One piece of wire the parser resolved away.
#[derive(Clone, Debug)]
pub enum VcardWirePart<'a> {
    /// An RFC 6350 3.2 fold: a line break, then the single whitespace that
    /// marked the continuation.
    Fold {
        /// Whether the break was `\r\n` rather than a bare `\n`.
        crlf: bool,
        /// The folding whitespace, a space or a tab.
        wsp: u8,
    },
    /// A `QUOTED-PRINTABLE` soft line break: an `=` and the break after it.
    Soft {
        /// Whether the break was `\r\n` rather than a bare `\n`.
        crlf: bool,
    },
    /// Bytes taken verbatim off the wire and dropped: the blank lines before a
    /// content line, the whitespace of a dangling continuation, or a trailing
    /// `=` left over from a soft break with nothing to continue.
    Skipped(Cow<'a, str>),
}

impl VcardWirePart<'_> {
    /// Write the piece back out.
    fn write_bytes(&self, out: &mut Vec<u8>) {
        match self {
            Self::Fold { crlf, wsp } => {
                write_eol(*crlf, out);
                out.push(*wsp);
            }
            Self::Soft { crlf } => {
                out.push(b'=');
                write_eol(*crlf, out);
            }
            Self::Skipped(bytes) => out.extend_from_slice(bytes.as_bytes()),
        }
    }

    /// Convert into an owned piece (`'static`).
    fn into_static(self) -> VcardWirePart<'static> {
        match self {
            Self::Fold { crlf, wsp } => VcardWirePart::Fold { crlf, wsp },
            Self::Soft { crlf } => VcardWirePart::Soft { crlf },
            Self::Skipped(bytes) => VcardWirePart::Skipped(Cow::Owned(bytes.into_owned())),
        }
    }
}

fn write_eol(crlf: bool, out: &mut Vec<u8>) {
    out.extend_from_slice(if crlf { b"\r\n" } else { b"\n" });
}

/// Fold the physical line written in `out` from `from` on, in place, every
/// physical line at most [`FOLD_OCTETS`] long and never cut inside a character.
pub(crate) fn refold(out: &mut Vec<u8>, from: usize, crlf: bool, wsp: u8) {
    let len = out.len() - from;
    let mut at = 0;
    let mut room = FOLD_OCTETS;

    while len - at > room {
        let mut cut = at + room;
        let floor = (cut - 3).max(at + 1);

        while cut > floor && out[from + cut] & 0xC0 == 0x80 {
            cut -= 1;
        }

        out.extend_from_within(from + at..from + cut);
        write_eol(crlf, out);
        out.push(wsp);

        at = cut;
        room = FOLD_OCTETS - 1;
    }

    out.extend_from_within(from + at..from + len);
    out.copy_within(from + len.., from);
    out.truncate(out.len() - len);
}

/// The wire shape of one content line: every piece the parser resolved away,
/// with the offset it sat at and the logical length those offsets index.
///
/// Empty for a line that was built rather than parsed, and for a line whose
/// wire shape *is* its logical shape (unfolded, with no blank line before it).
#[derive(Clone, Debug, Default)]
pub struct VcardWire<'a> {
    /// The pieces, in the order they occur on the wire.
    parts: Vec<(usize, VcardWirePart<'a>)>,
    /// The logical length these offsets were taken against.
    len: usize,
    /// Whether an edit [`cut`](Self::cut) the shape, its tail to refold.
    edited: bool,
}

impl<'a> VcardWire<'a> {
    /// The shape of a line laid out as it reads.
    pub(crate) const fn new() -> Self {
        Self {
            parts: Vec::new(),
            len: 0,
            edited: false,
        }
    }

    /// Whether the line's wire shape is its logical shape.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// About how many bytes the line takes on the wire, its ending aside:
    /// exact for an unedited parsed line folded with `\r\n`, a guess to size
    /// a buffer by otherwise.
    pub(crate) fn size_hint(&self) -> usize {
        self.len + 3 * self.parts.len()
    }

    /// The folds recorded, each as its offset and whether the break before it
    /// was a `\r\n`.
    pub(crate) fn folds(&self) -> impl Iterator<Item = (usize, bool)> + '_ {
        self.parts.iter().filter_map(|(offset, part)| match part {
            VcardWirePart::Fold { crlf, .. } => Some((*offset, *crlf)),
            _ => None,
        })
    }

    /// Record a fold at `offset`.
    pub(crate) fn fold(&mut self, offset: usize, crlf: bool, wsp: u8) {
        self.parts.push((offset, VcardWirePart::Fold { crlf, wsp }));
    }

    /// Record a `QUOTED-PRINTABLE` soft break at `offset`.
    pub(crate) fn soft(&mut self, offset: usize, crlf: bool) {
        self.parts.push((offset, VcardWirePart::Soft { crlf }));
    }

    /// Record bytes dropped verbatim at `offset`.
    pub(crate) fn skipped(&mut self, offset: usize, bytes: &'a str) {
        self.parts
            .push((offset, VcardWirePart::Skipped(Cow::Borrowed(bytes))));
    }

    /// Record `count` dropped `=` at `offset`, after every piece recorded at
    /// or before it, the pieces being in offset order.
    ///
    /// The dangling `=` of a `QUOTED-PRINTABLE` value, which the head
    /// splitter finds once the tokeniser recorded the rest: a soft break at
    /// the same offset or past it would otherwise go out first and swallow a
    /// line.
    pub(crate) fn insert_skipped_equals(&mut self, offset: usize, count: usize) {
        const EQUALS: &str = "================================================================";

        let bytes = match EQUALS.get(..count) {
            Some(bytes) => Cow::Borrowed(bytes),
            None => Cow::Owned("=".repeat(count)),
        };

        let at = self.parts.partition_point(|(at, _)| *at <= offset);
        self.parts
            .insert(at, (offset, VcardWirePart::Skipped(bytes)));
    }

    /// Pin the logical length the offsets were taken against.
    pub(crate) fn seal(&mut self, len: usize) {
        self.len = len;
    }

    /// Whether a line of `len` logical octets with no recorded piece goes out
    /// refolded: an edited or built line past [`FOLD_OCTETS`].
    pub(crate) fn refolds(&self, len: usize) -> bool {
        self.parts.is_empty() && (self.edited || self.len != len) && len > FOLD_OCTETS
    }

    /// Keep the pieces before `at`, the first byte an edit changed, the
    /// logical length going from `old` to `new`; the tail is refolded.
    ///
    /// A fold at `at` ends a physical line the edit left whole, so it stays. A
    /// shape already stale (`old` not its length) is left to be dropped.
    pub(crate) fn cut(&mut self, at: usize, old: usize, new: usize) {
        if self.len != old {
            return;
        }

        self.parts.retain(|(offset, part)| {
            *offset < at || (*offset == at && matches!(part, VcardWirePart::Fold { .. }))
        });
        self.len = new;
        self.edited = true;
    }

    /// Put `earlier`'s pieces before this shape's, keeping the sealed length.
    ///
    /// A piece at a shared offset belongs to the tokeniser first, so a stable
    /// sort by offset merges the two ordered lists. Concatenating alone would
    /// emit a soft break ahead of an earlier dangling `=`, swallowing a line.
    pub(crate) fn prepend(&mut self, mut earlier: VcardWire<'a>) {
        if earlier.parts.is_empty() {
            return;
        }

        if self.parts.is_empty() {
            self.parts = earlier.parts;
            return;
        }

        earlier.parts.append(&mut self.parts);
        earlier.parts.sort_by_key(|(offset, _)| *offset);
        self.parts = earlier.parts;
    }

    /// Lay the logical line written in `out` from `start` on back out to the
    /// wire, re-inserting every piece, in place.
    ///
    /// The wire bytes are written past the logical ones, then moved down over
    /// them, so no buffer is allocated for a folded line.
    ///
    /// A shape whose sealed length no longer matches the logical line is stale
    /// and dropped. `fold` gives the
    /// break to refold with, `\r\n` or not, or none when the line never folds.
    pub(crate) fn lay_out(
        &self,
        out: &mut Vec<u8>,
        start: usize,
        fold: impl FnOnce() -> Option<bool>,
    ) {
        let len = out.len() - start;
        let stale = self.len != len;

        if self.parts.is_empty() || stale {
            if (stale || self.edited)
                && len > FOLD_OCTETS
                && let Some(crlf) = fold()
            {
                refold(out, start, crlf, b' ');
            }

            return;
        }

        let mut at = 0;
        let (mut from, mut crlf, mut wsp) = (start + len, None, b' ');

        for (offset, part) in &self.parts {
            // NOTE: Clamped, so a shape recorded against other bytes can never
            // index out of this line or walk backwards.
            let offset = (*offset).clamp(at, len);
            out.extend_from_within(start + at..start + offset);
            part.write_bytes(out);
            at = offset;

            // NOTE: The physical line an edit's tail refolds from starts at
            // its folding whitespace, or past a piece written before it.
            match part {
                VcardWirePart::Fold {
                    crlf: eol,
                    wsp: space,
                } => {
                    (from, crlf, wsp) = (out.len() - 1, Some(*eol), *space);
                }
                _ => from = out.len(),
            }
        }

        out.extend_from_within(start + at..start + len);
        out.copy_within(start + len.., start);
        out.truncate(out.len() - len);

        let from = from - len;

        if self.edited
            && out.len() - from > FOLD_OCTETS
            && let Some(eol) = fold()
        {
            refold(out, from, crlf.unwrap_or(eol), wsp);
        }
    }

    /// Lay a line out to the wire as [`lay_out`](Self::lay_out) does, its
    /// head (name, parameters and colon) written in `out` from `start` on and
    /// its value read from `value`, which is copied once.
    ///
    /// The pieces before the head go in front of it. A piece inside the head
    /// (a fold before the colon), an edited shape and a stale one take the
    /// general way.
    pub(crate) fn lay_out_value(
        &self,
        out: &mut Vec<u8>,
        start: usize,
        value: &[u8],
        fold: impl FnOnce() -> Option<bool>,
    ) {
        let head = out.len() - start;
        let len = head + value.len();

        if self.parts.is_empty() && !self.edited && (self.len == len || len <= FOLD_OCTETS) {
            return put(out, value);
        }

        let lead = self.parts.iter().take_while(|(at, _)| *at == 0).count();
        let (front, back) = self.parts.split_at(lead);

        if self.parts.is_empty()
            || self.len != len
            || self.edited
            || back.iter().any(|(at, _)| *at < head)
        {
            out.extend_from_slice(value);
            return self.lay_out(out, start, fold);
        }

        if !front.is_empty() {
            for (_, part) in front {
                part.write_bytes(out);
            }

            let pieces = out.len() - start - head;
            out[start..].rotate_right(pieces);
        }

        let mut at = head;

        for (offset, part) in back {
            // NOTE: Clamped as in `lay_out`.
            let offset = (*offset).clamp(at, len);
            out.extend_from_slice(&value[at - head..offset - head]);
            part.write_bytes(out);
            at = offset;
        }

        out.extend_from_slice(&value[at - head..]);
    }

    /// Convert into an owned shape (`'static`).
    pub(crate) fn into_static(self) -> VcardWire<'static> {
        VcardWire {
            parts: self
                .parts
                .into_iter()
                .map(|(offset, part)| (offset, part.into_static()))
                .collect(),
            len: self.len,
            edited: self.edited,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use crate::tree::wire::VcardWire;

    fn written(wire: &VcardWire<'_>, logical: &[u8]) -> Vec<u8> {
        let mut out = b"before".to_vec();
        out.extend_from_slice(logical);
        wire.lay_out(&mut out, 6, || Some(true));
        let line = out.split_off(6);
        assert_eq!(out, b"before");
        line
    }

    #[test]
    fn re_inserts_a_fold_where_it_was() {
        let mut wire = VcardWire::default();
        wire.fold(3, true, b' ');
        wire.seal(6);

        assert_eq!(written(&wire, b"foobar"), b"foo\r\n bar");
    }

    #[test]
    fn re_inserts_a_blank_line_before_the_name() {
        let mut wire = VcardWire::default();
        wire.skipped(0, "\r\n");
        wire.seal(6);

        assert_eq!(written(&wire, b"foobar"), b"\r\nfoobar");
    }

    #[test]
    fn re_inserts_a_soft_break() {
        let mut wire = VcardWire::default();
        wire.soft(3, false);
        wire.seal(6);

        assert_eq!(written(&wire, b"foobar"), b"foo=\nbar");
    }

    /// What an edit leaves behind: the value grew, so every fold point after
    /// it is wrong and the whole shape has to go.
    #[test]
    fn drops_a_shape_taken_against_other_bytes() {
        let mut wire = VcardWire::default();
        wire.fold(3, true, b' ');
        wire.seal(6);

        assert_eq!(written(&wire, b"foobarbaz"), b"foobarbaz");
    }

    #[test]
    fn keeps_the_order_of_pieces_at_one_offset() {
        let mut wire = VcardWire::default();
        wire.skipped(0, "\r\n");
        wire.skipped(0, " ");
        wire.seal(3);

        assert_eq!(written(&wire, b"foo"), b"\r\n foo");
    }

    #[test]
    fn prepends_an_earlier_shape_before_a_later_one() {
        let mut earlier = VcardWire::default();
        earlier.skipped(0, "\r\n");

        let mut wire = VcardWire::default();
        wire.skipped(3, "=");
        wire.seal(3);
        wire.prepend(earlier);

        assert_eq!(written(&wire, b"foo"), b"\r\nfoo=");
    }

    /// What a value ending on two `=` leaves: a soft break past the last
    /// logical byte, and the dangling `=` that precedes it.
    #[test]
    fn orders_a_merged_shape_by_offset_rather_than_by_list() {
        let mut earlier = VcardWire::default();
        earlier.soft(4, true);

        let mut wire = VcardWire::default();
        wire.skipped(3, "=");
        wire.seal(3);
        wire.prepend(earlier);

        assert_eq!(written(&wire, b"foo"), b"foo==\r\n");
    }
}
