//! # Export
//!
//! The model-to-JSContact half: a decoded card assembled member by member.
//!
//! [`Card`] holds one field per RFC 9553 member and fills them property by
//! property, so the properties a card carries decide which members the JSON
//! object ends up with. A property that maps to no member, or one whose
//! parameters cannot ride the member it maps to, goes through the vCardProps
//! escape hatch instead.
//!
//! The map keys of a collection come from `PROP-ID` where a property carries
//! one, and from the running ordinal otherwise.

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use serde_json::{Map, Value, json};

use crate::{
    jscontact::{
        date::{date_object, utc_timestamp},
        params::ConvertedParams,
        pointer::set_pointer,
    },
    param::VcardParam,
    prop::{VcardProp, VcardPropKind, VcardPropName},
    value::{VcardValue, binary::VcardBinary},
};
/// The Card under construction: one field per JSContact member, filled
/// property by property, plus the vCardProps escape hatch.
#[derive(Default)]
pub(super) struct Card {
    uid: Option<Value>,
    prod_id: Option<Value>,
    updated: Option<Value>,
    created: Option<Value>,
    kind: Option<Value>,
    language: Option<Value>,
    gram_gender: Option<Value>,
    pronouns: Map<String, Value>,
    name: Map<String, Value>,
    addresses: Map<String, Value>,
    anniversaries: Map<String, Value>,
    calendars: Map<String, Value>,
    crypto_keys: Map<String, Value>,
    directories: Map<String, Value>,
    emails: Map<String, Value>,
    keywords: Map<String, Value>,
    links: Map<String, Value>,
    media: Map<String, Value>,
    members: Map<String, Value>,
    nicknames: Map<String, Value>,
    notes: Map<String, Value>,
    online_services: Map<String, Value>,
    organizations: Map<String, Value>,
    phones: Map<String, Value>,
    preferred_languages: Map<String, Value>,
    related_to: Map<String, Value>,
    scheduling_addresses: Map<String, Value>,
    titles: Map<String, Value>,
    /// The JSPROP payloads to graft onto the finished Card at their JSON
    /// pointers, each with its jCard fallback should the pointer not apply.
    js_props: Vec<(String, Value, Value)>,
    vcard_props: Vec<Value>,
}

/// A field selector, so a shared conversion can name the Card member it
/// fills without borrowing the whole builder.
type Member = fn(&mut Card) -> &mut Option<Value>;

/// A collection selector, same purpose for the id-keyed maps.
type Collection = fn(&mut Card) -> &mut Map<String, Value>;

