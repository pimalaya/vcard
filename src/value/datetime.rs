//! # Date and time values
//!
//! The decoded time-related value kinds: a date-and-or-time, and a timestamp.
//!
//! [`VcardDateAndOrTime`] (RFC 6350 4.3.4) backs `BDAY` and `ANNIVERSARY`;
//! [`VcardTimestamp`] (RFC 6350 4.3.5) backs `REV`.
//!
//! Their reduced-precision grammar is intricate (omitted components, truncated
//! forms), so the value is kept as its raw text rather than decoded into broken
//! calendar fields at the risk of a lossy round-trip. Callers needing calendar
//! semantics parse the string themselves.

use alloc::{borrow::Cow, format, string::String};

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

impl VcardDateAndOrTime<'_> {
    /// The value as a complete `yyyy-mm-dd` date, from the basic (`19850412`)
    /// or the extended (`1985-04-12`) form.
    ///
    /// `None` for a reduced-precision date, a year-less one (`--0412`) or a
    /// value carrying a time (RFC 6350 4.3.4), none of which every
    /// representation can hold.
    pub fn full_date(&self) -> Option<String> {
        let date = self.0.trim();
        let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());

        let mut parts = date.split('-');
        if let (Some(y), Some(m), Some(d), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
            && y.len() == 4
            && m.len() == 2
            && d.len() == 2
            && digits(y)
            && digits(m)
            && digits(d)
        {
            return Some(format!("{y}-{m}-{d}"));
        }

        if date.len() == 8 && digits(date) {
            return Some(format!("{}-{}-{}", &date[..4], &date[4..6], &date[6..]));
        }

        None
    }
}

/// A decoded timestamp value, kept as its raw text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VcardTimestamp<'a>(pub Cow<'a, str>);

