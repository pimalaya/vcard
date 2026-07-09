//! # jCard
//!
//! The RFC 7095 jCard codec: the decoded card as JSON, and back.
//!
//! jCard is the JSON spelling of the vCard model: a card is a
//! `["vcard", [...]]` array of `[name, {params}, type, value...]` property
//! entries (RFC 7095 3). [`Vcard::to_jcard`] writes the decoded model as a
//! [`serde_json::Value`]; [`Vcard::from_jcard`] reads one back, borrowing the
//! JSON tree's strings. Import resolves each property's value kind through
//! the same spec vtable as the wire decoder, so a jCard and the vCard it was
//! written from decode to the same model.
//!
//! The codec keeps the crate's Postel stance. On the way out it follows RFC
//! 7095: names are lowercased, a group prefix moves to the `group` parameter
//! (RFC 7095 3.3.1.2), the `VALUE` parameter moves to the type slot (RFC
//! 7095 3.3.1.1), and date, time and UTC-offset values are re-spelled in the
//! extended ISO 8601 format (RFC 7095 3.5). On the way in anything is
//! accepted: unknown names, parameters and type slots survive verbatim,
//! non-string scalars are coerced to text, and a type slot that names no
//! known kind falls back to the property's version default.
//!
//! Two deliberate, lossless departures from the letter of the RFC. A card of
//! any version is written, carrying its own version in the `version` entry,
//! where RFC 7095 defines jCard for vCard 4.0 only. And an undecoded value
//! is written structurally, mirroring its semicolon and comma components as
//! a string, an array, or an array of arrays, rather than as the RFC's one
//! re-escaped string: the decoded model holds unescaped components, and
//! re-escaping belongs to the wire codec.
//!
//! Round-tripping normalizes rather than preserves: parameter order is lost
//! to the JSON object, a declared `VALUE` equal to the property's default is
//! dropped, and names come back in their canonical spelling. Byte fidelity
//! is the syntax tree's job; jCard is a projection of the decoded model.

use core::{error, fmt};

use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde_json::{Map, Value, json};

use crate::{
    param::{VcardParam, VcardParamKind},
    prop::{VcardProp, VcardPropName},
    tree::prop::prop_spec,
    value::{
        VcardUnknownValue, VcardValue, VcardValueKind,
        adr::VcardAdr,
        binary::VcardBinary,
        client_pid_map::VcardClientPidMap,
        datetime::{VcardDateAndOrTime, VcardTimestamp},
        gender::VcardGender,
        geo::VcardGeo,
        language::VcardLanguageTag,
        n::VcardN,
        org::VcardOrg,
        text::{VcardText, VcardTextList},
        uri::VcardUri,
        utc_offset::VcardUtcOffset,
    },
    vcard::Vcard,
    version::VcardVersion,
};

/// Parse jCard error.
#[derive(Debug)]
pub enum ParseJcardError {
    /// The root is not a two-element `["vcard", [...]]` array.
    InvalidCard,
    /// A property entry is not a `[name, {params}, type, ...]` array.
    InvalidProp(String),
}

impl fmt::Display for ParseJcardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCard => {
                write!(f, "Cannot parse jCard: the root is not [\"vcard\", [...]]")
            }
            Self::InvalidProp(prop) => write!(f, "Cannot parse jCard property `{prop}`"),
        }
    }
}

impl error::Error for ParseJcardError {}

