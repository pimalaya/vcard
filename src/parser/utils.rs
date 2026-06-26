use core::{array, ops::Range};

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::parser::{leaf::VcardLeaf, param::node::VcardParamNode};

/// The property name (before any parameters) of a content-line head.
pub fn property_name(head: &str) -> &str {
    match memchr::memchr(b';', head.as_bytes()) {
        Some(semi) => &head[..semi],
        None => head,
    }
}

/// Split a head range into its name range and its parameter range (the latter
/// leading with `;`, empty when the property carries none).
pub fn split_name_params(input: &str, prop: Range<usize>) -> (Range<usize>, Range<usize>) {
    match memchr::memchr(b';', &input.as_bytes()[prop.clone()]) {
        Some(rel) => {
            let semi = prop.start + rel;
            (prop.start..semi, semi..prop.end)
        }
        None => (prop.clone(), prop.end..prop.end),
    }
}

/// Split a parameter range (leading with `;`) into its `;`-separated parameters.
pub fn parse_params(input: &str, params: Range<usize>) -> Vec<VcardParamNode> {
    let bytes = input.as_bytes();
    let mut parsed = Vec::new();
    let mut start = params.start;

    while start < params.end {
        if bytes[start] == b';' {
            start += 1;
            continue;
        }

        let end = match memchr::memchr(b';', &bytes[start..params.end]) {
            Some(rel) => start + rel,
            None => params.end,
        };

        parsed.push(VcardParamNode::parse(input, start..end));
        start = end;
    }

    parsed
}

/// Split a value range into its `K` `;`-separated component ranges
/// (escape-aware), padding with empty ranges when fewer are present.
pub(crate) fn components<const K: usize>(input: &str, value: Range<usize>) -> [Range<usize>; K] {
    let bytes = input.as_bytes();
    let end = value.end;
    let mut ranges: [Range<usize>; K] = array::from_fn(|_| end..end);
    let mut start = value.start;

    for slot in ranges.iter_mut().take(K - 1) {
        match next_unescaped(bytes, start..end, b';') {
            Some(semi) => {
                *slot = start..semi;
                start = semi + 1;
            }
            None => {
                *slot = start..end;
                return ranges;
            }
        }
    }

    ranges[K - 1] = start..end;
    ranges
}

/// Split a component range into its `,`-separated value leaves (escape-aware),
/// always yielding at least one (possibly empty) leaf.
pub(crate) fn value_leaves(input: &str, component: Range<usize>) -> Vec<VcardLeaf> {
    let bytes = input.as_bytes();
    let mut leaves = Vec::new();
    let mut start = component.start;

    while let Some(comma) = next_unescaped(bytes, start..component.end, b',') {
        leaves.push(VcardLeaf::new(start..comma));
        start = comma + 1;
    }
    leaves.push(VcardLeaf::new(start..component.end));

    leaves
}

/// Split a parameter value range into its `,`-separated values, treating a
/// comma inside double quotes as literal. Parameter values are quoted, not
/// backslash-escaped, so this is the parameter counterpart to `value_leaves`.
pub fn split_param_values(input: &str, range: Range<usize>) -> Vec<Range<usize>> {
    let bytes = input.as_bytes();
    let mut ranges = Vec::new();
    let mut start = range.start;
    let mut quoted = false;

    for i in range.clone() {
        match bytes[i] {
            b'"' => quoted = !quoted,
            b',' if !quoted => {
                ranges.push(start..i);
                start = i + 1;
            }
            _ => {}
        }
    }
    ranges.push(start..range.end);

    ranges
}

/// Decode a list of value leaves into model values.
pub(crate) fn decode_values<'a>(leaves: &'a [VcardLeaf], input: &'a str) -> Vec<Cow<'a, str>> {
    leaves
        .iter()
        .map(|leaf| unescape(leaf.text(input)))
        .collect()
}

/// Resolve the RFC 6350 escapes `\\` `\,` `\;` `\n` in a value, borrowing when
/// there is nothing to unescape.
pub(crate) fn unescape(text: &str) -> Cow<'_, str> {
    if !text.contains('\\') {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }

    Cow::Owned(out)
}

/// The offset of the next unescaped `sep` byte in `range`, if any. A byte
/// preceded by a backslash is escaped and never matches.
fn next_unescaped(bytes: &[u8], range: Range<usize>, sep: u8) -> Option<usize> {
    let mut escaped = false;

    for i in range {
        if escaped {
            escaped = false;
        } else if bytes[i] == b'\\' {
            escaped = true;
        } else if bytes[i] == sep {
            return Some(i);
        }
    }

    None
}
