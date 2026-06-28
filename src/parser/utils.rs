use core::fmt::{self, Formatter};

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::parser::{leaf::VcardLeaf, param::node::VcardParamNode};

/// The property name (before any parameters) of a content-line head.
pub(crate) fn property_name(head: &str) -> &str {
    match head.find(';') {
        Some(semi) => &head[..semi],
        None => head,
    }
}

/// Parse a content-line head into its name and parameters. The head is the part
/// before the colon, for example `EMAIL;TYPE=home`; the name is everything
/// before the first `;`, the rest splits into `;`-separated parameters.
pub(crate) fn parse_params(head: &str) -> (&str, Vec<VcardParamNode<'_>>) {
    let (name, mut rest) = match head.find(';') {
        Some(semi) => (&head[..semi], &head[semi..]),
        None => return (head, Vec::new()),
    };

    let mut params = Vec::new();

    while let Some(after) = rest.strip_prefix(';') {
        let (param, tail) = match after.find(';') {
            Some(semi) => (&after[..semi], &after[semi..]),
            None => (after, ""),
        };

        params.push(VcardParamNode::parse(param));
        rest = tail;
    }

    (name, params)
}

/// Split a value into its `K` `;`-separated components (escape-aware), padding
/// with empty slices when fewer are present.
pub(crate) fn components<const K: usize>(value: &str) -> [&str; K] {
    let bytes = value.as_bytes();
    let mut out: [&str; K] = [""; K];
    let mut start = 0;

    for slot in out.iter_mut().take(K - 1) {
        match next_unescaped(bytes, start, b';') {
            Some(semi) => {
                *slot = &value[start..semi];
                start = semi + 1;
            }
            None => {
                *slot = &value[start..];
                return out;
            }
        }
    }

    out[K - 1] = &value[start..];
    out
}

/// Split a component into its `,`-separated value leaves (escape-aware), always
/// yielding at least one (possibly empty) leaf.
pub(crate) fn value_leaves(component: &str) -> Vec<VcardLeaf<'_>> {
    let bytes = component.as_bytes();
    let mut leaves = Vec::new();
    let mut start = 0;

    while let Some(comma) = next_unescaped(bytes, start, b',') {
        leaves.push(VcardLeaf::from(&component[start..comma]));
        start = comma + 1;
    }
    leaves.push(VcardLeaf::from(&component[start..]));

    leaves
}

/// Split a parameter value list into its `,`-separated value leaves, treating a
/// comma inside double quotes as literal. Parameter values are quoted, not
/// backslash-escaped, so this is the parameter counterpart to `value_leaves`.
pub(crate) fn param_values(values: &str) -> Vec<VcardLeaf<'_>> {
    let bytes = values.as_bytes();
    let mut leaves = Vec::new();
    let mut start = 0;
    let mut quoted = false;

    for (i, &byte) in bytes.iter().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b',' if !quoted => {
                leaves.push(VcardLeaf::from(&values[start..i]));
                start = i + 1;
            }
            _ => {}
        }
    }
    leaves.push(VcardLeaf::from(&values[start..]));

    leaves
}

/// Decode value leaves into model values, resolving escapes.
pub(crate) fn decode_values<'a>(leaves: &'a [VcardLeaf<'a>]) -> Vec<Cow<'a, str>> {
    leaves.iter().map(|leaf| unescape(leaf.text())).collect()
}

/// Write `;`-separated components, each a list of `,`-separated value leaves,
/// reproducing a structured value (the N or ADR component layout).
pub(crate) fn write_components(
    f: &mut Formatter<'_>,
    components: &[&[VcardLeaf<'_>]],
) -> fmt::Result {
    for (i, component) in components.iter().enumerate() {
        if i > 0 {
            f.write_str(";")?;
        }
        write_values(f, component)?;
    }

    Ok(())
}

/// Write `,`-separated value leaves verbatim.
pub(crate) fn write_values(f: &mut Formatter<'_>, values: &[VcardLeaf<'_>]) -> fmt::Result {
    for (i, value) in values.iter().enumerate() {
        if i > 0 {
            f.write_str(",")?;
        }
        f.write_str(value.text())?;
    }

    Ok(())
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

/// The offset of the next unescaped `sep` byte at or after `start`, if any. A
/// byte preceded by a backslash is escaped and never matches.
fn next_unescaped(bytes: &[u8], start: usize, sep: u8) -> Option<usize> {
    let mut escaped = false;

    for (i, &byte) in bytes.iter().enumerate().skip(start) {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == sep {
            return Some(i);
        }
    }

    None
}
