//! # Date and time values
//!
//! The decoded time-related value kinds: a date-and-or-time, and a timestamp.
//!
//! [`VcardDateAndOrTime`] (RFC 6350 4.3.4) backs `BDAY` and `ANNIVERSARY`;
//! [`VcardTimestamp`] (RFC 6350 4.3.5) backs `REV`. RFC 6350 date/time values
//! have an intricate reduced-precision grammar (omitted components, truncated
//! forms); rather than decode into broken calendar fields and risk a lossy
//! round-trip, the value is kept as its raw text. Callers that need calendar
//! semantics parse the string themselves. Pure data, no escaping; the owning
//! property's wire name lives on [`crate::prop::VcardProp::name`].

use alloc::{borrow::Cow, string::String};

/// A decoded date-and-or-time value, kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardDateAndOrTime<'a>(pub Cow<'a, str>);

impl<'a> From<&'a str> for VcardDateAndOrTime<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for VcardDateAndOrTime<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for VcardDateAndOrTime<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}

/// A decoded timestamp value, kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardTimestamp<'a>(pub Cow<'a, str>);

impl VcardTimestamp<'_> {
    /// Normalizes the RFC 6350 timestamp to seconds from the Unix epoch, so two
    /// revisions can be ordered; `None` when the text will not parse.
    ///
    /// Accepts the ISO 8601 basic (`20260711T172559Z`) and extended
    /// (`2026-07-11T17:25:59Z`) forms, a `Z` or numeric offset, and reduced
    /// precision (omitted trailing components read as zero); no zone means UTC.
    /// For ordering only: it disagrees with the derived `PartialEq`, which
    /// compares raw text, so it is not exposed as `Ord`.
    pub fn to_unix_seconds(&self) -> Option<i64> {
        parse_timestamp(self.0.as_ref())
    }
}

impl<'a> From<&'a str> for VcardTimestamp<'a> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for VcardTimestamp<'_> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<Cow<'a, str>> for VcardTimestamp<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self(value)
    }
}

/// Parses an RFC 6350 timestamp into seconds from the Unix epoch, UTC.
fn parse_timestamp(raw: &str) -> Option<i64> {
    let text = raw.trim();
    let (date, time) = match text.find(['T', 't']) {
        Some(pos) => (&text[..pos], Some(&text[pos + 1..])),
        None => (text, None),
    };

    let (year, month, day) = parse_date(date)?;
    let (hour, minute, second, offset) = match time {
        Some(time) => parse_time(time)?,
        None => (0, 0, 0, 0),
    };

    let days = days_from_civil(year, month, day);
    Some(days * 86_400 + hour * 3_600 + minute * 60 + second - offset)
}

/// Parses the date part (basic `YYYYMMDD` or extended `YYYY-MM-DD`).
fn parse_date(date: &str) -> Option<(i64, i64, i64)> {
    let digits: String = date.chars().filter(|c| *c != '-').collect();
    if digits.len() != 8 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let year = digits[0..4].parse().ok()?;
    let month = digits[4..6].parse().ok()?;
    let day = digits[6..8].parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some((year, month, day))
}

/// Parses the time part with its optional zone, returning the hour,
/// minute, second and the zone offset in seconds.
fn parse_time(time: &str) -> Option<(i64, i64, i64, i64)> {
    let (clock, offset) = if let Some(stripped) = time.strip_suffix(['Z', 'z']) {
        (stripped, 0)
    } else if let Some(sign) = time.rfind(['+', '-']) {
        (&time[..sign], parse_offset(&time[sign..])?)
    } else {
        (time, 0)
    };

    // NOTE: Trailing time components may be omitted; pad them to zero.
    let mut digits: String = clock.chars().filter(|c| *c != ':').collect();
    if digits.is_empty() || digits.len() > 6 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    while digits.len() < 6 {
        digits.push('0');
    }

    let hour = digits[0..2].parse().ok()?;
    let minute = digits[2..4].parse().ok()?;
    let second = digits[4..6].parse().ok()?;
    // NOTE: A leap second (60) is accepted and folds into the next minute.
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    Some((hour, minute, second, offset))
}

/// Parses a signed zone offset (`+02`, `+0200`, `-05:00`) into seconds.
fn parse_offset(zone: &str) -> Option<i64> {
    let sign = match zone.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };

    let digits: String = zone[1..].chars().filter(|c| *c != ':').collect();
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let (hours, minutes) = match digits.len() {
        2 => (digits[0..2].parse::<i64>().ok()?, 0),
        4 => (
            digits[0..2].parse::<i64>().ok()?,
            digits[2..4].parse::<i64>().ok()?,
        ),
        _ => return None,
    };
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 3_600 + minutes * 60))
}

/// Days from the Unix epoch to the civil date (Howard Hinnant's
/// algorithm), integer-only so it stays `no_std`.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = (if year >= 0 { year } else { year - 399 }) / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use crate::value::datetime::VcardTimestamp;

    fn seconds(raw: &str) -> Option<i64> {
        VcardTimestamp::from(raw).to_unix_seconds()
    }

    #[test]
    fn parses_basic_and_extended_utc_alike() {
        assert_eq!(seconds("20260711T172559Z"), seconds("2026-07-11T17:25:59Z"));
    }

    #[test]
    fn zone_offset_folds_into_utc() {
        // NOTE: 19:25:59+02:00 is the same instant as 17:25:59Z.
        assert_eq!(
            seconds("2026-07-11T19:25:59+02:00"),
            seconds("2026-07-11T17:25:59Z"),
        );
        assert_eq!(
            seconds("2026-07-11T12:25:59-0500"),
            seconds("2026-07-11T17:25:59Z"),
        );
        // NOTE: The hour-only form, which the offset grammar allows and nothing
        // exercised before.
        assert_eq!(
            seconds("2026-07-11T19:25:59+02"),
            seconds("2026-07-11T17:25:59Z"),
        );
    }

    #[test]
    fn a_later_revision_orders_after_an_earlier_one() {
        assert!(seconds("2026-07-11T17:25:59Z") > seconds("2026-07-11T17:08:14Z"));
        assert!(seconds("2027-01-01T00:00:00Z") > seconds("2026-12-31T23:59:59Z"));
    }

    #[test]
    fn a_missing_zone_reads_as_utc() {
        assert_eq!(
            seconds("2026-07-11T17:25:59"),
            seconds("2026-07-11T17:25:59Z")
        );
    }

    #[test]
    fn reduced_precision_pads_with_zero() {
        assert_eq!(seconds("2026-07-11T17Z"), seconds("2026-07-11T170000Z"));
        assert_eq!(seconds("20260711"), seconds("2026-07-11T000000Z"));
    }

    #[test]
    fn rejects_non_timestamps() {
        assert_eq!(seconds(""), None);
        assert_eq!(seconds("not-a-date"), None);
        assert_eq!(seconds("2026-13-11T00:00:00Z"), None);
        assert_eq!(seconds("2026-07-11T25:00:00Z"), None);
        // An offset with no sign, with non-digits, of an unusable width, or out
        // of range is not an offset.
        assert_eq!(seconds("2026-07-11T19:25:5902:00"), None);
        assert_eq!(seconds("2026-07-11T19:25:59+ab:00"), None);
        assert_eq!(seconds("2026-07-11T19:25:59+020"), None);
        assert_eq!(seconds("2026-07-11T19:25:59+24:00"), None);
        assert_eq!(seconds("2026-07-11T19:25:59+02:60"), None);
    }
}
