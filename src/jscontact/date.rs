//! # Dates
//!
//! The RFC 9553 date shapes and the RFC 6350 values they stand for.
//!
//! A JSContact anniversary is either a `Timestamp`, a complete UTC date-time,
//! or a `PartialDate`, a possibly reduced or truncated calendar date. A vCard
//! date-and-or-time that fits neither is preserved through the escape hatch
//! rather than approximated.

use alloc::{borrow::Cow, format, string::String};

use serde_json::{Map, Value, json};

use crate::jcard::datetime::{basic_to_extended, extended_to_basic};
/// A JSContact Timestamp or PartialDate back to the RFC 6350 basic format,
/// `None` when it fits neither.
pub(super) fn date_from_jscontact(value: &Value) -> Option<String> {
    let object = value.as_object()?;
    let known = |allowed: &[&str]| {
        object
            .keys()
            .all(|member| member == "@type" || allowed.contains(&member.as_str()))
    };

    if let Some(utc) = object.get("utc").and_then(Value::as_str) {
        return known(&["utc"]).then(|| extended_to_basic(Cow::Borrowed(utc)).into_owned());
    }

    if !known(&["year", "month", "day"]) {
        return None;
    }
    let part = |member: &str| object.get(member).and_then(Value::as_u64);
    match (part("year"), part("month"), part("day")) {
        (Some(year), Some(month), Some(day)) => Some(format!("{year:04}{month:02}{day:02}")),
        (Some(year), Some(month), None) => Some(format!("{year:04}-{month:02}")),
        (Some(year), None, None) => Some(format!("{year:04}")),
        (None, Some(month), Some(day)) => Some(format!("--{month:02}{day:02}")),
        (None, Some(month), None) => Some(format!("--{month:02}")),
        (None, None, Some(day)) => Some(format!("---{day:02}")),
        _ => None,
    }
}

/// An anniversary date as a JSContact Timestamp (a complete UTC date-time)
/// or PartialDate (a possibly reduced or truncated date), `None` when the
/// value fits neither.
pub(super) fn date_object(raw: &str) -> Option<Value> {
    if raw.contains('T') {
        let utc = utc_timestamp(raw)?;
        return Some(json!({ "@type": "Timestamp", "utc": utc }));
    }

    let (year, month, day) = partial_date(raw)?;
    let mut object = Map::new();
    object.insert("@type".into(), "PartialDate".into());
    for (member, part) in [("year", year), ("month", month), ("day", day)] {
        if let Some(part) = part {
            object.insert(member.into(), Value::from(part));
        }
    }
    Some(Value::Object(object))
}

/// A complete Zulu date-time re-spelled extended, `None` for anything short
/// of one (a floating or offset time cannot be a JSContact UTC timestamp).
pub(super) fn utc_timestamp(raw: &str) -> Option<String> {
    let (date, time) = raw.split_once('T')?;
    let time = time.strip_suffix('Z')?;

    let date_digits = date.bytes().filter(u8::is_ascii_digit).count();
    let time_digits = time.bytes().filter(u8::is_ascii_digit).count();
    let punctuated = date.bytes().all(|b| b.is_ascii_digit() || b == b'-')
        && time.bytes().all(|b| b.is_ascii_digit() || b == b':');
    if !punctuated || date_digits != 8 || time_digits != 6 {
        return None;
    }

    Some(basic_to_extended(raw))
}

/// The year / month / day parts of a complete, reduced or truncated RFC 6350
/// date, in its basic or extended spelling; `None` when it fits no shape.
#[allow(clippy::type_complexity)]
pub(super) fn partial_date(raw: &str) -> Option<(Option<u64>, Option<u64>, Option<u64>)> {
    let part = |digits: &str| digits.parse::<u64>().ok();

    if let Some(day) = raw.strip_prefix("---") {
        return Some((None, None, Some(part(day)?)));
    }

    if let Some(rest) = raw.strip_prefix("--") {
        return match rest.len() {
            2 => Some((None, Some(part(rest)?), None)),
            4 => Some((None, Some(part(&rest[..2])?), Some(part(&rest[2..])?))),
            5 if rest.as_bytes()[2] == b'-' => {
                Some((None, Some(part(&rest[..2])?), Some(part(&rest[3..])?)))
            }
            _ => None,
        };
    }

    match raw.len() {
        4 => Some((Some(part(raw)?), None, None)),
        7 if raw.as_bytes()[4] == b'-' => {
            Some((Some(part(&raw[..4])?), Some(part(&raw[5..])?), None))
        }
        8 => Some((
            Some(part(&raw[..4])?),
            Some(part(&raw[4..6])?),
            Some(part(&raw[6..])?),
        )),
        10 if raw.as_bytes()[4] == b'-' && raw.as_bytes()[7] == b'-' => Some((
            Some(part(&raw[..4])?),
            Some(part(&raw[5..7])?),
            Some(part(&raw[8..])?),
        )),
        _ => None,
    }
}