impl Vcard<'_> {
    /// Write the card as its RFC 7095 jCard [`Value`].
    pub fn to_jcard(&self) -> Value {
        let mut entries = Vec::with_capacity(self.properties.len() + 1);
        entries.push(json!(["version", {}, "text", &*self.version]));
        entries.extend(self.properties.iter().map(prop_to_jcard));
        json!(["vcard", entries])
    }

    /// Read a card from its RFC 7095 jCard [`Value`], borrowing its strings.
    ///
    /// Liberal: only a structurally broken root or property entry errors;
    /// unknown names, parameters and type slots are kept.
    pub fn from_jcard(jcard: &Value) -> Result<Vcard<'_>, ParseJcardError> {
        let root = jcard
            .as_array()
            .filter(|root| root.len() == 2)
            .ok_or(ParseJcardError::InvalidCard)?;
        let tag = root[0].as_str().ok_or(ParseJcardError::InvalidCard)?;
        if !tag.eq_ignore_ascii_case("vcard") {
            return Err(ParseJcardError::InvalidCard);
        }
        let entries = root[1].as_array().ok_or(ParseJcardError::InvalidCard)?;

        // NOTE: the version is the card's indicator, not a free property,
        // mirroring the wire decoder; an unreadable one normalises to 4.0.
        let version = entries
            .iter()
            .filter_map(Value::as_array)
            .find(|entry| entry_is_version(entry))
            .and_then(|entry| entry.get(3))
            .and_then(Value::as_str)
            .and_then(|version| version.parse().ok())
            .unwrap_or(VcardVersion::V4_0);

        let mut properties = Vec::new();

        for entry in entries {
            let invalid = || ParseJcardError::InvalidProp(entry.to_string());
            let entry = entry
                .as_array()
                .filter(|entry| entry.len() >= 3)
                .ok_or_else(invalid)?;

            if entry_is_version(entry) {
                continue;
            }

            let name = entry[0].as_str().ok_or_else(invalid)?;
            let params = entry[1].as_object().ok_or_else(invalid)?;
            let slot = entry[2].as_str().ok_or_else(invalid)?;

            properties.push(prop_from_jcard(name, params, slot, &entry[3..], version));
        }

        Ok(Vcard {
            version,
            properties,
        })
    }
}

/// Write one decoded property as a jCard `[name, {params}, type, value...]`
/// entry. Also the encoder behind the RFC 9555 vCardProps escape hatch, which
/// preserves whole properties in jCard syntax.
pub(crate) fn prop_to_jcard(prop: &VcardProp<'_>) -> Value {
    let full = &*prop.name;
    let (group, name) = match full.split_once('.') {
        Some((group, name)) => (Some(group), name),
        None => (None, full),
    };

    let mut params = Map::new();
    if let Some(group) = group {
        params.insert("group".into(), Value::String(group.to_ascii_lowercase()));
    }

    let mut declared = None;
    for param in &prop.params {
        if let VcardParam::Value(kind) = param {
            declared = Some(kind.to_ascii_lowercase());
            continue;
        }
        let (key, value) = param_to_jcard(param);
        merge_param(&mut params, key, value);
    }

    let (default_slot, values) = value_to_jcard(&prop.value);
    let slot = declared.unwrap_or_else(|| default_slot.to_string());

    let mut entry = vec![
        Value::String(name.to_ascii_lowercase()),
        Value::Object(params),
        Value::String(slot),
    ];
    entry.extend(values);
    Value::Array(entry)
}

