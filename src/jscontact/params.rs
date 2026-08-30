//! # Converted parameters
//!
//! The parameter split shared by every converted object.
//!
//! `TYPE` maps to contexts (`home` becomes `private`) and, on `TEL`, to
//! features (`cell` becomes `mobile`); `PREF` maps to `pref` and `PROP-ID` to
//! the map key. `LABEL`, `MEDIATYPE`, `SORT-AS`, `GEO` and `TZ` are consumed
//! by whichever member wants them, and everything left over rides the
//! vCardParams escape hatch.

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use serde_json::{Map, Value};

use crate::{jcard::export::merge_param, param::VcardParam};
/// The parameter split shared by every converted object: contexts and
/// features from TYPE, `pref` from PREF, the map key from PROP-ID, the
/// consumable specials (LABEL, MEDIATYPE, SORT-AS, GEO, TZ), and the
/// unconverted rest bound for vCardParams.
pub(super) struct ConvertedParams<'a> {
    contexts: Map<String, Value>,
    features: Map<String, Value>,
    pref: Option<u64>,
    prop_id: Option<String>,
    label: Option<&'a VcardParam<'a>>,
    media_type: Option<&'a VcardParam<'a>>,
    sort_as: Option<&'a VcardParam<'a>>,
    geo: Option<&'a VcardParam<'a>>,
    tz: Option<&'a VcardParam<'a>>,
    service_type: Option<&'a VcardParam<'a>>,
    username: Option<&'a VcardParam<'a>>,
    rest: Vec<&'a VcardParam<'a>>,
}

impl<'a> ConvertedParams<'a> {
    /// Split a property's parameters; `phone` routes the TYPE feature values
    /// (voice, cell, ...) to features instead of contexts.
    pub(super) fn split(params: &'a [VcardParam<'a>], phone: bool) -> Self {
        let mut split = Self {
            contexts: Map::new(),
            features: Map::new(),
            pref: None,
            prop_id: None,
            label: None,
            media_type: None,
            sort_as: None,
            geo: None,
            tz: None,
            service_type: None,
            username: None,
            rest: Vec::new(),
        };

        for param in params {
            match param {
                VcardParam::Type(values) => {
                    for value in values.iter().filter(|value| !value.is_empty()) {
                        split.r#type(&value.to_ascii_lowercase(), phone);
                    }
                }
                VcardParam::Pref(value) => split.pref = value.trim().parse().ok(),
                VcardParam::PropId(value) => split.prop_id = Some(value.to_string()),
                VcardParam::Label(_) => split.label = Some(param),
                VcardParam::MediaType(_) => split.media_type = Some(param),
                VcardParam::SortAs(_) => split.sort_as = Some(param),
                VcardParam::Geo(_) => split.geo = Some(param),
                VcardParam::Tz(_) => split.tz = Some(param),
                VcardParam::ServiceType(_) => split.service_type = Some(param),
                VcardParam::Username(_) => split.username = Some(param),
                // NOTE: the declared VALUE is already reflected by the
                // decoded value's kind, nothing left to preserve.
                VcardParam::Value(_) => {}
                param => split.rest.push(param),
            }
        }

        split
    }

    /// Route one lowercased TYPE value: work / home to contexts, `pref` to
    /// pref, a feature to features on a phone, anything else kept verbatim.
    fn r#type(&mut self, value: &str, phone: bool) {
        match value {
            "work" => {
                self.contexts.insert("work".into(), Value::Bool(true));
            }
            "home" => {
                self.contexts.insert("private".into(), Value::Bool(true));
            }
            "pref" => {
                self.pref.get_or_insert(1);
            }
            "cell" if phone => {
                self.features.insert("mobile".into(), Value::Bool(true));
            }
            value if phone => {
                self.features.insert(value.into(), Value::Bool(true));
            }
            value => {
                self.contexts.insert(value.into(), Value::Bool(true));
            }
        }
    }

    /// Consume the LABEL parameter's text.
    /// The map key the object goes under, where the property named one.
    pub(super) fn take_prop_id(&mut self) -> Option<String> {
        self.prop_id.take()
    }

    pub(super) fn take_label(&mut self) -> Option<&'a str> {
        match self.label.take() {
            Some(VcardParam::Label(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Consume the MEDIATYPE parameter's text.
    pub(super) fn take_media_type(&mut self) -> Option<&'a str> {
        match self.media_type.take() {
            Some(VcardParam::MediaType(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Consume the SORT-AS parameter's values.
    pub(super) fn take_sort_as(&mut self) -> Option<&'a [Cow<'a, str>]> {
        match self.sort_as.take() {
            Some(VcardParam::SortAs(values)) => Some(values.as_slice()),
            _ => None,
        }
    }

    /// Consume the GEO parameter's text.
    pub(super) fn take_geo(&mut self) -> Option<&'a str> {
        match self.geo.take() {
            Some(VcardParam::Geo(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Consume the TZ parameter's text.
    pub(super) fn take_tz(&mut self) -> Option<&'a str> {
        match self.tz.take() {
            Some(VcardParam::Tz(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Consume the SERVICE-TYPE parameter's text.
    pub(super) fn take_service_type(&mut self) -> Option<&'a str> {
        match self.service_type.take() {
            Some(VcardParam::ServiceType(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Consume the USERNAME parameter's text.
    pub(super) fn take_username(&mut self) -> Option<&'a str> {
        match self.username.take() {
            Some(VcardParam::Username(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Close the object: contexts, features and pref where present, and
    /// everything unconsumed as the vCardParams escape hatch, in jCard
    /// parameter syntax.
    pub(super) fn finish(self, object: &mut Map<String, Value>) {
        if !self.contexts.is_empty() {
            object.insert("contexts".into(), Value::Object(self.contexts));
        }
        if !self.features.is_empty() {
            object.insert("features".into(), Value::Object(self.features));
        }
        if let Some(pref) = self.pref {
            object.insert("pref".into(), Value::from(pref));
        }

        let unconsumed = [
            self.label,
            self.media_type,
            self.sort_as,
            self.geo,
            self.tz,
            self.service_type,
            self.username,
        ];
        let mut escaped = Map::new();
        for param in self
            .rest
            .into_iter()
            .chain(unconsumed.into_iter().flatten())
        {
            let (key, value) = param.to_jcard();
            merge_param(&mut escaped, key, value);
        }
        if !escaped.is_empty() {
            object.insert("vCardParams".into(), Value::Object(escaped));
        }
    }
}
