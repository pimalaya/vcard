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
//! An edited line is written unfolded, which RFC 6350 3.2 permits (it
//! recommends 75 octets, it does not require them).

use alloc::{borrow::Cow, vec::Vec};

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
}

impl<'a> VcardWire<'a> {
    /// The shape of a line laid out as it reads.
    pub(crate) const fn new() -> Self {
        Self {
            parts: Vec::new(),
            len: 0,
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
    /// A shape whose sealed length no longer matches the logical line is
    /// stale, left by an edit, and is dropped: the line goes out unfolded.
    pub(crate) fn lay_out(&self, out: &mut Vec<u8>, start: usize) {
        let len = out.len() - start;

        if self.parts.is_empty() || self.len != len {
            return;
        }

        let mut at = 0;

        for (offset, part) in &self.parts {
            // NOTE: Clamped, so a shape recorded against other bytes can never
            // index out of this line or walk backwards.
            let offset = (*offset).clamp(at, len);
            out.extend_from_within(start + at..start + offset);
            part.write_bytes(out);
            at = offset;
        }

        out.extend_from_within(start + at..start + len);
        out.copy_within(start + len.., start);
        out.truncate(out.len() - len);
    }

    /// Lay a line out to the wire as [`lay_out`](Self::lay_out) does, its
    /// head (name, parameters and colon) written in `out` from `start` on and
    /// its value read from `value`, which is copied once.
    ///
    /// The pieces before the head go in front of it. A piece inside the head
    /// (a fold before the colon) takes the general way.
    pub(crate) fn lay_out_value(&self, out: &mut Vec<u8>, start: usize, value: &[u8]) {
        if self.parts.is_empty() {
            return out.extend_from_slice(value);
        }

        let head = out.len() - start;
        let len = head + value.len();

        let lead = self.parts.iter().take_while(|(at, _)| *at == 0).count();
        let (front, back) = self.parts.split_at(lead);

        if self.len != len || back.iter().any(|(at, _)| *at < head) {
            out.extend_from_slice(value);
            return self.lay_out(out, start);
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
        wire.lay_out(&mut out, 6);
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