/// The jCard spelling of one parameter: its lowercased wire name and its
/// value (a string, or an array for a multi-valued parameter). Also the
/// encoder behind the RFC 9555 vCardParams escape hatch.
pub(crate) fn param_to_jcard(param: &VcardParam<'_>) -> (String, Value) {
    match param {
        VcardParam::Unknown { name, values } => (name.to_ascii_lowercase(), text_or_list(values)),
        VcardParam::Pid(values) | VcardParam::SortAs(values) | VcardParam::Type(values) => {
            (known_param_key(param), text_or_list(values))
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
        | VcardParam::Value(value) => (known_param_key(param), Value::String(value.to_string())),
    }
}

/// The jCard object key of a known parameter: its lowercased wire name.
fn known_param_key(param: &VcardParam<'_>) -> String {
    param
        .kind()
        .map(|kind| kind.to_ascii_lowercase())
        .unwrap_or_default()
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

/// Write a decoded value as its default jCard type slot and value slots.
fn value_to_jcard(value: &VcardValue<'_>) -> (&'static str, Vec<Value>) {
    match value {
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
        VcardValue::LanguageTag(tag) => ("language-tag", vec![Value::String(tag.0.to_string())]),
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
        VcardValue::Binary(VcardBinary::Uri(uri)) => ("uri", vec![Value::String(uri.to_string())]),
        VcardValue::Binary(VcardBinary::Base64(data)) => {
            ("unknown", vec![Value::String(data.to_string())])
        }
        VcardValue::Unknown(unknown) => ("unknown", vec![unknown_to_jcard(unknown)]),
    }
}

/// Write an undecoded value structurally: one component group collapses to a
/// string or an array, several stay an array of arrays so the two nesting
/// levels stay apart (see the module docs).
fn unknown_to_jcard(unknown: &VcardUnknownValue<'_>) -> Value {
    match unknown.components.as_slice() {
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

/// Whether a jCard property entry is the `version` pseudo-property.
fn entry_is_version(entry: &[Value]) -> bool {
    matches!(
        entry.first().and_then(Value::as_str),
        Some(name) if name.eq_ignore_ascii_case("version"),
    )
}

/// Read one jCard entry into a decoded property, resolving its value kind
/// through the property spec like the wire decoder does. Also the decoder
/// behind the RFC 9555 vCardProps escape hatch.
pub(crate) fn prop_from_jcard<'a>(
    name: &'a str,
    params: &'a Map<String, Value>,
    slot: &'a str,
    values: &'a [Value],
    version: VcardVersion,
) -> VcardProp<'a> {
    // NOTE: a grouped name is rebuilt as its wire form (group prefix kept on
    // the name), which is how the wire decoder models groups too.
    let group = params.get("group").and_then(Value::as_str);
    let name = match group {
        Some(group) => format!("{group}.{}", name.to_ascii_uppercase()),
        None => name.to_ascii_uppercase(),
    };
    let name = VcardPropName::from(Cow::Owned(name));

    let mut prop_params: Vec<VcardParam<'_>> = params
        .iter()
        .filter(|(key, _)| !key.eq_ignore_ascii_case("group"))
        .map(|(key, value)| param_from_jcard(key, value))
        .collect();

    // The type slot goes back to a VALUE parameter only where the wire form
    // would need one: when it differs from the kind the spec would pick with
    // no declaration (for an unknown property, from its "unknown" default).
    let kind = match &name {
        VcardPropName::Kind(prop) => {
            let spec = prop_spec(*prop);
            let default = (spec.value)(version, None);

            // NOTE: jCard types every text-shaped value "text" (a structured
            // N is `["n", {}, "text", [...]]`), so against a text-shaped
            // default the slot declares nothing and must not flatten the
            // structured kind to a single text.
            let declared = slot
                .parse::<VcardValueKind>()
                .ok()
                .filter(|declared| !(*declared == VcardValueKind::Text && is_text_shaped(default)));

            if declared.is_some() && declared != Some(default) {
                prop_params.push(VcardParam::Value(Cow::Borrowed(slot)));
            }
            Some((spec.value)(version, declared))
        }
        VcardPropName::Unknown(_) => {
            if !slot.eq_ignore_ascii_case("unknown") {
                prop_params.push(VcardParam::Value(Cow::Borrowed(slot)));
            }
            None
        }
    };

    let value = match kind {
        Some(kind) => value_from_jcard(kind, values),
        None => VcardValue::Unknown(unknown_from_jcard(values)),
    };

    VcardProp {
        name,
        params: prop_params,
        value,
    }
}

/// Read one jCard params-object member into a decoded parameter. Also the
/// decoder behind the RFC 9555 vCardParams escape hatch.
pub(crate) fn param_from_jcard<'a>(key: &'a str, value: &'a Value) -> VcardParam<'a> {
    let Ok(kind) = key.parse::<VcardParamKind>() else {
        return VcardParam::Unknown {
            name: Cow::Owned(key.to_ascii_uppercase()),
            values: scalars(value),
        };
    };

    match kind {
        VcardParamKind::AltId => VcardParam::AltId(scalar(value)),
        VcardParamKind::CalScale => VcardParam::CalScale(scalar(value)),
        VcardParamKind::Charset => VcardParam::Charset(scalar(value)),
        VcardParamKind::Encoding => VcardParam::Encoding(scalar(value)),
        VcardParamKind::Geo => VcardParam::Geo(scalar(value)),
        VcardParamKind::Label => VcardParam::Label(scalar(value)),
        VcardParamKind::Language => VcardParam::Language(scalar(value)),
        VcardParamKind::MediaType => VcardParam::MediaType(scalar(value)),
        VcardParamKind::Pid => VcardParam::Pid(scalars(value)),
        VcardParamKind::Pref => VcardParam::Pref(scalar(value)),
        VcardParamKind::SortAs => VcardParam::SortAs(scalars(value)),
        VcardParamKind::Type => VcardParam::Type(scalars(value)),
        VcardParamKind::Tz => VcardParam::Tz(scalar(value)),
        VcardParamKind::Value => VcardParam::Value(scalar(value)),
        VcardParamKind::Author => VcardParam::Author(scalar(value)),
        VcardParamKind::AuthorName => VcardParam::AuthorName(scalar(value)),
        VcardParamKind::Created => VcardParam::Created(scalar(value)),
        VcardParamKind::Derived => VcardParam::Derived(scalar(value)),
        VcardParamKind::Jsptr => VcardParam::Jsptr(scalar(value)),
        VcardParamKind::Phonetic => VcardParam::Phonetic(scalar(value)),
        VcardParamKind::PropId => VcardParam::PropId(scalar(value)),
        VcardParamKind::Script => VcardParam::Script(scalar(value)),
        VcardParamKind::ServiceType => VcardParam::ServiceType(scalar(value)),
        VcardParamKind::Username => VcardParam::Username(scalar(value)),
    }
}

/// Read jCard value slots as the given decoded value kind.
fn value_from_jcard<'a>(kind: VcardValueKind, values: &'a [Value]) -> VcardValue<'a> {
    match kind {
        VcardValueKind::Text => VcardValue::Text(VcardText(first_scalar(values))),
        VcardValueKind::TextList => {
            VcardValue::TextList(VcardTextList(values.iter().flat_map(scalars).collect()))
        }
        VcardValueKind::Uri => VcardValue::Uri(VcardUri(first_scalar(values))),
        VcardValueKind::DateAndOrTime => {
            VcardValue::DateAndOrTime(VcardDateAndOrTime(extended_to_basic(first_scalar(values))))
        }
        VcardValueKind::Timestamp => {
            VcardValue::Timestamp(VcardTimestamp(extended_to_basic(first_scalar(values))))
        }
        VcardValueKind::LanguageTag => {
            VcardValue::LanguageTag(VcardLanguageTag(first_scalar(values)))
        }
        VcardValueKind::UtcOffset => {
            VcardValue::UtcOffset(VcardUtcOffset(offset_to_basic(first_scalar(values))))
        }
        VcardValueKind::N => {
            let mut components = structured(values.first()).into_iter();
            VcardValue::N(VcardN {
                family: components.next().unwrap_or_default(),
                given: components.next().unwrap_or_default(),
                additional: components.next().unwrap_or_default(),
                prefixes: components.next().unwrap_or_default(),
                suffixes: components.next().unwrap_or_default(),
            })
        }
        VcardValueKind::Adr => {
            let mut components = structured(values.first()).into_iter();
            let mut next = || components.next().unwrap_or_default();
            VcardValue::Adr(VcardAdr {
                po_box: next(),
                extended: next(),
                street: next(),
                locality: next(),
                region: next(),
                postal_code: next(),
                country: next(),
                room: next(),
                apartment: next(),
                floor: next(),
                street_number: next(),
                street_name: next(),
                building: next(),
                block: next(),
                subdistrict: next(),
                district: next(),
                landmark: next(),
                direction: next(),
            })
        }
        VcardValueKind::Gender => {
            let mut components = structured(values.first()).into_iter();
            VcardValue::Gender(VcardGender {
                sex: first_of(components.next()),
                identity: first_of(components.next()),
            })
        }
        VcardValueKind::Org => VcardValue::Org(VcardOrg(
            structured(values.first())
                .into_iter()
                .map(|unit| first_of(Some(unit)))
                .collect(),
        )),
        VcardValueKind::ClientPidMap => {
            let mut components = structured(values.first()).into_iter();
            VcardValue::ClientPidMap(VcardClientPidMap {
                id: first_of(components.next()),
                uri: first_of(components.next()),
            })
        }
        VcardValueKind::Geo => {
            let mut components = structured(values.first()).into_iter();
            VcardValue::Geo(VcardGeo {
                latitude: first_of(components.next()),
                longitude: first_of(components.next()),
            })
        }
        // NOTE: a binary kind resolves only with no declared slot (a URI
        // reference declares VALUE=uri and resolves to Uri), so the payload
        // can only be the 2.1 / 3.0 inline base64 reading.
        VcardValueKind::Binary => VcardValue::Binary(VcardBinary::Base64(first_scalar(values))),
    }
}