impl Card {
    /// Convert one property, escaping it whole when it does not map.
    pub(super) fn prop(&mut self, prop: &VcardProp<'_>) {
        let VcardPropName::Kind(kind) = &prop.name else {
            return self.escape(prop);
        };

        match kind {
            VcardPropKind::Uid => self.singleton(prop, |card| &mut card.uid),
            VcardPropKind::ProdId => self.singleton(prop, |card| &mut card.prod_id),
            VcardPropKind::Language => self.singleton(prop, |card| &mut card.language),
            VcardPropKind::Rev => self.utc(prop, |card| &mut card.updated),
            VcardPropKind::Created => self.utc(prop, |card| &mut card.created),
            VcardPropKind::Kind => self.lowercase_singleton(prop, |card| &mut card.kind),
            VcardPropKind::GramGender => {
                self.lowercase_singleton(prop, |card| &mut card.gram_gender)
            }
            VcardPropKind::Pronouns => self.pronouns(prop),
            VcardPropKind::SocialProfile => self.social_profile(prop),
            VcardPropKind::JsProp => self.js_prop(prop),
            VcardPropKind::Fn => self.full_name(prop),
            VcardPropKind::N => self.n(prop),
            VcardPropKind::Nickname => self.nickname(prop),
            VcardPropKind::Adr => self.adr(prop),
            VcardPropKind::Email => {
                self.text_object(prop, |card| &mut card.emails, "EmailAddress", "address")
            }
            VcardPropKind::Tel => self.tel(prop),
            VcardPropKind::Impp => self.text_object(
                prop,
                |card| &mut card.online_services,
                "OnlineService",
                "uri",
            ),
            VcardPropKind::Lang => self.text_object(
                prop,
                |card| &mut card.preferred_languages,
                "LanguagePref",
                "language",
            ),
            VcardPropKind::Org => self.org(prop),
            VcardPropKind::Title => self.title(prop, "title"),
            VcardPropKind::Role => self.title(prop, "role"),
            VcardPropKind::Bday => self.anniversary(prop, "birth"),
            VcardPropKind::Anniversary => self.anniversary(prop, "wedding"),
            VcardPropKind::Photo => {
                self.resource(prop, |card| &mut card.media, "Media", Some("photo"))
            }
            VcardPropKind::Logo => {
                self.resource(prop, |card| &mut card.media, "Media", Some("logo"))
            }
            VcardPropKind::Sound => {
                self.resource(prop, |card| &mut card.media, "Media", Some("sound"))
            }
            VcardPropKind::Key => {
                self.resource(prop, |card| &mut card.crypto_keys, "CryptoKey", None)
            }
            VcardPropKind::CalUri => self.resource(
                prop,
                |card| &mut card.calendars,
                "Calendar",
                Some("calendar"),
            ),
            VcardPropKind::FbUrl => self.resource(
                prop,
                |card| &mut card.calendars,
                "Calendar",
                Some("freeBusy"),
            ),
            VcardPropKind::CalAdrUri => self.resource(
                prop,
                |card| &mut card.scheduling_addresses,
                "SchedulingAddress",
                None,
            ),
            VcardPropKind::Url => self.resource(prop, |card| &mut card.links, "Link", None),
            VcardPropKind::Source => self.resource(
                prop,
                |card| &mut card.directories,
                "Directory",
                Some("entry"),
            ),
            VcardPropKind::Categories => self.categories(prop),
            VcardPropKind::Note => self.text_object(prop, |card| &mut card.notes, "Note", "note"),
            VcardPropKind::Member => self.member(prop),
            VcardPropKind::Related => self.related(prop),

            // NOTE: no JSContact counterpart: GENDER (speakToAs comes from
            // the RFC 9554 properties, not from sex codes), the standalone
            // GEO / TZ / XML, and the 2.1 / 3.0 legacy set.
            VcardPropKind::Agent
            | VcardPropKind::Class
            | VcardPropKind::ClientPidMap
            | VcardPropKind::Gender
            | VcardPropKind::Geo
            | VcardPropKind::Label
            | VcardPropKind::Mailer
            | VcardPropKind::Name
            | VcardPropKind::Profile
            | VcardPropKind::SortString
            | VcardPropKind::Tz
            | VcardPropKind::Xml => self.escape(prop),
        }
    }

    /// A single-valued text member (uid, prodId): the first paramless
    /// instance wins, anything else is escaped.
    fn singleton(&mut self, prop: &VcardProp<'_>, member: Member) {
        let text = text(&prop.value).filter(|_| prop.params.is_empty());
        match text {
            Some(text) if member(self).is_none() => {
                *member(self) = Some(Value::String(text.into_owned()));
            }
            _ => self.escape(prop),
        }
    }

    /// A single-valued UTC-timestamp member (updated, created); anything
    /// short of a complete Zulu timestamp is escaped.
    fn utc(&mut self, prop: &VcardProp<'_>, member: Member) {
        let utc = match &prop.value {
            VcardValue::Timestamp(timestamp) => utc_timestamp(&timestamp.0),
            _ => None,
        }
        .filter(|_| prop.params.is_empty() && member(self).is_none());
        match utc {
            Some(utc) => *member(self) = Some(Value::String(utc)),
            None => self.escape(prop),
        }
    }

    /// A single-valued lowercased text member (kind, grammaticalGender).
    fn lowercase_singleton(&mut self, prop: &VcardProp<'_>, member: Member) {
        let value = text(&prop.value).filter(|_| prop.params.is_empty() && member(self).is_none());
        match value {
            Some(value) => *member(self) = Some(Value::String(value.to_ascii_lowercase())),
            None => self.escape(prop),
        }
    }

    /// PRONOUNS as `speakToAs.pronouns`: one Pronouns object per property.
    fn pronouns(&mut self, prop: &VcardProp<'_>) {
        let Some(value) = text(&prop.value) else {
            return self.escape(prop);
        };

        let mut params = ConvertedParams::split(&prop.params, false);
        let prop_id = params.take_prop_id();
        let mut object = object("Pronouns", "pronouns", &value);
        params.finish(&mut object);
        self.insert(|card| &mut card.pronouns, prop_id, object);
    }

