//! # Wire shape
//!
//! What a content line looked like on the wire, kept beside the logical line it
//! parsed into.
//!
//! A real card folds at 75 octets, blank-lines its properties apart and, under
//! vCard 2.1, breaks a `QUOTED-PRINTABLE` value across physical lines. Every
//! layer above the parser wants the *logical* line, so
//! [`VcardLine::take`](crate::tree::line::VcardLine::take) resolves all three
//! away. [`VcardWire`] is what makes that resolution reversible: a list of
//! byte offsets into the logical line, each holding the bytes the wire carried
//! there, so serialization reproduces the input exactly rather than a
//! normalised paraphrase of it.
//!
//! ## Offsets are logical, and checked
//!
//! An offset indexes the line's logical bytes (its name, its parameters and
//! its value, exactly as `VcardLine::write_bytes` lays them out, line ending
//! excluded, which is why a blank line before the line is an insertion at
//! offset 0). The logical length is recorded with them, and a shape whose
//! length no longer matches is dropped rather than applied: an edit that
//! changes a value's length moves every byte after it, so the old fold points
//! would land in the wrong places. An edited line is written unfolded, which
//! RFC 6350 3.2 permits (it recommends 75 octets, it does not require them).

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
    /// Whether the line's wire shape is its logical shape.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
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

    /// Pin the logical length the offsets were taken against.
    pub(crate) fn seal(&mut self, len: usize) {
        self.len = len;
    }

    /// Put `earlier`'s pieces before this shape's, keeping the sealed length.
    ///
    /// The tokeniser records what it resolved (blank lines, folds, soft breaks)
    /// and the line splitter records a dangling `=` the value ends on. The two
    /// lists are each ordered, and a piece sitting at the same offset in both
    /// belongs to the tokeniser first, so a stable sort by offset merges them.
    ///
    /// The sort is not cosmetic. A value ending on two `=` gives the tokeniser
    /// a soft break past the last logical byte and the splitter a dangling `=`
    /// before it, so concatenating alone would emit the soft break first and
    /// the reparsed line would swallow the one that follows.
    pub(crate) fn prepend(&mut self, mut earlier: VcardWire<'a>) {
        if earlier.parts.is_empty() {
            return;
        }

        earlier.parts.append(&mut self.parts);
        earlier.parts.sort_by_key(|(offset, _)| *offset);
        self.parts = earlier.parts;
    }

    /// Write `logical` back to the wire, re-inserting every piece.
    ///
    /// A shape whose sealed length no longer matches `logical` is stale, left
    /// by an edit, and is dropped: the logical bytes go out unfolded.
    pub(crate) fn write_bytes(&self, logical: &[u8], out: &mut Vec<u8>) {
        if self.parts.is_empty() || self.len != logical.len() {
            out.extend_from_slice(logical);
            return;
        }

        let mut at = 0;

        for (offset, part) in &self.parts {
            // NOTE: Clamped, so a shape recorded against other bytes can never
            // index out of this line or walk backwards.
            let offset = (*offset).clamp(at, logical.len());
            out.extend_from_slice(&logical[at..offset]);
            part.write_bytes(out);
            at = offset;
        }

        out.extend_from_slice(&logical[at..]);
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
        let mut out = Vec::new();
        wire.write_bytes(logical, &mut out);
        out
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

    #[test]
    fn drops_a_shape_taken_against_other_bytes() {
        // NOTE: What an edit leaves behind: the value grew, so every fold point
        // after it is wrong and the whole shape has to go.
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

    #[test]
    fn orders_a_merged_shape_by_offset_rather_than_by_list() {
        // NOTE: What a value ending on two `=` leaves: a soft break past the
        // last logical byte, and the dangling `=` that precedes it.
        let mut earlier = VcardWire::default();
        earlier.soft(4, true);

        let mut wire = VcardWire::default();
        wire.skipped(3, "=");
        wire.seal(3);
        wire.prepend(earlier);

        assert_eq!(written(&wire, b"foo"), b"foo==\r\n");
    }
}
