//! # Export
//!
//! The model-to-jCard half: a decoded property, parameter and value written
//! as their RFC 7095 slots.
//!
//! Out, the codec follows the RFC: names are lowercased, a group prefix moves
//! to the `group` parameter (3.3.1.2), the `VALUE` parameter moves to the type
//! slot (3.3.1.1), and date, time and UTC-offset values are re-spelled in
//! extended ISO 8601 (3.5).
//!
//! A repeated parameter name merges into one array value, since a JSON object
//! holds each key once.

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde_json::{Map, Value};

use crate::{
    jcard::datetime::{basic_to_extended, offset_to_extended},
    param::VcardParam,
    prop::VcardProp,
    value::{VcardValue, VcardValueUnknown, binary::VcardBinary},
};

impl VcardProp<'_> {
    /// Write the property as a jCard `[name, {params}, type, value...]` entry.
    ///
    /// Also the encoder behind the RFC 9555 vCardProps escape hatch, which
    /// preserves whole properties in jCard syntax.
    pub(crate) fn to_jcard(&self) -> Value {
        let full = &*self.name;
        let (group, name) = match full.split_once('.') {
            Some((group, name)) => (Some(group), name),
            None => (None, full),
        };

        let mut params = Map::new();
        if let Some(group) = group {
            params.insert("group".into(), Value::String(group.to_ascii_lowercase()));
        }

        let mut declared = None;
        for param in &self.params {
            if let VcardParam::Value(kind) = param {
                declared = Some(kind.to_ascii_lowercase());
                continue;
            }
            let (key, value) = param.to_jcard();
            merge_param(&mut params, key, value);
        }

        let (default_slot, values) = self.value.to_jcard();
        let slot = declared.unwrap_or_else(|| default_slot.to_string());

        let mut entry = vec![
            Value::String(name.to_ascii_lowercase()),
            Value::Object(params),
            Value::String(slot),
        ];
        entry.extend(values);
        Value::Array(entry)
    }
}

impl VcardParam<'_> {
    /// The jCard spelling of the parameter: its lowercased wire name and its
    /// value, a string or an array for a multi-valued one.
    ///
    /// Also the encoder behind the RFC 9555 vCardParams escape hatch.
    pub(crate) fn to_jcard(&self) -> (String, Value) {
        match self {
            VcardParam::Unknown { name, values } => {
                (name.to_ascii_lowercase(), text_or_list(values))
            }
            VcardParam::Pid(values) | VcardParam::SortAs(values) | VcardParam::Type(values) => {
                (self.jcard_key(), text_or_list(values))
            }
            VcardParam::AltId(value)
            | VcardParam::Author(value)
            | VcardParam::AuthorName(value)
            | VcardParam::CalScale(value)
            | VcardParam::Charset(value)
            | VcardParam::Created(value)
            | VcardParam::Derived(value)
            | VcardParam::Encoding(value)
            | VcardParam::Geo(value)
            | VcardParam::Jsptr(value)
            | VcardParam::Label(value)
            | VcardParam::Language(value)
            | VcardParam::MediaType(value)
            | VcardParam::Phonetic(value)
            | VcardParam::Pref(value)
            | VcardParam::PropId(value)
            | VcardParam::Script(value)
            | VcardParam::ServiceType(value)
            | VcardParam::Tz(value)
            | VcardParam::Username(value)
            | VcardParam::Value(value) => (self.jcard_key(), Value::String(value.to_string())),
        }
    }

    /// The jCard object key of a known parameter: its lowercased wire name.
    fn jcard_key(&self) -> String {
        self.kind()
            .map(|kind| kind.to_ascii_lowercase())
            .unwrap_or_default()
    }
}

/// Insert a parameter into the jCard params object, merging a repeated name
/// into one array value.
pub(crate) fn merge_param(params: &mut Map<String, Value>, key: String, value: Value) {
    match params.get_mut(&key) {
        None => {
            params.insert(key, value);
        }
        Some(Value::Array(existing)) => match value {
            Value::Array(values) => existing.extend(values),
            value => existing.push(value),
        },
        Some(existing) => {
            let mut values = vec![existing.take()];
            match value {
                Value::Array(more) => values.extend(more),
                value => values.push(value),
            }
            *existing = Value::Array(values);
        }
    }
}