    /// SOCIALPROFILE as `onlineServices`: a URI value is `uri`, a text value
    /// is `user`; SERVICE-TYPE is `service` and USERNAME is `user`.
    fn social_profile(&mut self, prop: &VcardProp<'_>) {
        let (member, value) = match &prop.value {
            VcardValue::Uri(uri) => ("uri", uri.0.as_ref()),
            VcardValue::Text(text) => ("user", text.0.as_ref()),
            _ => return self.escape(prop),
        };

        let mut params = ConvertedParams::split(&prop.params, false);
        let prop_id = params.take_prop_id();
        let mut object = object("OnlineService", member, value);
        if let Some(service) = params.take_service_type() {
            object.insert("service".into(), Value::String(service.to_string()));
        }
        if let Some(user) = params.take_username() {
            object
                .entry("user")
                .or_insert(Value::String(user.to_string()));
        }
        params.finish(&mut object);
        self.insert(|card| &mut card.online_services, prop_id, object);
    }

    /// JSPROP back to the Card member its JSPTR points at, carrying its
    /// jCard fallback should the pointer fail to apply at assembly time.
    fn js_prop(&mut self, prop: &VcardProp<'_>) {
        let pointer = prop.params.iter().find_map(|param| match param {
            VcardParam::Jsptr(pointer) => Some(pointer.as_ref()),
            _ => None,
        });
        let only_jsptr = prop
            .params
            .iter()
            .all(|param| matches!(param, VcardParam::Jsptr(_)));
        let value = match &prop.value {
            VcardValue::Text(text) => serde_json::from_str::<Value>(&text.0).ok(),
            _ => None,
        };

        match (pointer, value) {
            (Some(pointer), Some(value)) if only_jsptr && pointer.starts_with('/') => {
                self.js_props
                    .push((pointer.to_string(), value, prop.to_jcard()));
            }
            _ => self.escape(prop),
        }
    }

    /// FN as `name.full`: the first instance wins, a repeat is escaped.
    fn full_name(&mut self, prop: &VcardProp<'_>) {
        let Some(full) = text(&prop.value).filter(|_| !self.name.contains_key("full")) else {
            return self.escape(prop);
        };

        let params = ConvertedParams::split(&prop.params, false);
        self.name
            .insert("full".into(), Value::String(full.into_owned()));
        params.finish(&mut self.name);
    }

    /// N as `name.components` (a multi-valued slot yields one component per
    /// value), with SORT-AS as `name.sortAs`.
    fn n(&mut self, prop: &VcardProp<'_>) {
        let VcardValue::N(n) = &prop.value else {
            return self.escape(prop);
        };
        if self.name.contains_key("components") {
            return self.escape(prop);
        }

        let slots = [
            ("surname", &n.family),
            ("given", &n.given),
            ("given2", &n.additional),
            ("title", &n.prefixes),
            ("credential", &n.suffixes),
        ];
        let components = components(&slots, "NameComponent");

        let mut params = ConvertedParams::split(&prop.params, false);
        if let Some(sort_as) = params.take_sort_as() {
            let mut map = Map::new();
            for (kind, value) in ["surname", "given"].into_iter().zip(sort_as) {
                map.insert(kind.into(), Value::String(value.to_string()));
            }
            self.name.insert("sortAs".into(), Value::Object(map));
        }

        self.name
            .insert("components".into(), Value::Array(components));
        params.finish(&mut self.name);
    }

    /// NICKNAME as `nicknames`: one Nickname object per list item, each
    /// carrying the property's contexts and pref.
    fn nickname(&mut self, prop: &VcardProp<'_>) {
        let VcardValue::TextList(list) = &prop.value else {
            return self.escape(prop);
        };

        for name in &list.0 {
            let mut params = ConvertedParams::split(&prop.params, false);
            let prop_id = params.take_prop_id();
            let mut object = object("Nickname", "name", name);
            params.finish(&mut object);
            self.insert(|card| &mut card.nicknames, prop_id, object);
        }
    }

