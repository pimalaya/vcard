//! # JSON pointers
//!
//! The RFC 6901 pointers the `JSPROP` escape hatch is keyed by, in both
//! directions: a property preserved out of JSContact remembers where it sat,
//! and a `JSPROP` read back is grafted at that place again.
//!
//! A pointer segment escapes `~` as `~0` and `/` as `~1` (RFC 6901 section 3).

use alloc::string::String;

use serde_json::{Map, Value};
/// Graft a value onto the Card at a JSON pointer, creating intermediate
/// objects; `false` when a step lands on a non-object.
pub(super) fn set_pointer(root: &mut Map<String, Value>, pointer: &str, value: Value) -> bool {
    let Some(path) = pointer.strip_prefix('/') else {
        return false;
    };

    let mut segments = path.split('/').map(unescape_pointer);
    let mut key = match segments.next() {
        Some(key) => key,
        None => return false,
    };

    let mut current = root;
    for next in segments {
        let entry = current
            .entry(key)
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(object) = entry.as_object_mut() else {
            return false;
        };
        current = object;
        key = next;
    }

    current.insert(key, value);
    true
}

/// Undo the RFC 6901 escapes of one pointer segment.
fn unescape_pointer(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}

/// Apply the RFC 6901 escapes to one pointer segment.
pub(super) fn escape_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}
