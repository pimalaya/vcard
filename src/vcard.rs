//! # vCard
//!
//! The decoded card: a version indicator and its properties, in source order.
//!
//! The property, value, parameter and version types live in the sibling
//! modules. Like the rest of the decoded model, [`Vcard`] has no dependency on
//! [`crate::tree`]; rendering it back to bytes is a
//! [`Display`](core::fmt::Display) impl that lives on the syntax side.
//!
//! [`Vcard::to_version`] rewrites a card between vCard 3.0 (RFC 2426) and
//! 4.0 (RFC 6350 Appendix A), keeping every property it has no rule for.

use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::{
    param::VcardParam,
    prop::{VcardProp, VcardPropKind, VcardPropName},
    value::{VcardValue, VcardValueUnknown, binary::VcardBinary, text::VcardText, uri::VcardUri},
    version::VcardVersion,
};

/// A decoded card: its version and its properties, in source order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vcard<'a> {
    /// The card version.
    pub version: VcardVersion,
    /// The properties, in source order.
    pub properties: Vec<VcardProp<'a>>,
}

/// The Apple extension holding a 3.0 card's kind.
const APPLE_KIND: &str = "X-ADDRESSBOOKSERVER-KIND";

/// The Apple extension holding a 3.0 group's member.
const APPLE_MEMBER: &str = "X-ADDRESSBOOKSERVER-MEMBER";

impl<'a> Vcard<'a> {
    /// The card written as `version`.
    ///
    /// Up to 4.0 from an earlier version, a `pref` type is `PREF=1`, an
    /// inline `PHOTO`, `LOGO`, `SOUND` or `KEY` (`ENCODING=b`) is a `data:`
    /// URI and a referenced one drops its `VALUE=uri`, an `AGENT` URI is a
    /// `RELATED;TYPE=agent` (an inline agent card is dropped), and Apple's
    /// `X-ADDRESSBOOKSERVER-KIND` and `-MEMBER` are `KIND` and `MEMBER`.
    /// Down from 4.0 to 3.0, each rule runs the other way, a `PREF` other
    /// than 1 staying as written. Groups (`item1.`) and every other property
    /// and parameter are kept; any other pair of versions only changes the
    /// version.
    pub fn to_version(&self, version: VcardVersion) -> Vcard<'a> {
        let up = self.version != VcardVersion::V4_0 && version == VcardVersion::V4_0;
        let down = self.version == VcardVersion::V4_0 && version == VcardVersion::V3_0;

        let properties = match (up, down) {
            (true, _) => self.properties.iter().filter_map(prop_up).collect(),
            (_, true) => self.properties.iter().map(prop_down).collect(),
            _ => self.properties.clone(),
        };

        Vcard {
            version,
            properties,
        }
    }
}

/// One property written for 4.0, `None` when 4.0 has no place for it.
fn prop_up<'a>(prop: &VcardProp<'a>) -> Option<VcardProp<'a>> {
    let mut prop = prop.clone();
    let mut pref = false;

    for param in &mut prop.params {
        if let VcardParam::Type(types) = param {
            let before = types.len();
            types.retain(|kind| !kind.eq_ignore_ascii_case("pref"));
            pref |= types.len() != before;
        }
    }

    prop.params
        .retain(|param| !matches!(param, VcardParam::Type(types) if types.is_empty()));

    if pref && !prop.params.iter().any(|p| matches!(p, VcardParam::Pref(_))) {
        prop.params.push(VcardParam::Pref(Cow::Borrowed("1")));
    }

    let name = prop.name.clone();

    match &name {
        VcardPropName::Kind(kind) if media(*kind).is_some() => media_up(&mut prop, *kind),
        VcardPropName::Kind(VcardPropKind::Agent) => {
            let uri = text(&prop.value).filter(|uri| uri.contains(':') && !uri.contains('\n'))?;
            let inline = prop.params.iter().any(|param| {
                matches!(param, VcardParam::Value(kind) if !kind.eq_ignore_ascii_case("uri"))
            });

            if inline {
                return None;
            }

            prop.name = VcardPropName::Kind(VcardPropKind::Related);
            prop.params
                .retain(|param| !matches!(param, VcardParam::Value(_)));
            prop.params
                .push(VcardParam::Type(vec![Cow::Borrowed("agent")]));
            prop.value = VcardValue::Uri(VcardUri(Cow::Owned(uri)));
        }
        VcardPropName::Unknown(name) if name.eq_ignore_ascii_case(APPLE_KIND) => {
            prop.value = VcardValue::Text(VcardText(Cow::Owned(text(&prop.value)?)));
            prop.name = VcardPropName::Kind(VcardPropKind::Kind);
        }
        VcardPropName::Unknown(name) if name.eq_ignore_ascii_case(APPLE_MEMBER) => {
            prop.value = VcardValue::Uri(VcardUri(Cow::Owned(text(&prop.value)?)));
            prop.name = VcardPropName::Kind(VcardPropKind::Member);
        }
        _ => {}
    }

    Some(prop)
}

