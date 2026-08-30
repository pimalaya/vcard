//! # Import
//!
//! The JSContact-to-model half: each Card member read back into the decoded
//! properties it stands for.
//!
//! [`Import`] walks the members in order and appends the properties each one
//! yields, so a card comes back with its properties grouped the way JSContact
//! grouped them. A member, an entry or a piece of an entry that maps to no
//! property is preserved as a `JSPROP` at its own JSON pointer, and a
//! `vCardProps` entry is read straight back through the jCard decoder.

use core::mem;

use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde_json::{Map, Value};

use crate::{
    jcard::datetime::extended_to_basic,
    jscontact::{date::date_from_jscontact, pointer::escape_pointer},
    param::VcardParam,
    prop::{VcardProp, VcardPropKind, VcardPropName},
    value::{
        VcardValue,
        adr::VcardAdr,
        datetime::{VcardDateAndOrTime, VcardTimestamp},
        language::VcardLanguageTag,
        n::VcardN,
        org::VcardOrg,
        text::{VcardText, VcardTextList},
        uri::VcardUri,
    },
    version::VcardVersion,
};
/// A component-shaped object (a `kind` naming a slot through `slot_of`, a
/// string `value`, nothing else): its slot and text, `None` when it is
/// anything else so the caller can preserve the whole piece.
fn component_slot(component: &Value, slot_of: fn(&str) -> Option<usize>) -> Option<(usize, &str)> {
    let object = component.as_object()?;
    let known = object
        .keys()
        .all(|member| matches!(member.as_str(), "@type" | "kind" | "value" | "name"));
    if !known {
        return None;
    }

    let text = object
        .get("value")
        .or_else(|| object.get("name"))
        .and_then(Value::as_str)?;
    let slot = match object.get("kind") {
        Some(kind) => slot_of(kind.as_str()?)?,
        None => slot_of("")?,
    };
    Some((slot, text))
}

/// A `sortAs` object's values in N slot order (surname then given), `None`
/// when it carries anything else.
fn sort_as_values(sort_as: &Value) -> Option<Vec<Cow<'_, str>>> {
    let object = sort_as.as_object()?;
    let known = object
        .keys()
        .all(|member| matches!(member.as_str(), "@type" | "surname" | "given"));
    if !known {
        return None;
    }

    let surname = object.get("surname").and_then(Value::as_str);
    let given = object.get("given").and_then(Value::as_str);
    match (surname, given) {
        (Some(surname), Some(given)) => Some(vec![Cow::Borrowed(surname), Cow::Borrowed(given)]),
        (Some(surname), None) => Some(vec![Cow::Borrowed(surname)]),
        (None, Some(given)) => Some(vec![Cow::Borrowed(""), Cow::Borrowed(given)]),
        (None, None) => None,
    }
}

/// The parameters escaped in a vCardParams object, decoded back.
fn escaped_params(value: &Value) -> Vec<VcardParam<'_>> {
    value
        .as_object()
        .into_iter()
        .flatten()
        .map(|(key, value)| VcardParam::from_jcard(key, value))
        .collect()
}

/// The keys of a JSON object, or nothing for any other shape.
fn object_keys(value: &Value) -> impl Iterator<Item = &str> {
    value
        .as_object()
        .into_iter()
        .flatten()
        .map(|(key, _)| key.as_str())
}