impl VcardValue<'_> {
    /// Write the value as its default jCard type slot and value slots.
    fn to_jcard(&self) -> (&'static str, Vec<Value>) {
        match self {
            VcardValue::Text(text) => ("text", vec![Value::String(text.0.to_string())]),
            VcardValue::TextList(list) if list.0.is_empty() => {
                ("text", vec![Value::String(String::new())])
            }
            VcardValue::TextList(list) => ("text", strings(&list.0)),
            VcardValue::Uri(uri) => ("uri", vec![Value::String(uri.0.to_string())]),
            VcardValue::DateAndOrTime(date) => (
                "date-and-or-time",
                vec![Value::String(basic_to_extended(&date.0))],
            ),
            VcardValue::Timestamp(timestamp) => (
                "timestamp",
                vec![Value::String(basic_to_extended(&timestamp.0))],
            ),
            VcardValue::LanguageTag(tag) => {
                ("language-tag", vec![Value::String(tag.0.to_string())])
            }
            VcardValue::UtcOffset(offset) => (
                "utc-offset",
                vec![Value::String(offset_to_extended(&offset.0))],
            ),
            VcardValue::N(n) => (
                "text",
                vec![Value::Array(vec![
                    text_or_list(&n.family),
                    text_or_list(&n.given),
                    text_or_list(&n.additional),
                    text_or_list(&n.prefixes),
                    text_or_list(&n.suffixes),
                ])],
            ),
            VcardValue::Adr(adr) => {
                let mut components = vec![
                    text_or_list(&adr.po_box),
                    text_or_list(&adr.extended),
                    text_or_list(&adr.street),
                    text_or_list(&adr.locality),
                    text_or_list(&adr.region),
                    text_or_list(&adr.postal_code),
                    text_or_list(&adr.country),
                ];
                if adr.has_extended_components() {
                    components.extend([
                        text_or_list(&adr.room),
                        text_or_list(&adr.apartment),
                        text_or_list(&adr.floor),
                        text_or_list(&adr.street_number),
                        text_or_list(&adr.street_name),
                        text_or_list(&adr.building),
                        text_or_list(&adr.block),
                        text_or_list(&adr.subdistrict),
                        text_or_list(&adr.district),
                        text_or_list(&adr.landmark),
                        text_or_list(&adr.direction),
                    ]);
                }
                ("text", vec![Value::Array(components)])
            }
            VcardValue::Gender(gender) if gender.identity.is_empty() => {
                ("text", vec![Value::String(gender.sex.to_string())])
            }
            VcardValue::Gender(gender) => (
                "text",
                vec![Value::Array(vec![
                    Value::String(gender.sex.to_string()),
                    Value::String(gender.identity.to_string()),
                ])],
            ),
            VcardValue::Org(org) => match org.0.as_slice() {
                [] => ("text", vec![Value::String(String::new())]),
                [unit] => ("text", vec![Value::String(unit.to_string())]),
                units => ("text", vec![Value::Array(strings(units))]),
            },
            VcardValue::ClientPidMap(map) => (
                "text",
                vec![Value::Array(vec![
                    Value::String(map.id.to_string()),
                    Value::String(map.uri.to_string()),
                ])],
            ),
            VcardValue::Geo(geo) => (
                "text",
                vec![Value::Array(vec![
                    Value::String(geo.latitude.to_string()),
                    Value::String(geo.longitude.to_string()),
                ])],
            ),
            VcardValue::Binary(VcardBinary::Uri(uri)) => {
                ("uri", vec![Value::String(uri.to_string())])
            }
            VcardValue::Binary(VcardBinary::Base64(data)) => {
                ("unknown", vec![Value::String(data.to_string())])
            }
            VcardValue::Unknown(unknown) => ("unknown", vec![unknown.to_jcard()]),
        }
    }
}

impl VcardValueUnknown<'_> {
    /// Write the value structurally: one component group collapses to a string or
    /// an array, several stay an array of arrays so the two nesting levels stay
    /// apart (see the module docs).
    fn to_jcard(&self) -> Value {
        match self.components.as_slice() {
            [] => Value::String(String::new()),
            [group] => text_or_list(group),
            groups => Value::Array(
                groups
                    .iter()
                    .map(|group| Value::Array(strings(group)))
                    .collect(),
            ),
        }
    }
}

/// Write a text list as jCard does everywhere: nothing is an empty string,
/// one value a string, several an array.
fn text_or_list(values: &[Cow<'_, str>]) -> Value {
    match values {
        [] => Value::String(String::new()),
        [value] => Value::String(value.to_string()),
        values => Value::Array(strings(values)),
    }
}

/// Each value as a JSON string.
fn strings(values: &[Cow<'_, str>]) -> Vec<Value> {
    values
        .iter()
        .map(|value| Value::String(value.to_string()))
        .collect()
}