    /// ADR as `addresses`: components from the RFC 6350 and 9554 slots,
    /// LABEL as `full`, the GEO / TZ parameters as `coordinates` /
    /// `timeZone`.
    fn adr(&mut self, prop: &VcardProp<'_>) {
        let VcardValue::Adr(adr) = &prop.value else {
            return self.escape(prop);
        };

        // NOTE: the legacy extended-address / street slots and the RFC 9554
        // apartment / street-name ones share a JSContact component kind; a
        // card filling both sides of a pair cannot be represented, so it is
        // escaped whole.
        let non_empty = |values: &[Cow<'_, str>]| values.iter().any(|value| !value.is_empty());
        if (non_empty(&adr.extended) && non_empty(&adr.apartment))
            || (non_empty(&adr.street) && non_empty(&adr.street_name))
        {
            return self.escape(prop);
        }
        let apartment = if non_empty(&adr.apartment) {
            &adr.apartment
        } else {
            &adr.extended
        };
        let street_name = if non_empty(&adr.street_name) {
            &adr.street_name
        } else {
            &adr.street
        };

        let slots = [
            ("postOfficeBox", &adr.po_box),
            ("apartment", apartment),
            ("name", street_name),
            ("locality", &adr.locality),
            ("region", &adr.region),
            ("postcode", &adr.postal_code),
            ("country", &adr.country),
            ("room", &adr.room),
            ("floor", &adr.floor),
            ("number", &adr.street_number),
            ("building", &adr.building),
            ("block", &adr.block),
            ("subdistrict", &adr.subdistrict),
            ("district", &adr.district),
            ("landmark", &adr.landmark),
            ("direction", &adr.direction),
        ];
        let components = components(&slots, "AddressComponent");

        let mut params = ConvertedParams::split(&prop.params, false);
        let prop_id = params.take_prop_id();
        let mut object = Map::new();
        object.insert("@type".into(), "Address".into());
        if !components.is_empty() {
            object.insert("components".into(), Value::Array(components));
        }
        if let Some(full) = params.take_label() {
            object.insert("full".into(), Value::String(full.to_string()));
        }
        if let Some(coordinates) = params.take_geo() {
            object.insert("coordinates".into(), Value::String(coordinates.to_string()));
        }
        if let Some(time_zone) = params.take_tz() {
            object.insert("timeZone".into(), Value::String(time_zone.to_string()));
        }
        params.finish(&mut object);
        self.insert(|card| &mut card.addresses, prop_id, object);
    }

    /// TEL as `phones`, with the TYPE feature values as `features`.
    fn tel(&mut self, prop: &VcardProp<'_>) {
        let Some(number) = text(&prop.value) else {
            return self.escape(prop);
        };

        let mut params = ConvertedParams::split(&prop.params, true);
        let prop_id = params.take_prop_id();
        let mut object = object("Phone", "number", &number);
        params.finish(&mut object);
        self.insert(|card| &mut card.phones, prop_id, object);
    }

    /// ORG as `organizations`: the first unit is the name, the rest are
    /// units, SORT-AS is `sortAs`.
    fn org(&mut self, prop: &VcardProp<'_>) {
        let VcardValue::Org(org) = &prop.value else {
            return self.escape(prop);
        };
        let Some((name, units)) = org.0.split_first() else {
            return self.escape(prop);
        };

        let mut params = ConvertedParams::split(&prop.params, false);
        let prop_id = params.take_prop_id();
        let mut object = object("Organization", "name", name);
        if !units.is_empty() {
            let units: Vec<Value> = units
                .iter()
                .map(|unit| json!({ "@type": "OrgUnit", "name": unit }))
                .collect();
            object.insert("units".into(), Value::Array(units));
        }
        if let Some(first) = params.take_sort_as().and_then(<[_]>::first) {
            object.insert("sortAs".into(), Value::String(first.to_string()));
        }
        params.finish(&mut object);
        self.insert(|card| &mut card.organizations, prop_id, object);
    }

    /// TITLE / ROLE as `titles`, told apart by `kind`.
    fn title(&mut self, prop: &VcardProp<'_>, kind: &str) {
        let Some(name) = text(&prop.value) else {
            return self.escape(prop);
        };

        let mut params = ConvertedParams::split(&prop.params, false);
        let prop_id = params.take_prop_id();
        let mut object = object("Title", "name", &name);
        object.insert("kind".into(), Value::String(kind.to_string()));
        params.finish(&mut object);
        self.insert(|card| &mut card.titles, prop_id, object);
    }

    /// BDAY / ANNIVERSARY as `anniversaries`, told apart by `kind`; a value
    /// that is neither a date nor a UTC timestamp is escaped.
    fn anniversary(&mut self, prop: &VcardProp<'_>, kind: &str) {
        let date = match &prop.value {
            VcardValue::DateAndOrTime(date) => date_object(&date.0),
            _ => None,
        };
        let Some(date) = date else {
            return self.escape(prop);
        };

        let mut params = ConvertedParams::split(&prop.params, false);
        let prop_id = params.take_prop_id();
        let mut object = Map::new();
        object.insert("@type".into(), "Anniversary".into());
        object.insert("kind".into(), Value::String(kind.to_string()));
        object.insert("date".into(), date);
        params.finish(&mut object);
        self.insert(|card| &mut card.anniversaries, prop_id, object);
    }

    /// A URI-valued resource property (PHOTO, KEY, URL, ...) as its
    /// collection's object, tagged with the type name RFC 9553 §2.6
    /// registers for that collection; an inline 2.1 / 3.0 binary payload
    /// has no URI to point at and is escaped.
    fn resource(
        &mut self,
        prop: &VcardProp<'_>,
        collection: Collection,
        r#type: &str,
        kind: Option<&str>,
    ) {
        let uri = match &prop.value {
            VcardValue::Binary(VcardBinary::Uri(uri)) => Some(Cow::Borrowed(uri.as_ref())),
            value => text(value),
        };
        let Some(uri) = uri else {
            return self.escape(prop);
        };

        let mut params = ConvertedParams::split(&prop.params, false);
        let prop_id = params.take_prop_id();
        let mut object = object(r#type, "uri", &uri);
        if let Some(kind) = kind {
            object.insert("kind".into(), Value::String(kind.to_string()));
        }
        if let Some(media_type) = params.take_media_type() {
            object.insert("mediaType".into(), Value::String(media_type.to_string()));
        }
        params.finish(&mut object);
        self.insert(collection, prop_id, object);
    }

    /// CATEGORIES as the `keywords` boolean map; the map cannot carry
    /// parameters, so a parameterized property is escaped whole.
    fn categories(&mut self, prop: &VcardProp<'_>) {
        let VcardValue::TextList(list) = &prop.value else {
            return self.escape(prop);
        };
        if !prop.params.is_empty() {
            return self.escape(prop);
        }

        for keyword in &list.0 {
            self.keywords.insert(keyword.to_string(), Value::Bool(true));
        }
    }

    /// MEMBER as the `members` boolean map; same paramless rule as keywords.
    fn member(&mut self, prop: &VcardProp<'_>) {
        let uri = text(&prop.value).filter(|_| prop.params.is_empty());
        match uri {
            Some(uri) => {
                self.members.insert(uri.into_owned(), Value::Bool(true));
            }
            None => self.escape(prop),
        }
    }

    /// RELATED as `relatedTo`, keyed by the target, with the TYPE values as
    /// the relation set; any other parameter escapes the property whole.
    fn related(&mut self, prop: &VcardProp<'_>) {
        let only_types = prop
            .params
            .iter()
            .all(|param| matches!(param, VcardParam::Type(_)));
        let Some(target) = text(&prop.value).filter(|_| only_types) else {
            return self.escape(prop);
        };

        let mut relation = Map::new();
        for param in &prop.params {
            if let VcardParam::Type(values) = param {
                for value in values.iter().filter(|value| !value.is_empty()) {
                    relation.insert(value.to_ascii_lowercase(), Value::Bool(true));
                }
            }
        }

        let mut object = Map::new();
        object.insert("@type".into(), "Relation".into());
        if !relation.is_empty() {
            object.insert("relation".into(), Value::Object(relation));
        }
        self.related_to
            .insert(target.into_owned(), Value::Object(object));
    }

    /// A one-text-member object in a collection (emails, notes, ...).
    fn text_object(
        &mut self,
        prop: &VcardProp<'_>,
        collection: Collection,
        r#type: &str,
        member: &str,
    ) {
        let Some(value) = text(&prop.value) else {
            return self.escape(prop);
        };

        let mut params = ConvertedParams::split(&prop.params, false);
        let prop_id = params.take_prop_id();
        let mut object = object(r#type, member, &value);
        params.finish(&mut object);
        self.insert(collection, prop_id, object);
    }

    /// Insert an object into a collection under its PROP-ID, or under its
    /// 1-based source order when there is none (or it collides).
    fn insert(
        &mut self,
        collection: Collection,
        prop_id: Option<String>,
        object: Map<String, Value>,
    ) {
        let collection = collection(self);

        let key = prop_id
            .filter(|id| !id.is_empty() && !collection.contains_key(id))
            .unwrap_or_else(|| {
                let mut index = collection.len() + 1;
                while collection.contains_key(&index.to_string()) {
                    index += 1;
                }
                index.to_string()
            });

        collection.insert(key, Value::Object(object));
    }

    /// Preserve a property whole in vCardProps, in jCard syntax.
    fn escape(&mut self, prop: &VcardProp<'_>) {
        self.vcard_props.push(prop.to_jcard());
    }

    /// The finished Card object, with only its non-empty members.
    pub(super) fn into_value(mut self) -> Value {
        let mut card = Map::new();
        card.insert("@type".into(), "Card".into());
        card.insert("version".into(), "1.0".into());

        if !self.name.is_empty() {
            self.name.insert("@type".into(), "Name".into());
            card.insert("name".into(), Value::Object(self.name));
        }

        let singletons = [
            ("uid", self.uid),
            ("prodId", self.prod_id),
            ("created", self.created),
            ("updated", self.updated),
            ("kind", self.kind),
            ("language", self.language),
        ];
        for (member, value) in singletons {
            if let Some(value) = value {
                card.insert(member.into(), value);
            }
        }

        if self.gram_gender.is_some() || !self.pronouns.is_empty() {
            let mut speak_to_as = Map::new();
            speak_to_as.insert("@type".into(), "SpeakToAs".into());
            if let Some(gram_gender) = self.gram_gender {
                speak_to_as.insert("grammaticalGender".into(), gram_gender);
            }
            if !self.pronouns.is_empty() {
                speak_to_as.insert("pronouns".into(), Value::Object(self.pronouns));
            }
            card.insert("speakToAs".into(), Value::Object(speak_to_as));
        }

        let collections = [
            ("addresses", self.addresses),
            ("anniversaries", self.anniversaries),
            ("calendars", self.calendars),
            ("cryptoKeys", self.crypto_keys),
            ("directories", self.directories),
            ("emails", self.emails),
            ("keywords", self.keywords),
            ("links", self.links),
            ("media", self.media),
            ("members", self.members),
            ("nicknames", self.nicknames),
            ("notes", self.notes),
            ("onlineServices", self.online_services),
            ("organizations", self.organizations),
            ("phones", self.phones),
            ("preferredLanguages", self.preferred_languages),
            ("relatedTo", self.related_to),
            ("schedulingAddresses", self.scheduling_addresses),
            ("titles", self.titles),
        ];
        for (member, collection) in collections {
            if !collection.is_empty() {
                card.insert(member.into(), Value::Object(collection));
            }
        }

        for (pointer, value, fallback) in self.js_props {
            if !set_pointer(&mut card, &pointer, value) {
                self.vcard_props.push(fallback);
            }
        }

        if !self.vcard_props.is_empty() {
            card.insert("vCardProps".into(), Value::Array(self.vcard_props));
        }

        Value::Object(card)
    }
}

/// The component objects of a structured value: one per non-empty slot
/// value, in slot order.
fn components(slots: &[(&str, &Vec<Cow<'_, str>>)], r#type: &str) -> Vec<Value> {
    let mut components = Vec::new();

    for (kind, values) in slots {
        for value in values.iter().filter(|value| !value.is_empty()) {
            components.push(json!({
                "@type": r#type,
                "kind": kind,
                "value": value,
            }));
        }
    }

    components
}

/// A fresh object with its @type and one text member.
fn object(r#type: &str, member: &str, value: &str) -> Map<String, Value> {
    let mut object = Map::new();
    object.insert("@type".into(), Value::String(r#type.to_string()));
    object.insert(member.to_string(), Value::String(value.to_string()));
    object
}

/// The property value as one text, for the value shapes that carry one.
fn text<'a>(value: &'a VcardValue<'_>) -> Option<Cow<'a, str>> {
    match value {
        VcardValue::Text(text) => Some(Cow::Borrowed(text.0.as_ref())),
        VcardValue::Uri(uri) => Some(Cow::Borrowed(uri.0.as_ref())),
        VcardValue::LanguageTag(tag) => Some(Cow::Borrowed(tag.0.as_ref())),
        _ => None,
    }
}