/// Read an undecoded value's slots back into semicolon and comma components,
/// the inverse of [`unknown_to_jcard`].
fn unknown_from_jcard(values: &[Value]) -> VcardUnknownValue<'_> {
    let components = match values {
        [] => vec![vec![Cow::Borrowed("")]],
        [Value::Array(groups)] if groups.iter().any(Value::is_array) => {
            groups.iter().map(scalars).collect()
        }
        [Value::Array(group)] => vec![group.iter().map(scalar).collect()],
        [value] => vec![vec![scalar(value)]],
        values => values.iter().map(scalars).collect(),
    };

    VcardUnknownValue { components }
}

/// Whether a kind is one jCard spells with the plain "text" type slot: the
/// single and list text kinds plus the structured text shapes.
fn is_text_shaped(kind: VcardValueKind) -> bool {
    matches!(
        kind,
        VcardValueKind::Text
            | VcardValueKind::TextList
            | VcardValueKind::N
            | VcardValueKind::Adr
            | VcardValueKind::Gender
            | VcardValueKind::Org
            | VcardValueKind::ClientPidMap
            | VcardValueKind::Geo,
    )
}

/// A structured value's component groups: each entry of the array is one
/// group, itself a string or an array; a bare scalar is one one-value group.
fn structured(value: Option<&Value>) -> Vec<Vec<Cow<'_, str>>> {
    match value {
        None => Vec::new(),
        Some(Value::Array(components)) => components.iter().map(scalars).collect(),
        Some(value) => vec![vec![scalar(value)]],
    }
}

