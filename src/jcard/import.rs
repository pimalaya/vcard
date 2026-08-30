//! # Import
//!
//! The jCard-to-model half: a jCard property entry, parameter member and
//! value slot read back into the decoded model.
//!
//! In, anything is accepted. Unknown names, parameters and type slots survive
//! verbatim, non-string scalars are coerced to text, and a type slot naming no
//! known kind falls back to the property's version default.
//!
//! A value kind is resolved through the same spec vtable as the wire decoder,
//! so a jCard and the vCard it came from decode to the same model.

use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde_json::{Map, Value};

use crate::{
    jcard::datetime::{extended_to_basic, offset_to_basic},
    param::{VcardParam, VcardParamKind},
    prop::spec::prop_spec,
    prop::{VcardProp, VcardPropName},
    value::{
        VcardValue, VcardValueKind, VcardValueUnknown,
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
    version::VcardVersion,
};
impl<'a> VcardProp<'a> {
    /// Read one jCard entry into a decoded property, resolving its value kind
    /// through the property spec like the wire decoder does.
    ///
    /// Also the decoder behind the RFC 9555 vCardProps escape hatch.
    pub(crate) fn from_jcard(
        name: &'a str,
        params: &'a Map<String, Value>,
        slot: &'a str,
        values: &'a [Value],
        version: VcardVersion,
    ) -> Self {
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
            .map(|(key, value)| VcardParam::from_jcard(key, value))
            .collect();

        // NOTE: The type slot goes back to a VALUE parameter only where the wire
        // form would need one: when it differs from the kind the spec would pick
        // with no declaration (for an unknown property, from its "unknown"
        // default).
        let kind = match &name {
            VcardPropName::Kind(prop) => {
                let spec = prop_spec(*prop);
                let default = (spec.value)(version, None);

                // NOTE: jCard types every text-shaped value "text" (a structured
                // N is `["n", {}, "text", [...]]`), so against a text-shaped
                // default the slot declares nothing and must not flatten the
                // structured kind to a single text.
                let declared = slot.parse::<VcardValueKind>().ok().filter(|declared| {
                    !(*declared == VcardValueKind::Text && default.is_text_shaped())
                });

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
            Some(kind) => VcardValue::from_jcard(kind, values),
            None => VcardValue::Unknown(VcardValueUnknown::from_jcard(values)),
        };

        Self {
            name,
            params: prop_params,
            value,
        }
    }
}

impl<'a> VcardParam<'a> {
    /// Read one jCard params-object member into a decoded parameter.
    ///
    /// Also the decoder behind the RFC 9555 vCardParams escape hatch.
    pub(crate) fn from_jcard(key: &'a str, value: &'a Value) -> Self {
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
}

impl<'a> VcardValue<'a> {
    /// Read jCard value slots as the given decoded value kind.
    fn from_jcard(kind: VcardValueKind, values: &'a [Value]) -> Self {
        match kind {
            VcardValueKind::Text => VcardValue::Text(VcardText(first_scalar(values))),
            VcardValueKind::TextList => {
                VcardValue::TextList(VcardTextList(values.iter().flat_map(scalars).collect()))
            }
            VcardValueKind::Uri => VcardValue::Uri(VcardUri(first_scalar(values))),
            VcardValueKind::DateAndOrTime => VcardValue::DateAndOrTime(VcardDateAndOrTime(
                extended_to_basic(first_scalar(values)),
            )),
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
}

impl<'a> VcardValueUnknown<'a> {
    /// Read an undecoded value's slots back into semicolon and comma components,
    /// the inverse of the export.
    fn from_jcard(values: &'a [Value]) -> Self {
        let components = match values {
            [] => vec![vec![Cow::Borrowed("")]],
            [Value::Array(groups)] if groups.iter().any(Value::is_array) => {
                groups.iter().map(scalars).collect()
            }
            [Value::Array(group)] => vec![group.iter().map(scalar).collect()],
            [value] => vec![vec![scalar(value)]],
            values => values.iter().map(scalars).collect(),
        };

        Self { components }
    }
}

impl VcardValueKind {
    /// Whether jCard spells the kind with the plain "text" type slot: the single
    /// and list text kinds plus the structured text shapes.
    fn is_text_shaped(self) -> bool {
        matches!(
            self,
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