/// The decoded value wraps the import scalars pick from.
fn text_value(value: Cow<'_, str>) -> VcardValue<'_> {
    VcardValue::Text(VcardText(value))
}

/// See [`text_value`].
fn uri_value(value: Cow<'_, str>) -> VcardValue<'_> {
    VcardValue::Uri(VcardUri(value))
}

/// See [`text_value`].
fn language_value(value: Cow<'_, str>) -> VcardValue<'_> {
    VcardValue::LanguageTag(VcardLanguageTag(value))
}

/// See [`text_value`]; the extended UTC date-time comes back basic.
fn timestamp_value(value: Cow<'_, str>) -> VcardValue<'_> {
    VcardValue::Timestamp(VcardTimestamp(extended_to_basic(value)))
}

/// See [`text_value`]; a Nickname holds one name, a one-item NICKNAME list.
fn nickname_value(value: Cow<'_, str>) -> VcardValue<'_> {
    VcardValue::TextList(VcardTextList(vec![value]))
}

/// The media property for a Media kind.
fn media_kind(kind: Option<&str>) -> Option<VcardPropKind> {
    match kind {
        Some("photo") => Some(VcardPropKind::Photo),
        Some("logo") => Some(VcardPropKind::Logo),
        Some("sound") => Some(VcardPropKind::Sound),
        _ => None,
    }
}

/// The calendar property for a Calendar kind.
fn calendar_kind(kind: Option<&str>) -> Option<VcardPropKind> {
    match kind {
        Some("calendar") => Some(VcardPropKind::CalUri),
        Some("freeBusy") => Some(VcardPropKind::FbUrl),
        _ => None,
    }
}

/// The directory property for a Directory kind.
fn directory_kind(kind: Option<&str>) -> Option<VcardPropKind> {
    match kind {
        Some("entry") => Some(VcardPropKind::Source),
        _ => None,
    }
}

/// A kind-less CryptoKey is a KEY.
fn crypto_kind(kind: Option<&str>) -> Option<VcardPropKind> {
    kind.is_none().then_some(VcardPropKind::Key)
}

/// A kind-less SchedulingAddress is a CALADRURI.
fn scheduling_kind(kind: Option<&str>) -> Option<VcardPropKind> {
    kind.is_none().then_some(VcardPropKind::CalAdrUri)
}

/// A kind-less Link is a URL.
fn link_kind(kind: Option<&str>) -> Option<VcardPropKind> {
    kind.is_none().then_some(VcardPropKind::Url)
}

/// An OrgUnit carries no kind; its single slot is the unit name.
fn unit_slot(kind: &str) -> Option<usize> {
    kind.is_empty().then_some(0)
}

/// The card under reconstruction: the decoded properties a Card converts
/// back into, in member order.
#[derive(Default)]
pub(super) struct Import<'a> {
    pub(super) properties: Vec<VcardProp<'a>>,
}

impl<'a> Import<'a> {
    /// Convert one Card member, preserving what does not map as JSPROP.
    pub(super) fn member(&mut self, member: &'a str, value: &'a Value) {
        match member {
            "@type" | "version" => {}
            "uid" => self.scalar(member, value, VcardPropKind::Uid, uri_value),
            "prodId" => self.scalar(member, value, VcardPropKind::ProdId, text_value),
            "kind" => self.scalar(member, value, VcardPropKind::Kind, text_value),
            "language" => self.scalar(member, value, VcardPropKind::Language, language_value),
            "created" => self.scalar(member, value, VcardPropKind::Created, timestamp_value),
            "updated" => self.scalar(member, value, VcardPropKind::Rev, timestamp_value),
            "name" => self.name(value),
            "speakToAs" => self.speak_to_as(value),
            "nicknames" => self.simple(
                "/nicknames",
                value,
                VcardPropKind::Nickname,
                "name",
                false,
                nickname_value,
            ),
            "emails" => self.simple(
                "/emails",
                value,
                VcardPropKind::Email,
                "address",
                false,
                text_value,
            ),
            "phones" => self.simple(
                "/phones",
                value,
                VcardPropKind::Tel,
                "number",
                true,
                text_value,
            ),
            "preferredLanguages" => self.simple(
                "/preferredLanguages",
                value,
                VcardPropKind::Lang,
                "language",
                false,
                language_value,
            ),
            "notes" => self.simple(
                "/notes",
                value,
                VcardPropKind::Note,
                "note",
                false,
                text_value,
            ),
            "onlineServices" => self.online_services(value),
            "organizations" => self.organizations(value),
            "titles" => self.titles(value),
            "addresses" => self.addresses(value),
            "anniversaries" => self.anniversaries(value),
            "media" => self.resources("/media", value, media_kind),
            "cryptoKeys" => self.resources("/cryptoKeys", value, crypto_kind),
            "calendars" => self.resources("/calendars", value, calendar_kind),
            "schedulingAddresses" => self.resources("/schedulingAddresses", value, scheduling_kind),
            "links" => self.resources("/links", value, link_kind),
            "directories" => self.resources("/directories", value, directory_kind),
            "keywords" => self.keywords(value),
            "members" => self.members(value),
            "relatedTo" => self.related_to(value),
            "vCardProps" => self.vcard_props(value),
            member => self.jsprop_member(member, value),
        }
    }

    /// A single string member as one paramless property.
    fn scalar(
        &mut self,
        member: &'a str,
        value: &'a Value,
        kind: VcardPropKind,
        wrap: fn(Cow<'a, str>) -> VcardValue<'a>,
    ) {
        match value.as_str() {
            Some(text) => self.prop(kind, Vec::new(), wrap(Cow::Borrowed(text))),
            None => self.jsprop_member(member, value),
        }
    }

    /// The `name` member as FN and N (with SORT-AS from `sortAs`).
    fn name(&mut self, value: &'a Value) {
        let Some(object) = value.as_object() else {
            return self.jsprop("/name".to_string(), value);
        };

        let mut n = VcardN::default();
        let mut has_components = false;
        if let Some(components) = object.get("components") {
            let Some(entries) = components.as_array() else {
                return self.jsprop("/name".to_string(), value);
            };
            has_components = true;

            for component in entries {
                let slot = component_slot(component, |kind| match kind {
                    "surname" => Some(0),
                    "given" => Some(1),
                    "given2" => Some(2),
                    "title" => Some(3),
                    "credential" => Some(4),
                    _ => None,
                });
                let Some((slot, text)) = slot else {
                    return self.jsprop("/name".to_string(), value);
                };
                let slot = match slot {
                    0 => &mut n.family,
                    1 => &mut n.given,
                    2 => &mut n.additional,
                    3 => &mut n.prefixes,
                    _ => &mut n.suffixes,
                };
                slot.push(Cow::Borrowed(text));
            }
        }

        let mut params = Vec::new();
        for (member, value) in object {
            match member.as_str() {
                "@type" | "full" | "components" | "sortAs" => {}
                "vCardParams" => params.extend(escaped_params(value)),
                member => {
                    let pointer = format!("/name/{}", escape_pointer(member));
                    self.jsprop(pointer, value);
                }
            }
        }

        if let Some(sort_as) = object.get("sortAs") {
            match sort_as_values(sort_as) {
                Some(values) => params.push(VcardParam::SortAs(values)),
                None => self.jsprop("/name/sortAs".to_string(), sort_as),
            }
        }

        if let Some(full) = object.get("full") {
            match full.as_str() {
                Some(full) => {
                    let params = if has_components {
                        Vec::new()
                    } else {
                        mem::take(&mut params)
                    };
                    self.prop(VcardPropKind::Fn, params, text_value(Cow::Borrowed(full)));
                }
                None => self.jsprop("/name/full".to_string(), full),
            }
        }

        if has_components {
            self.prop(VcardPropKind::N, params, VcardValue::N(n));
        }
    }

    /// The `speakToAs` member as GRAMGENDER and PRONOUNS.
    fn speak_to_as(&mut self, value: &'a Value) {
        let Some(object) = value.as_object() else {
            return self.jsprop("/speakToAs".to_string(), value);
        };

        for (member, value) in object {
            match member.as_str() {
                "@type" => {}
                "grammaticalGender" => match value.as_str() {
                    Some(gender) => self.prop(
                        VcardPropKind::GramGender,
                        Vec::new(),
                        text_value(Cow::Borrowed(gender)),
                    ),
                    None => self.jsprop("/speakToAs/grammaticalGender".to_string(), value),
                },
                "pronouns" => self.simple(
                    "/speakToAs/pronouns",
                    value,
                    VcardPropKind::Pronouns,
                    "pronouns",
                    false,
                    text_value,
                ),
                member => {
                    let pointer = format!("/speakToAs/{}", escape_pointer(member));
                    self.jsprop(pointer, value);
                }
            }
        }
    }

    /// A collection of one-text-member objects (emails, notes, ...), each as
    /// one property.
    fn simple(
        &mut self,
        prefix: &str,
        collection: &'a Value,
        kind: VcardPropKind,
        member: &str,
        phone: bool,
        wrap: fn(Cow<'a, str>) -> VcardValue<'a>,
    ) {
        let Some(collection) = collection.as_object() else {
            return self.jsprop(prefix.to_string(), collection);
        };

        for (key, entry) in collection {
            let object = entry.as_object();
            let value = object
                .and_then(|object| object.get(member))
                .and_then(Value::as_str);
            let (Some(object), Some(value)) = (object, value) else {
                self.jsprop_entry(prefix, key, entry);
                continue;
            };

            let params = self.common_params(prefix, key, object, &[member], phone);
            self.prop(kind, params, wrap(Cow::Borrowed(value)));
        }
    }

    /// `onlineServices` back to SOCIALPROFILE (a `service` or `user` member)
    /// or IMPP (a bare URI).
    fn online_services(&mut self, collection: &'a Value) {
        let prefix = "/onlineServices";
        let Some(collection) = collection.as_object() else {
            return self.jsprop(prefix.to_string(), collection);
        };

        for (key, entry) in collection {
            let Some(object) = entry.as_object() else {
                self.jsprop_entry(prefix, key, entry);
                continue;
            };
            let uri = object.get("uri").and_then(Value::as_str);
            let user = object.get("user").and_then(Value::as_str);
            let service = object.get("service").and_then(Value::as_str);

            if service.is_none() && user.is_none() {
                let Some(uri) = uri else {
                    self.jsprop_entry(prefix, key, entry);
                    continue;
                };
                let params = self.common_params(prefix, key, object, &["uri"], false);
                self.prop(VcardPropKind::Impp, params, uri_value(Cow::Borrowed(uri)));
                continue;
            }

            if uri.is_none() && user.is_none() {
                self.jsprop_entry(prefix, key, entry);
                continue;
            }

            let mut params =
                self.common_params(prefix, key, object, &["uri", "user", "service"], false);
            if let Some(service) = service {
                params.push(VcardParam::ServiceType(Cow::Borrowed(service)));
            }
            match (uri, user) {
                (Some(uri), user) => {
                    if let Some(user) = user {
                        params.push(VcardParam::Username(Cow::Borrowed(user)));
                    }
                    self.prop(
                        VcardPropKind::SocialProfile,
                        params,
                        uri_value(Cow::Borrowed(uri)),
                    );
                }
                (None, user) => {
                    params.push(VcardParam::Value(Cow::Borrowed("text")));
                    let user = user.unwrap_or_default();
                    self.prop(
                        VcardPropKind::SocialProfile,
                        params,
                        text_value(Cow::Borrowed(user)),
                    );
                }
            }
        }
    }

    /// `organizations` back to ORG (with SORT-AS from `sortAs`).
    fn organizations(&mut self, collection: &'a Value) {
        let prefix = "/organizations";
        let Some(collection) = collection.as_object() else {
            return self.jsprop(prefix.to_string(), collection);
        };

        for (key, entry) in collection {
            let Some(object) = entry.as_object() else {
                self.jsprop_entry(prefix, key, entry);
                continue;
            };

            let mut units: Vec<Cow<'a, str>> = Vec::new();
            units.extend(
                object
                    .get("name")
                    .and_then(Value::as_str)
                    .map(Cow::Borrowed),
            );

            let mut valid = !units.is_empty();
            if let Some(entries) = object.get("units") {
                match entries.as_array() {
                    Some(entries) => {
                        for unit in entries {
                            match component_slot(unit, unit_slot) {
                                Some((_, name)) => units.push(Cow::Borrowed(name)),
                                None => valid = false,
                            }
                        }
                    }
                    None => valid = false,
                }
            }
            if !valid {
                self.jsprop_entry(prefix, key, entry);
                continue;
            }

            let mut params =
                self.common_params(prefix, key, object, &["name", "units", "sortAs"], false);
            if let Some(sort_as) = object.get("sortAs").and_then(Value::as_str) {
                params.push(VcardParam::SortAs(vec![Cow::Borrowed(sort_as)]));
            }
            self.prop(VcardPropKind::Org, params, VcardValue::Org(VcardOrg(units)));
        }
    }

    /// `titles` back to TITLE or ROLE, told apart by `kind`.
    fn titles(&mut self, collection: &'a Value) {
        let prefix = "/titles";
        let Some(collection) = collection.as_object() else {
            return self.jsprop(prefix.to_string(), collection);
        };

        for (key, entry) in collection {
            let object = entry.as_object();
            let name = object
                .and_then(|object| object.get("name"))
                .and_then(Value::as_str);
            let kind = object
                .and_then(|object| object.get("kind"))
                .and_then(Value::as_str)
                .unwrap_or("title");
            let kind = match kind {
                "title" => Some(VcardPropKind::Title),
                "role" => Some(VcardPropKind::Role),
                _ => None,
            };
            let (Some(object), Some(name), Some(kind)) = (object, name, kind) else {
                self.jsprop_entry(prefix, key, entry);
                continue;
            };

            let params = self.common_params(prefix, key, object, &["name", "kind"], false);
            self.prop(kind, params, text_value(Cow::Borrowed(name)));
        }
    }

    /// `addresses` back to ADR (with LABEL, GEO and TZ parameters).
    fn addresses(&mut self, collection: &'a Value) {
        let prefix = "/addresses";
        let Some(collection) = collection.as_object() else {
            return self.jsprop(prefix.to_string(), collection);
        };

        for (key, entry) in collection {
            let Some(object) = entry.as_object() else {
                self.jsprop_entry(prefix, key, entry);
                continue;
            };

            let mut adr = VcardAdr::default();
            let mut valid = true;
            if let Some(components) = object.get("components") {
                match components.as_array() {
                    Some(entries) => {
                        for component in entries {
                            // NOTE: apartment and name land on the legacy
                            // extended-address / street slots, the pair the
                            // export reads first, so consumers keep seeing
                            // the classic seven components.
                            let slot = component_slot(component, |kind| match kind {
                                "postOfficeBox" => Some(0),
                                "apartment" => Some(1),
                                "name" => Some(2),
                                "locality" => Some(3),
                                "region" => Some(4),
                                "postcode" => Some(5),
                                "country" => Some(6),
                                "room" => Some(7),
                                "floor" => Some(9),
                                "number" => Some(10),
                                "building" => Some(12),
                                "block" => Some(13),
                                "subdistrict" => Some(14),
                                "district" => Some(15),
                                "landmark" => Some(16),
                                "direction" => Some(17),
                                _ => None,
                            });
                            match slot {
                                Some((slot, text)) => {
                                    let slot = match slot {
                                        0 => &mut adr.po_box,
                                        1 => &mut adr.extended,
                                        2 => &mut adr.street,
                                        3 => &mut adr.locality,
                                        4 => &mut adr.region,
                                        5 => &mut adr.postal_code,
                                        6 => &mut adr.country,
                                        7 => &mut adr.room,
                                        9 => &mut adr.floor,
                                        10 => &mut adr.street_number,
                                        12 => &mut adr.building,
                                        13 => &mut adr.block,
                                        14 => &mut adr.subdistrict,
                                        15 => &mut adr.district,
                                        16 => &mut adr.landmark,
                                        _ => &mut adr.direction,
                                    };
                                    slot.push(Cow::Borrowed(text));
                                }
                                None => valid = false,
                            }
                        }
                    }
                    None => valid = false,
                }
            }
            if !valid {
                self.jsprop_entry(prefix, key, entry);
                continue;
            }

            let consumed = ["components", "full", "coordinates", "timeZone"];
            let mut params = self.common_params(prefix, key, object, &consumed, false);
            if let Some(full) = object.get("full").and_then(Value::as_str) {
                params.push(VcardParam::Label(Cow::Borrowed(full)));
            }
            if let Some(coordinates) = object.get("coordinates").and_then(Value::as_str) {
                params.push(VcardParam::Geo(Cow::Borrowed(coordinates)));
            }
            if let Some(time_zone) = object.get("timeZone").and_then(Value::as_str) {
                params.push(VcardParam::Tz(Cow::Borrowed(time_zone)));
            }
            self.prop(VcardPropKind::Adr, params, VcardValue::Adr(adr));
        }
    }

    /// `anniversaries` back to BDAY or ANNIVERSARY, told apart by `kind`.
    fn anniversaries(&mut self, collection: &'a Value) {
        let prefix = "/anniversaries";
        let Some(collection) = collection.as_object() else {
            return self.jsprop(prefix.to_string(), collection);
        };

        for (key, entry) in collection {
            let object = entry.as_object();
            let kind = object
                .and_then(|object| object.get("kind"))
                .and_then(Value::as_str);
            let kind = match kind {
                Some("birth") => Some(VcardPropKind::Bday),
                Some("wedding") => Some(VcardPropKind::Anniversary),
                _ => None,
            };
            let date = object
                .and_then(|object| object.get("date"))
                .and_then(date_from_jscontact);
            let (Some(object), Some(kind), Some(date)) = (object, kind, date) else {
                self.jsprop_entry(prefix, key, entry);
                continue;
            };

            let params = self.common_params(prefix, key, object, &["kind", "date"], false);
            self.prop(
                kind,
                params,
                VcardValue::DateAndOrTime(VcardDateAndOrTime(Cow::Owned(date))),
            );
        }
    }

    /// A resource collection back to its URI-valued property, the property
    /// picked from the entry's `kind` member. The entry's `@type` is
    /// ignored, so a Card written against an earlier JSContact draft
    /// converts back just as well.
    fn resources(
        &mut self,
        prefix: &str,
        collection: &'a Value,
        kind_of: fn(Option<&str>) -> Option<VcardPropKind>,
    ) {
        let Some(collection) = collection.as_object() else {
            return self.jsprop(prefix.to_string(), collection);
        };

        for (key, entry) in collection {
            let object = entry.as_object();
            let uri = object
                .and_then(|object| object.get("uri"))
                .and_then(Value::as_str);
            let kind = object
                .and_then(|object| object.get("kind"))
                .and_then(Value::as_str);
            let (Some(object), Some(uri), Some(prop_kind)) = (object, uri, kind_of(kind)) else {
                self.jsprop_entry(prefix, key, entry);
                continue;
            };

            let consumed = ["uri", "kind", "mediaType"];
            let mut params = self.common_params(prefix, key, object, &consumed, false);
            if let Some(media_type) = object.get("mediaType").and_then(Value::as_str) {
                params.push(VcardParam::MediaType(Cow::Borrowed(media_type)));
            }
            self.prop(prop_kind, params, uri_value(Cow::Borrowed(uri)));
        }
    }

    /// `keywords` back to one CATEGORIES property.
    fn keywords(&mut self, value: &'a Value) {
        match value.as_object() {
            Some(keywords) if !keywords.is_empty() => {
                let list = keywords
                    .keys()
                    .map(|keyword| Cow::Borrowed(keyword.as_str()))
                    .collect();
                self.prop(
                    VcardPropKind::Categories,
                    Vec::new(),
                    VcardValue::TextList(VcardTextList(list)),
                );
            }
            _ => self.jsprop_member("keywords", value),
        }
    }

    /// `members` back to one MEMBER property per entry.
    fn members(&mut self, value: &'a Value) {
        match value.as_object() {
            Some(members) if !members.is_empty() => {
                for uri in members.keys() {
                    self.prop(
                        VcardPropKind::Member,
                        Vec::new(),
                        uri_value(Cow::Borrowed(uri)),
                    );
                }
            }
            _ => self.jsprop_member("members", value),
        }
    }

    /// `relatedTo` back to RELATED, the relation set as the TYPE parameter.
    fn related_to(&mut self, collection: &'a Value) {
        let Some(collection) = collection.as_object() else {
            return self.jsprop_member("relatedTo", collection);
        };

        for (target, entry) in collection {
            let object = entry.as_object();
            let known = object.is_some_and(|object| {
                object
                    .keys()
                    .all(|member| matches!(member.as_str(), "@type" | "relation"))
            });
            let relation = object
                .and_then(|object| object.get("relation"))
                .and_then(Value::as_object);
            if !known || (object.is_some_and(|o| o.contains_key("relation")) && relation.is_none())
            {
                self.jsprop_entry("/relatedTo", target, entry);
                continue;
            }

            let mut params = Vec::new();
            let types: Vec<Cow<'a, str>> = relation
                .into_iter()
                .flatten()
                .map(|(relation, _)| Cow::Borrowed(relation.as_str()))
                .collect();
            if !types.is_empty() {
                params.push(VcardParam::Type(types));
            }
            self.prop(
                VcardPropKind::Related,
                params,
                uri_value(Cow::Borrowed(target)),
            );
        }
    }

    /// `vCardProps` entries back to properties, through the jCard decoder.
    fn vcard_props(&mut self, value: &'a Value) {
        let Some(entries) = value.as_array() else {
            return self.jsprop_member("vCardProps", value);
        };

        for (index, entry) in entries.iter().enumerate() {
            let parsed = entry.as_array().filter(|e| e.len() >= 3).and_then(|e| {
                let name = e[0].as_str()?;
                let params = e[1].as_object()?;
                let slot = e[2].as_str()?;
                Some(VcardProp::from_jcard(
                    name,
                    params,
                    slot,
                    &e[3..],
                    VcardVersion::V4_0,
                ))
            });
            match parsed {
                Some(prop) => self.properties.push(prop),
                None => self.jsprop(format!("/vCardProps/{index}"), entry),
            }
        }
    }

    /// The common inverse split of a converted object: the map key as
    /// PROP-ID, contexts and features back to TYPE, `pref` back to PREF,
    /// vCardParams back to parameters; an unknown member becomes a JSPROP at
    /// its pointer.
    fn common_params(
        &mut self,
        prefix: &str,
        key: &'a str,
        object: &'a Map<String, Value>,
        consumed: &[&str],
        phone: bool,
    ) -> Vec<VcardParam<'a>> {
        let mut params = vec![VcardParam::PropId(Cow::Borrowed(key))];
        let mut types: Vec<Cow<'a, str>> = Vec::new();

        for (member, value) in object {
            match member.as_str() {
                "@type" => {}
                member if consumed.contains(&member) => {}
                "contexts" => {
                    for context in object_keys(value) {
                        types.push(match context {
                            "private" => Cow::Borrowed("home"),
                            context => Cow::Borrowed(context),
                        });
                    }
                }
                "features" if phone => {
                    for feature in object_keys(value) {
                        types.push(match feature {
                            "mobile" => Cow::Borrowed("cell"),
                            feature => Cow::Borrowed(feature),
                        });
                    }
                }
                "pref" => {
                    if let Some(pref) = value.as_u64() {
                        params.push(VcardParam::Pref(Cow::Owned(pref.to_string())));
                    }
                }
                "vCardParams" => params.extend(escaped_params(value)),
                member => {
                    let pointer = format!(
                        "{prefix}/{}/{}",
                        escape_pointer(key),
                        escape_pointer(member),
                    );
                    self.jsprop(pointer, value);
                }
            }
        }

        if !types.is_empty() {
            params.push(VcardParam::Type(types));
        }

        params
    }

    /// Preserve a JSON piece as a JSPROP property at its pointer.
    fn jsprop(&mut self, pointer: String, value: &Value) {
        self.prop(
            VcardPropKind::JsProp,
            vec![VcardParam::Jsptr(Cow::Owned(pointer))],
            VcardValue::Text(VcardText(Cow::Owned(value.to_string()))),
        );
    }

    /// Preserve a whole unknown Card member.
    fn jsprop_member(&mut self, member: &str, value: &Value) {
        self.jsprop(format!("/{}", escape_pointer(member)), value);
    }

    /// Preserve a whole unconvertible collection entry.
    fn jsprop_entry(&mut self, prefix: &str, key: &str, entry: &Value) {
        self.jsprop(format!("{prefix}/{}", escape_pointer(key)), entry);
    }

    /// Push a known-name property.
    fn prop(&mut self, kind: VcardPropKind, params: Vec<VcardParam<'a>>, value: VcardValue<'a>) {
        self.properties.push(VcardProp {
            name: VcardPropName::Kind(kind),
            params,
            value,
        });
    }
}