/// The first value of a component group, or empty.
fn first_of(group: Option<Vec<Cow<'_, str>>>) -> Cow<'_, str> {
    group
        .and_then(|group| group.into_iter().next())
        .unwrap_or_default()
}

/// The first value slot as text, or empty.
fn first_scalar(values: &[Value]) -> Cow<'_, str> {
    values.first().map(scalar).unwrap_or_default()
}

/// A JSON value as one text value: a string borrows, a number or boolean is
/// coerced, an array yields its first value, anything else is empty.
fn scalar(value: &Value) -> Cow<'_, str> {
    match value {
        Value::String(value) => Cow::Borrowed(value.as_str()),
        Value::Number(_) | Value::Bool(_) => Cow::Owned(value.to_string()),
        Value::Array(values) => values.first().map(scalar).unwrap_or_default(),
        Value::Null | Value::Object(_) => Cow::Borrowed(""),
    }
}

/// A JSON value as a text list: an array yields one value per entry, anything
/// else one value.
fn scalars(value: &Value) -> Vec<Cow<'_, str>> {
    match value {
        Value::Array(values) => values.iter().map(scalar).collect(),
        value => vec![scalar(value)],
    }
}

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
fn offset_to_extended(offset: &str) -> String {
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
fn extended_str_to_basic(raw: &str) -> Option<String> {
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
fn offset_to_basic(raw: Cow<'_, str>) -> Cow<'_, str> {
    if raw.contains(':') {
        Cow::Owned(raw.chars().filter(|c| *c != ':').collect())
    } else {
        raw
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, vec};

    use serde_json::json;

    use crate::{
        jcard::{ParseJcardError, basic_to_extended, extended_str_to_basic},
        param::VcardParam,
        prop::VcardPropName,
        tree::cst::VcardCst,
        value::{VcardUnknownValue, VcardValue, text::VcardText},
        vcard::Vcard,
        version::VcardVersion,
    };

    #[test]
    fn exports_a_simple_card_with_its_version_entry() {
        let cst =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nEND:VCARD\r\n").unwrap();

        assert_eq!(
            cst.decode().to_jcard(),
            json!([
                "vcard",
                [
                    ["version", {}, "text", "4.0"],
                    ["fn", {}, "text", "John Doe"],
                ],
            ]),
        );
    }

    #[test]
    fn moves_the_value_param_into_the_type_slot_and_back() {
        let cst = VcardCst::parse(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nBDAY;VALUE=text:circa 1800\r\nEND:VCARD\r\n",
        )
        .unwrap();
        let jcard = cst.decode().to_jcard();

        assert_eq!(
            jcard,
            json!([
                "vcard",
                [
                    ["version", {}, "text", "4.0"],
                    ["bday", {}, "text", "circa 1800"],
                ],
            ]),
        );

        // NOTE: text is not BDAY's default kind, so the declared VALUE comes
        // back as a parameter and the value stays text.
        let card = Vcard::from_jcard(&jcard).unwrap();
        let prop = &card.properties[0];
        assert_eq!(prop.params, vec![VcardParam::Value(Cow::Borrowed("text"))]);
        assert_eq!(
            prop.value,
            VcardValue::Text(VcardText(Cow::Borrowed("circa 1800"))),
        );
    }

    #[test]
    fn moves_a_group_prefix_into_the_group_parameter_and_back() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "ITEM1.X-ABLABEL:Nickname\r\n",
            "END:VCARD\r\n",
        );
        let jcard = VcardCst::parse(input).unwrap().decode().to_jcard();

        assert_eq!(
            jcard,
            json!([
                "vcard",
                [
                    ["version", {}, "text", "4.0"],
                    ["x-ablabel", { "group": "item1" }, "unknown", "Nickname"],
                ],
            ]),
        );

        let card = Vcard::from_jcard(&jcard).unwrap();
        let prop = &card.properties[0];
        assert_eq!(
            prop.name,
            VcardPropName::Unknown(Cow::Borrowed("item1.X-ABLABEL")),
        );
        assert_eq!(
            prop.value,
            VcardValue::Unknown(VcardUnknownValue {
                components: vec![vec![Cow::Borrowed("Nickname")]],
            }),
        );
    }

    #[test]
    fn converts_dates_between_basic_and_extended() {
        assert_eq!(basic_to_extended("19850412"), "1985-04-12");
        assert_eq!(basic_to_extended("1985-04"), "1985-04");
        assert_eq!(basic_to_extended("--0412"), "--04-12");
        assert_eq!(basic_to_extended("---12"), "---12");
        assert_eq!(basic_to_extended("T102200"), "T10:22:00");
        assert_eq!(basic_to_extended("T-2200"), "T-22:00");
        assert_eq!(
            basic_to_extended("19961022T140000-0500"),
            "1996-10-22T14:00:00-05:00",
        );
        assert_eq!(
            basic_to_extended("19961022T140000Z"),
            "1996-10-22T14:00:00Z"
        );
        assert_eq!(basic_to_extended("circa 1800"), "circa 1800");

        assert_eq!(
            extended_str_to_basic("1996-10-22T14:00:00-05:00").as_deref(),
            Some("19961022T140000-0500"),
        );
        assert_eq!(extended_str_to_basic("--04-12").as_deref(), Some("--0412"));
        assert_eq!(extended_str_to_basic("1985-04"), None);
        assert_eq!(extended_str_to_basic("circa 1800"), None);
    }

    #[test]
    fn merges_a_repeated_parameter_and_keeps_a_list_one() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "TEL;TYPE=work,home;TYPE=voice:tel:123\r\n",
            "END:VCARD\r\n",
        );
        let jcard = VcardCst::parse(input).unwrap().decode().to_jcard();

        assert_eq!(
            jcard,
            json!([
                "vcard",
                [
                    ["version", {}, "text", "4.0"],
                    ["tel", { "type": ["work", "home", "voice"] }, "text", "tel:123"],
                ],
            ]),
        );

        let card = Vcard::from_jcard(&jcard).unwrap();
        assert_eq!(
            card.properties[0].params,
            vec![VcardParam::Type(vec![
                Cow::Borrowed("work"),
                Cow::Borrowed("home"),
                Cow::Borrowed("voice"),
            ])],
        );
    }

    #[test]
    fn round_trips_a_card_through_jcard() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John Doe\r\n",
            "N:Doe;John;;Dr.;\r\n",
            "EMAIL;TYPE=work:john@example.com\r\n",
            "CATEGORIES:friend,colleague\r\n",
            "BDAY:19850412\r\n",
            "LANG;PREF=1:fr\r\n",
            "REV:19961022T140000Z\r\n",
            "TZ;VALUE=utc-offset:-0500\r\n",
            "UID:urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1\r\n",
            "X-SKI:snowboard\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode();
        let jcard = card.to_jcard();
        let reimported = Vcard::from_jcard(&jcard).unwrap();

        assert_eq!(reimported.version, VcardVersion::V4_0);
        assert_eq!(reimported, card);
    }

    #[test]
    fn imports_a_foreign_jcard_liberally() {
        // NOTE: rfc 7095 2.1-style example: a number where text is expected,
        // an uppercase tag, no version entry.
        let jcard = json!([
            "VCARD",
            [
                ["fn", {}, "text", "SimonPerreault"],
                ["clientpidmap", {}, "text", [1, "urn:uuid:x"]],
                [
                    "n",
                    {},
                    "text",
                    ["Perreault", "Simon", "", "", ["ing.jr", "M.Sc."]]
                ],
            ],
        ]);
        let card = Vcard::from_jcard(&jcard).unwrap();

        assert_eq!(card.version, VcardVersion::V4_0);
        assert_eq!(card.properties.len(), 3);
        assert_eq!(
            card.properties[1].value,
            VcardValue::ClientPidMap(crate::value::client_pid_map::VcardClientPidMap {
                id: Cow::Borrowed("1"),
                uri: Cow::Borrowed("urn:uuid:x"),
            }),
        );
        assert_eq!(
            card.properties[2].value,
            VcardValue::N(crate::value::n::VcardN {
                family: vec![Cow::Borrowed("Perreault")],
                given: vec![Cow::Borrowed("Simon")],
                additional: vec![Cow::Borrowed("")],
                prefixes: vec![Cow::Borrowed("")],
                suffixes: vec![Cow::Borrowed("ing.jr"), Cow::Borrowed("M.Sc.")],
            }),
        );
    }

    #[test]
    fn errors_on_a_broken_root_or_property_entry() {
        for jcard in [json!({}), json!(["vcalendar", []]), json!(["vcard", [], 3])] {
            assert!(matches!(
                Vcard::from_jcard(&jcard),
                Err(ParseJcardError::InvalidCard),
            ));
        }

        let jcard = json!(["vcard", [["fn", {}]]]);
        assert!(matches!(
            Vcard::from_jcard(&jcard),
            Err(ParseJcardError::InvalidProp(_)),
        ));
    }
}