impl VcardTimestamp<'_> {
    /// The RFC 6350 timestamp in Unix epoch seconds, `None` if unparsable.
    ///
    /// For ordering revisions. Accepts the ISO 8601 basic and extended forms,
    /// a `Z` or numeric offset, reduced precision (omitted components read as
    /// zero, no zone meaning UTC) and the fraction of a second RFC 2425 section
    /// 5.8.4 allows vCard 3.0, which is dropped. Disagrees with `PartialEq`, so
    /// no `Ord`.
    pub fn to_unix_seconds(&self) -> Option<i64> {
        parse_timestamp(self.0.as_ref()).map(|(seconds, _)| seconds)
    }

    /// The timestamp in Unix epoch nanoseconds, `None` if unparsable.
    ///
    /// The reading [`to_unix_seconds`](Self::to_unix_seconds) makes, its
    /// fraction of a second kept, so two revisions one second apart or less
    /// still order.
    pub fn to_unix_nanos(&self) -> Option<i128> {
        let (seconds, nanos) = parse_timestamp(self.0.as_ref())?;

        Some(i128::from(seconds) * 1_000_000_000 + i128::from(nanos))
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

/// Parses an RFC 6350 timestamp into seconds from the Unix epoch, UTC, and
/// the nanoseconds of its fraction.
fn parse_timestamp(raw: &str) -> Option<(i64, u32)> {
    let text = raw.trim();
    let (date, time) = match text.find(['T', 't']) {
        Some(pos) => (&text[..pos], Some(&text[pos + 1..])),
        None => (text, None),
    };

    let (year, month, day) = parse_date(date)?;
    let (hour, minute, second, nanos, offset) = match time {
        Some(time) => parse_time(time)?,
        None => (0, 0, 0, 0, 0),
    };

    let days = days_from_civil(year, month, day);
    let seconds = days * 86_400 + hour * 3_600 + minute * 60 + second - offset;

    Some((seconds, nanos))
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

/// Parses the time part with its optional zone, returning the hour, minute,
/// second, the nanoseconds of the fraction and the zone offset in seconds.
fn parse_time(time: &str) -> Option<(i64, i64, i64, u32, i64)> {
    let (clock, offset) = if let Some(stripped) = time.strip_suffix(['Z', 'z']) {
        (stripped, 0)
    } else if let Some(sign) = time.rfind(['+', '-']) {
        (&time[..sign], parse_offset(&time[sign..])?)
    } else {
        (time, 0)
    };

    let (clock, fraction) = match clock.find([',', '.']) {
        Some(at) => (&clock[..at], Some(&clock[at + 1..])),
        None => (clock, None),
    };

    // NOTE: Trailing time components may be omitted; pad them to zero.
    let mut digits: String = clock.chars().filter(|c| *c != ':').collect();
    if digits.is_empty() || digits.len() > 6 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    // NOTE: A fraction belongs to the second (RFC 2425 section 5.8.4), so a
    // time without one carries none.
    let nanos = match fraction {
        Some(fraction) if digits.len() == 6 => parse_fraction(fraction)?,
        Some(_) => return None,
        None => 0,
    };

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
    Some((hour, minute, second, nanos, offset))
}

/// Parses the digits of a fraction of a second into nanoseconds, any digit
/// past the ninth dropped.
fn parse_fraction(fraction: &str) -> Option<u32> {
    if fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let mut digits: String = fraction.chars().take(9).collect();
    while digits.len() < 9 {
        digits.push('0');
    }

    digits.parse().ok()
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

    /// 19:25:59+02:00 is the same instant as 17:25:59Z, and the hour-only
    /// offset form the grammar allows reads the same way.
    #[test]
    fn zone_offset_folds_into_utc() {
        assert_eq!(
            seconds("2026-07-11T19:25:59+02:00"),
            seconds("2026-07-11T17:25:59Z"),
        );
        assert_eq!(
            seconds("2026-07-11T12:25:59-0500"),
            seconds("2026-07-11T17:25:59Z"),
        );
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

    /// RFC 2425 section 5.8.4 lets a vCard 3.0 date-time carry a fraction of
    /// a second after a comma, which ISO 8601 also writes after a full stop.
    #[test]
    fn a_fraction_of_a_second_orders_and_truncates() {
        let nanos = |raw: &str| VcardTimestamp::from(raw).to_unix_nanos();
        let whole = seconds("19951031T222710Z").unwrap();

        assert_eq!(seconds("1995-10-31T22:27:10,5Z"), Some(whole));
        assert_eq!(
            nanos("1995-10-31T22:27:10,5Z"),
            nanos("19951031T222710.500Z")
        );
        assert!(nanos("19951031T222710.25Z") < nanos("19951031T222710,5Z"));
        assert_eq!(
            nanos("19951031T232710.1234567891+01:00"),
            Some(i128::from(whole) * 1_000_000_000 + 123_456_789),
        );
        assert_eq!(seconds("19951031T2227,5Z"), None);
        assert_eq!(seconds("19951031T222710,Z"), None);
        assert_eq!(seconds("19951031T222710,5aZ"), None);
    }

    #[test]
    fn rejects_non_timestamps() {
        assert_eq!(seconds(""), None);
        assert_eq!(seconds("not-a-date"), None);
        assert_eq!(seconds("2026-13-11T00:00:00Z"), None);
        assert_eq!(seconds("2026-07-11T25:00:00Z"), None);
        // NOTE: an offset with no sign, with non-digits, of an unusable width,
        // or out of range is not an offset.
        assert_eq!(seconds("2026-07-11T19:25:5902:00"), None);
        assert_eq!(seconds("2026-07-11T19:25:59+ab:00"), None);
        assert_eq!(seconds("2026-07-11T19:25:59+020"), None);
        assert_eq!(seconds("2026-07-11T19:25:59+24:00"), None);
        assert_eq!(seconds("2026-07-11T19:25:59+02:60"), None);
    }

    #[test]
    fn full_date_reads_both_complete_forms_and_nothing_else() {
        use crate::value::datetime::VcardDateAndOrTime;

        let full = |raw: &str| VcardDateAndOrTime::from(raw).full_date();

        assert_eq!(full("1985-04-12").as_deref(), Some("1985-04-12"));
        assert_eq!(full(" 19850412 ").as_deref(), Some("1985-04-12"));
        assert_eq!(full("--0412"), None);
        assert_eq!(full("1985-04"), None);
        assert_eq!(full("19850412T102200"), None);
    }
}
