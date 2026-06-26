//! The value types of RFC 6350 section 4, plus the multi-type value
//! choices a few properties allow.

use alloc::borrow::Cow;

/// A date and/or time, any component of which may be absent (RFC 6350
/// date, time, date-time, date-and-or-time and timestamp values).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VcardDateAndOrTime {
    /// The year, if present.
    pub year: Option<i32>,
    /// The month from 1 to 12, if present.
    pub month: Option<u8>,
    /// The day of month, if present.
    pub day: Option<u8>,
    /// The hour, if present.
    pub hour: Option<u8>,
    /// The minute, if present.
    pub minute: Option<u8>,
    /// The second, if present.
    pub second: Option<u8>,
    /// The UTC offset, if the value is zoned.
    pub utc_offset: Option<VcardUtcOffset>,
}

/// A UTC offset (RFC 6350 UTC-OFFSET value) as signed hours and minutes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VcardUtcOffset {
    /// Hours, signed (the offset sign lives here).
    pub hours: i8,
    /// Minutes, from 0 to 59.
    pub minutes: u8,
}

/// A value that is either a URI or free text (TEL, UID, KEY, RELATED).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VcardUriOrText<'a> {
    /// A URI value.
    Uri(Cow<'a, str>),
    /// A free-text value.
    Text(Cow<'a, str>),
}

impl Default for VcardUriOrText<'_> {
    fn default() -> Self {
        Self::Text(Cow::Borrowed(""))
    }
}

/// A value that is either a date-and-or-time or free text (BDAY, ANNIVERSARY).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VcardDateOrText<'a> {
    /// A date and/or time value.
    DateTime(VcardDateAndOrTime),
    /// A free-text value.
    Text(Cow<'a, str>),
}

impl Default for VcardDateOrText<'_> {
    fn default() -> Self {
        Self::Text(Cow::Borrowed(""))
    }
}

/// A time-zone value: free text, a URI, or a UTC offset (TZ).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VcardTzValue<'a> {
    /// A free-text zone name.
    Text(Cow<'a, str>),
    /// A URI identifying the zone.
    Uri(Cow<'a, str>),
    /// A fixed UTC offset.
    UtcOffset(VcardUtcOffset),
}

impl Default for VcardTzValue<'_> {
    fn default() -> Self {
        Self::Text(Cow::Borrowed(""))
    }
}