/// One property written for 3.0.
fn prop_down<'a>(prop: &VcardProp<'a>) -> VcardProp<'a> {
    let mut prop = prop.clone();
    let first = prop
        .params
        .iter()
        .position(|param| matches!(param, VcardParam::Pref(pref) if pref.as_ref() == "1"));

    if let Some(at) = first {
        prop.params.remove(at);

        match prop.params.iter_mut().find_map(|param| match param {
            VcardParam::Type(types) => Some(types),
            _ => None,
        }) {
            Some(types) => types.push(Cow::Borrowed("pref")),
            None => prop
                .params
                .push(VcardParam::Type(vec![Cow::Borrowed("pref")])),
        }
    }

    let name = prop.name.clone();

    match &name {
        VcardPropName::Kind(kind) if media(*kind).is_some() => media_down(&mut prop, *kind),
        VcardPropName::Kind(VcardPropKind::Related) if is_agent(&prop) => {
            prop.params
                .retain(|param| !matches!(param, VcardParam::Type(_)));
            prop.params.push(VcardParam::Value(Cow::Borrowed("uri")));
            prop.name = VcardPropName::Kind(VcardPropKind::Agent);
        }
        VcardPropName::Kind(VcardPropKind::Kind) => {
            prop.name = VcardPropName::Unknown(Cow::Borrowed(APPLE_KIND));
            prop.value = unknown(text(&prop.value).unwrap_or_default());
        }
        VcardPropName::Kind(VcardPropKind::Member) => {
            prop.name = VcardPropName::Unknown(Cow::Borrowed(APPLE_MEMBER));
            prop.value = unknown(text(&prop.value).unwrap_or_default());
        }
        _ => {}
    }

    prop
}

/// The top-level media type a binary property's 3.0 `TYPE` names.
fn media(kind: VcardPropKind) -> Option<&'static str> {
    match kind {
        VcardPropKind::Photo | VcardPropKind::Logo => Some("image"),
        VcardPropKind::Sound => Some("audio"),
        VcardPropKind::Key => Some("application"),
        _ => None,
    }
}

/// An inline 3.0 binary value as a `data:` URI, a reference as a URI.
fn media_up(prop: &mut VcardProp<'_>, kind: VcardPropKind) {
    let inline = prop.params.iter().any(|param| {
        matches!(param, VcardParam::Encoding(encoding) if encoding.eq_ignore_ascii_case("b") || encoding.eq_ignore_ascii_case("base64"))
    });

    let Some(value) = text(&prop.value) else {
        return;
    };

    if !inline {
        prop.params.retain(
            |param| !matches!(param, VcardParam::Value(kind) if kind.eq_ignore_ascii_case("uri")),
        );
        prop.value = VcardValue::Uri(VcardUri(Cow::Owned(value)));
        return;
    }

    let subtype = prop.params.iter().find_map(|param| match param {
        VcardParam::Type(types) => types.first().map(|kind| kind.to_ascii_lowercase()),
        _ => None,
    });

    let mediatype = match subtype {
        Some(subtype) if subtype.contains('/') => subtype,
        Some(subtype) => format!("{}/{subtype}", media(kind).unwrap_or("application")),
        None => String::from("application/octet-stream"),
    };

    prop.params.retain(|param| {
        !matches!(
            param,
            VcardParam::Encoding(_) | VcardParam::Type(_) | VcardParam::Value(_)
        )
    });

    let data: String = value.split_whitespace().collect();
    prop.value = VcardValue::Uri(VcardUri(Cow::Owned(format!(
        "data:{mediatype};base64,{data}"
    ))));
}

/// A 4.0 `data:` URI as an inline 3.0 binary value, any other as a reference.
fn media_down(prop: &mut VcardProp<'_>, kind: VcardPropKind) {
    let Some(value) = text(&prop.value) else {
        return;
    };

    let inline = value
        .strip_prefix("data:")
        .and_then(|rest| rest.split_once(";base64,"));

    let Some((mediatype, data)) = inline else {
        prop.params.push(VcardParam::Value(Cow::Borrowed("uri")));
        prop.value = VcardValue::Binary(VcardBinary::Uri(Cow::Owned(value)));
        return;
    };

    let prefix = format!("{}/", media(kind).unwrap_or("application"));
    let subtype = match mediatype.strip_prefix(&prefix) {
        Some(subtype) => subtype.to_ascii_uppercase(),
        None => mediatype.to_string(),
    };

    prop.params.push(VcardParam::Encoding(Cow::Borrowed("b")));

    if mediatype != "application/octet-stream" && !mediatype.is_empty() {
        prop.params
            .push(VcardParam::Type(vec![Cow::Owned(subtype)]));
    }

    prop.value = VcardValue::Binary(VcardBinary::Base64(Cow::Owned(data.to_string())));
}

/// Whether a `RELATED` names an agent and nothing else.
fn is_agent(prop: &VcardProp<'_>) -> bool {
    let types: Vec<&Cow<'_, str>> = prop
        .params
        .iter()
        .filter_map(|param| match param {
            VcardParam::Type(types) => Some(types.iter()),
            _ => None,
        })
        .flatten()
        .collect();

    matches!(types.as_slice(), [only] if only.eq_ignore_ascii_case("agent"))
}

/// The one text a value holds, whatever its kind.
fn text(value: &VcardValue<'_>) -> Option<String> {
    let text = match value {
        VcardValue::Text(text) => &text.0,
        VcardValue::Uri(uri) => &uri.0,
        VcardValue::Binary(VcardBinary::Uri(text) | VcardBinary::Base64(text)) => text,
        VcardValue::Unknown(unknown) => match unknown.components.as_slice() {
            [component] => return Some(component.join(",")),
            _ => return None,
        },
        _ => return None,
    };

    Some(text.to_string())
}

/// A text as the value of a property this crate does not know.
fn unknown(text: String) -> VcardValue<'static> {
    VcardValue::Unknown(VcardValueUnknown {
        components: vec![vec![Cow::Owned(text)]],
    })
}
