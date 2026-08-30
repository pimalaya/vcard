//! # Extended and basic date-time
//!
//! RFC 7095 section 3.5 spells a date, a time and a UTC offset in the
//! extended ISO 8601 form (`1985-04-12T23:20:50Z`), where RFC 6350 section
//! 4.3 uses the basic one (`19850412T232050Z`).
//!
//! Both directions pass anything unrecognized through verbatim, and only the
//! shapes that actually differ are re-spelled: the reduced dates (`1985-04`,
//! `1985`, `--04`, `---12`) are written the same way in both formats.

use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
};

/// Re-spell an RFC 6350 basic date-and-or-time or timestamp in the RFC 7095
/// 3.5 extended format, passing anything unrecognized through verbatim.
pub(crate) fn basic_to_extended(raw: &str) -> String {
    let (date, time) = match raw.split_once('T') {
        Some((date, time)) => (date, Some(time)),
        None => (raw, None),
    };

    let date = date_to_extended(date);
    match time {
        Some(time) => format!("{date}T{}", time_to_extended(time)),
        None => date,
    }
}

/// A basic date in extended form: only the complete (`19850412`) and
/// truncated (`--0412`) shapes gain dashes; the reduced shapes (`1985-04`,
/// `1985`, `--04`, `---12`) are spelled the same in both formats.
fn date_to_extended(date: &str) -> String {
    let bytes = date.as_bytes();

    if bytes.len() == 8 && bytes.iter().all(u8::is_ascii_digit) {
        return format!("{}-{}-{}", &date[..4], &date[4..6], &date[6..]);
    }

    if let Some(rest) = date.strip_prefix("--") {
        let bytes = rest.as_bytes();
        if bytes.len() == 4 && bytes.iter().all(u8::is_ascii_digit) {
            return format!("--{}-{}", &rest[..2], &rest[2..]);
        }
    }

    date.to_string()
}

/// A basic time (with its optional zone) in extended form: colons between
/// the pairs of digits, the zone through [`offset_to_extended`].
fn time_to_extended(time: &str) -> String {
    if time.contains(':') {
        return time.to_string();
    }

    // NOTE: leading dashes are truncation markers, not a zone sign; the zone
    // starts at the first non-digit after them.
    let dashes = time.len() - time.trim_start_matches('-').len();
    let (prefix, rest) = time.split_at(dashes);
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let (digits, zone) = rest.split_at(digits);

    let body = match digits.len() {
        0 | 2 => digits.to_string(),
        4 => format!("{}:{}", &digits[..2], &digits[2..]),
        6 => format!("{}:{}:{}", &digits[..2], &digits[2..4], &digits[4..]),
        _ => return time.to_string(),
    };

    format!("{prefix}{body}{}", offset_to_extended(zone))
}

/// A basic UTC offset (`-0500`) in the extended `-05:00` form; `Z`, a bare
/// `±hh` and anything unrecognized pass through.
pub(super) fn offset_to_extended(offset: &str) -> String {
    match offset.as_bytes() {
        [b'+' | b'-', rest @ ..] if rest.len() == 4 && rest.iter().all(u8::is_ascii_digit) => {
            format!("{}:{}", &offset[..3], &offset[3..])
        }
        _ => offset.to_string(),
    }
}

/// Re-spell an extended date-and-or-time or timestamp in the RFC 6350 basic
/// format, passing an already-basic value through untouched.
pub(crate) fn extended_to_basic(raw: Cow<'_, str>) -> Cow<'_, str> {
    match extended_str_to_basic(&raw) {
        Some(basic) => Cow::Owned(basic),
        None => raw,
    }
}

/// The basic re-spelling of an extended value, `None` when nothing changes.
pub(super) fn extended_str_to_basic(raw: &str) -> Option<String> {
    let (date, time) = match raw.split_once('T') {
        Some((date, time)) => (date, Some(time)),
        None => (raw, None),
    };

    let basic_date = date_to_basic(date);
    if basic_date == date && !time.is_some_and(|time| time.contains(':')) {
        return None;
    }

    let mut basic = basic_date;
    if let Some(time) = time {
        basic.push('T');
        basic.extend(time.chars().filter(|c| *c != ':'));
    }

    Some(basic)
}

/// An extended date in basic form: the inverse of [`date_to_extended`].
fn date_to_basic(date: &str) -> String {
    let bytes = date.as_bytes();

    let full = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && [0, 1, 2, 3, 5, 6, 8, 9]
            .iter()
            .all(|&i| bytes[i].is_ascii_digit());
    if full {
        return format!("{}{}{}", &date[..4], &date[5..7], &date[8..]);
    }

    let truncated = bytes.len() == 7
        && date.starts_with("--")
        && bytes[4] == b'-'
        && [2, 3, 5, 6].iter().all(|&i| bytes[i].is_ascii_digit());
    if truncated {
        return format!("--{}{}", &date[2..4], &date[5..]);
    }

    date.to_string()
}

/// An extended UTC offset in basic form: drop the colon.
pub(super) fn offset_to_basic(raw: Cow<'_, str>) -> Cow<'_, str> {
    if raw.contains(':') {
        Cow::Owned(raw.chars().filter(|c| *c != ':').collect())
    } else {
        raw
    }
}
