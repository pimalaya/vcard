//! # JSContact
//!
//! The RFC 9555 conversion: the decoded card as an RFC 9553 JSContact Card, and
//! back.
//!
//! [`Vcard::to_jscontact`] writes the decoded model as the JSON Card object
//! JMAP for Contacts (RFC 9610) exchanges; [`Vcard::from_jscontact`] reads one
//! back, borrowing the JSON tree's strings.
//!
//! There is no JSContact model in this crate: the Card is a plain
//! [`serde_json::Value`], and vCard stays the one decoded model.
//!
//! Both directions are lossless through the RFC 9555 escape hatches, and only
//! a non-object root can fail the import.
//!
//! Exporting, a property with no JSContact counterpart (or whose value cannot
//! be represented, like a free-text birthday) is preserved whole in the Card's
//! `vCardProps` member, in jCard syntax through the sibling [`crate::jcard`]
//! codec; a leftover parameter goes to the object's `vCardParams` member.
//!
//! Importing, the mirror hatch applies: a Card member (or nested piece) with no
//! vCard counterpart becomes a `JSPROP` property holding its JSON, located by a
//! `JSPTR` parameter that the export grafts back onto the Card.
//!
//! A `PROP-ID` parameter carries each object's map key across conversions,
//! which is what keeps JMAP patch identity stable; without one, keys are the
//! 1-based source order.
//!
//! Mapped properties: UID, PRODID, REV, KIND, FN and N (with SORT-AS),
//! NICKNAME, ADR (all eighteen components, with LABEL, GEO, TZ), EMAIL, TEL,
//! IMPP, LANG, ORG, TITLE, ROLE, BDAY, ANNIVERSARY, PHOTO, LOGO, SOUND, KEY,
//! CALURI, FBURL, CALADRURI, URL, SOURCE, CATEGORIES, NOTE, MEMBER, RELATED.
//!
//! The RFC 9554 set follows: CREATED, LANGUAGE, GRAMGENDER, PRONOUNS,
//! SOCIALPROFILE and the JSPROP carrier. The TYPE parameter maps to contexts
//! (`home` becomes `private`) and, on TEL, to features (`cell` becomes
//! `mobile`); PREF maps to `pref`. Everything else rides the escape hatches.

use core::{error, fmt, mem};

use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use serde_json::{Map, Value, json};

use crate::{
    jcard::{
        basic_to_extended, extended_to_basic, merge_param, param_from_jcard, param_to_jcard,
        prop_from_jcard, prop_to_jcard,
    },
    param::VcardParam,
    prop::{VcardProp, VcardPropKind, VcardPropName},
    value::{
        VcardValue,
        adr::VcardAdr,
        binary::VcardBinary,
        datetime::{VcardDateAndOrTime, VcardTimestamp},
        language::VcardLanguageTag,
        n::VcardN,
        org::VcardOrg,
        text::{VcardText, VcardTextList},
        uri::VcardUri,
    },
    vcard::Vcard,
    version::VcardVersion,
};

/// Parse JSContact error.
#[derive(Debug)]
pub struct VcardJscontactParseError;

impl fmt::Display for VcardJscontactParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cannot parse JSContact card: the root is not an object")
    }
}

impl error::Error for VcardJscontactParseError {}

impl Vcard<'_> {
    /// Convert the card into an RFC 9553 JSContact Card [`Value`], following
    /// the RFC 9555 mapping.
    ///
    /// Infallible: a property or parameter with no JSContact counterpart is
    /// preserved in the vCardProps / vCardParams escape hatches.
    pub fn to_jscontact(&self) -> Value {
        let mut card = Card::default();

        for prop in &self.properties {
            card.prop(prop);
        }

        card.into_value()
    }

    /// Convert an RFC 9553 JSContact Card [`Value`] into a decoded card,
    /// following the RFC 9555 mapping and borrowing the JSON tree's strings.
    ///
    /// Liberal: only a non-object root errors; a member (or nested piece)
    /// with no vCard counterpart is preserved as a JSPROP property.
    pub fn from_jscontact(jscontact: &Value) -> Result<Vcard<'_>, VcardJscontactParseError> {
        let card = jscontact.as_object().ok_or(VcardJscontactParseError)?;
        let mut import = Import::default();

        for (member, value) in card {
            import.member(member, value);
        }

        Ok(Vcard {
            version: VcardVersion::V4_0,
            properties: import.properties,
        })
    }
}

/// The Card under construction: one field per JSContact member, filled
/// property by property, plus the vCardProps escape hatch.
#[derive(Default)]
struct Card {
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
    fn prop(&mut self, prop: &VcardProp<'_>) {
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
        let prop_id = params.prop_id.take();
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
        let prop_id = params.prop_id.take();
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
                    .push((pointer.to_string(), value, prop_to_jcard(prop)));
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
            let prop_id = params.prop_id.take();
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
        let prop_id = params.prop_id.take();
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
        let prop_id = params.prop_id.take();
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
        let prop_id = params.prop_id.take();
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
        let prop_id = params.prop_id.take();
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
        let prop_id = params.prop_id.take();
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
        let prop_id = params.prop_id.take();
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
        let prop_id = params.prop_id.take();
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
        self.vcard_props.push(prop_to_jcard(prop));
    }

    /// The finished Card object, with only its non-empty members.
    fn into_value(mut self) -> Value {
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

/// Graft a value onto the Card at a JSON pointer, creating intermediate
/// objects; `false` when a step lands on a non-object.
fn set_pointer(root: &mut Map<String, Value>, pointer: &str, value: Value) -> bool {
    let Some(path) = pointer.strip_prefix('/') else {
        return false;
    };

    let mut segments = path.split('/').map(unescape_pointer);
    let mut key = match segments.next() {
        Some(key) => key,
        None => return false,
    };

    let mut current = root;
    for next in segments {
        let entry = current
            .entry(key)
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(object) = entry.as_object_mut() else {
            return false;
        };
        current = object;
        key = next;
    }

    current.insert(key, value);
    true
}

/// Undo the RFC 6901 escapes of one pointer segment.
fn unescape_pointer(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}

/// Apply the RFC 6901 escapes to one pointer segment.
fn escape_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

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
        .map(|(key, value)| param_from_jcard(key, value))
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

/// A JSContact Timestamp or PartialDate back to the RFC 6350 basic format,
/// `None` when it fits neither.
fn date_from_jscontact(value: &Value) -> Option<String> {
    let object = value.as_object()?;
    let known = |allowed: &[&str]| {
        object
            .keys()
            .all(|member| member == "@type" || allowed.contains(&member.as_str()))
    };

    if let Some(utc) = object.get("utc").and_then(Value::as_str) {
        return known(&["utc"]).then(|| extended_to_basic(Cow::Borrowed(utc)).into_owned());
    }

    if !known(&["year", "month", "day"]) {
        return None;
    }
    let part = |member: &str| object.get(member).and_then(Value::as_u64);
    match (part("year"), part("month"), part("day")) {
        (Some(year), Some(month), Some(day)) => Some(format!("{year:04}{month:02}{day:02}")),
        (Some(year), Some(month), None) => Some(format!("{year:04}-{month:02}")),
        (Some(year), None, None) => Some(format!("{year:04}")),
        (None, Some(month), Some(day)) => Some(format!("--{month:02}{day:02}")),
        (None, Some(month), None) => Some(format!("--{month:02}")),
        (None, None, Some(day)) => Some(format!("---{day:02}")),
        _ => None,
    }
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
struct Import<'a> {
    properties: Vec<VcardProp<'a>>,
}

impl<'a> Import<'a> {
    /// Convert one Card member, preserving what does not map as JSPROP.
    fn member(&mut self, member: &'a str, value: &'a Value) {
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
                Some(prop_from_jcard(
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

/// The parameter split shared by every converted object: contexts and
/// features from TYPE, `pref` from PREF, the map key from PROP-ID, the
/// consumable specials (LABEL, MEDIATYPE, SORT-AS, GEO, TZ), and the
/// unconverted rest bound for vCardParams.
struct ConvertedParams<'a> {
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
    fn split(params: &'a [VcardParam<'a>], phone: bool) -> Self {
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
    fn take_label(&mut self) -> Option<&'a str> {
        match self.label.take() {
            Some(VcardParam::Label(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Consume the MEDIATYPE parameter's text.
    fn take_media_type(&mut self) -> Option<&'a str> {
        match self.media_type.take() {
            Some(VcardParam::MediaType(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Consume the SORT-AS parameter's values.
    fn take_sort_as(&mut self) -> Option<&'a [Cow<'a, str>]> {
        match self.sort_as.take() {
            Some(VcardParam::SortAs(values)) => Some(values.as_slice()),
            _ => None,
        }
    }

    /// Consume the GEO parameter's text.
    fn take_geo(&mut self) -> Option<&'a str> {
        match self.geo.take() {
            Some(VcardParam::Geo(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Consume the TZ parameter's text.
    fn take_tz(&mut self) -> Option<&'a str> {
        match self.tz.take() {
            Some(VcardParam::Tz(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Consume the SERVICE-TYPE parameter's text.
    fn take_service_type(&mut self) -> Option<&'a str> {
        match self.service_type.take() {
            Some(VcardParam::ServiceType(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Consume the USERNAME parameter's text.
    fn take_username(&mut self) -> Option<&'a str> {
        match self.username.take() {
            Some(VcardParam::Username(value)) => Some(value.as_ref()),
            _ => None,
        }
    }

    /// Close the object: contexts, features and pref where present, and
    /// everything unconsumed as the vCardParams escape hatch, in jCard
    /// parameter syntax.
    fn finish(self, object: &mut Map<String, Value>) {
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
            let (key, value) = param_to_jcard(param);
            merge_param(&mut escaped, key, value);
        }
        if !escaped.is_empty() {
            object.insert("vCardParams".into(), Value::Object(escaped));
        }
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

/// An anniversary date as a JSContact Timestamp (a complete UTC date-time)
/// or PartialDate (a possibly reduced or truncated date), `None` when the
/// value fits neither.
fn date_object(raw: &str) -> Option<Value> {
    if raw.contains('T') {
        let utc = utc_timestamp(raw)?;
        return Some(json!({ "@type": "Timestamp", "utc": utc }));
    }

    let (year, month, day) = partial_date(raw)?;
    let mut object = Map::new();
    object.insert("@type".into(), "PartialDate".into());
    for (member, part) in [("year", year), ("month", month), ("day", day)] {
        if let Some(part) = part {
            object.insert(member.into(), Value::from(part));
        }
    }
    Some(Value::Object(object))
}

/// A complete Zulu date-time re-spelled extended, `None` for anything short
/// of one (a floating or offset time cannot be a JSContact UTC timestamp).
fn utc_timestamp(raw: &str) -> Option<String> {
    let (date, time) = raw.split_once('T')?;
    let time = time.strip_suffix('Z')?;

    let date_digits = date.bytes().filter(u8::is_ascii_digit).count();
    let time_digits = time.bytes().filter(u8::is_ascii_digit).count();
    let punctuated = date.bytes().all(|b| b.is_ascii_digit() || b == b'-')
        && time.bytes().all(|b| b.is_ascii_digit() || b == b':');
    if !punctuated || date_digits != 8 || time_digits != 6 {
        return None;
    }

    Some(basic_to_extended(raw))
}

/// The year / month / day parts of a complete, reduced or truncated RFC 6350
/// date, in its basic or extended spelling; `None` when it fits no shape.
#[allow(clippy::type_complexity)]
fn partial_date(raw: &str) -> Option<(Option<u64>, Option<u64>, Option<u64>)> {
    let part = |digits: &str| digits.parse::<u64>().ok();

    if let Some(day) = raw.strip_prefix("---") {
        return Some((None, None, Some(part(day)?)));
    }

    if let Some(rest) = raw.strip_prefix("--") {
        return match rest.len() {
            2 => Some((None, Some(part(rest)?), None)),
            4 => Some((None, Some(part(&rest[..2])?), Some(part(&rest[2..])?))),
            5 if rest.as_bytes()[2] == b'-' => {
                Some((None, Some(part(&rest[..2])?), Some(part(&rest[3..])?)))
            }
            _ => None,
        };
    }

    match raw.len() {
        4 => Some((Some(part(raw)?), None, None)),
        7 if raw.as_bytes()[4] == b'-' => {
            Some((Some(part(&raw[..4])?), Some(part(&raw[5..])?), None))
        }
        8 => Some((
            Some(part(&raw[..4])?),
            Some(part(&raw[4..6])?),
            Some(part(&raw[6..])?),
        )),
        10 if raw.as_bytes()[4] == b'-' && raw.as_bytes()[7] == b'-' => Some((
            Some(part(&raw[..4])?),
            Some(part(&raw[5..7])?),
            Some(part(&raw[8..])?),
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, vec, vec::Vec};

    use serde_json::json;

    use crate::{prop::VcardPropName, tree::cst::VcardCst, vcard::Vcard};

    #[test]
    fn exports_a_minimal_card() {
        let cst =
            VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nEND:VCARD\r\n").unwrap();

        assert_eq!(
            cst.decode().to_jscontact(),
            json!({
                "@type": "Card",
                "version": "1.0",
                "name": { "@type": "Name", "full": "John Doe" },
            }),
        );
    }

    #[test]
    fn exports_a_full_card() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "UID:urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1\r\n",
            "FN:Simon Perreault\r\n",
            "N:Perreault;Simon;;;ing. jr,M.Sc.\r\n",
            "ORG;SORT-AS=Viagenie:Viagenie;IT\r\n",
            "TITLE:Director\r\n",
            "EMAIL;TYPE=work;PREF=1:simon@example.com\r\n",
            "TEL;TYPE=work,cell,voice:tel:+1-418-262-6501\r\n",
            "ADR;TYPE=home;LABEL=The full label:;;2875 boul. Laurier;Quebec;QC;G1V 2M2;Canada\r\n",
            "LANG;PREF=2:fr\r\n",
            "BDAY:--0203\r\n",
            "ANNIVERSARY:20090808T143000Z\r\n",
            "CATEGORIES:developer,ietf\r\n",
            "NOTE:Hello\r\n",
            "URL:https://example.com\r\n",
            "KEY;MEDIATYPE=application/pgp-keys:https://example.com/key.asc\r\n",
            "REV:20240101T000000Z\r\n",
            "GENDER:M\r\n",
            "X-FOO;X-BAR=1:baz\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();

        assert_eq!(
            cst.decode().to_jscontact(),
            json!({
                "@type": "Card",
                "version": "1.0",
                "uid": "urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1",
                "updated": "2024-01-01T00:00:00Z",
                "name": {
                    "@type": "Name",
                    "full": "Simon Perreault",
                    "components": [
                        { "@type": "NameComponent", "kind": "surname", "value": "Perreault" },
                        { "@type": "NameComponent", "kind": "given", "value": "Simon" },
                        { "@type": "NameComponent", "kind": "credential", "value": "ing. jr" },
                        { "@type": "NameComponent", "kind": "credential", "value": "M.Sc." },
                    ],
                },
                "organizations": {
                    "1": {
                        "@type": "Organization",
                        "name": "Viagenie",
                        "units": [{ "@type": "OrgUnit", "name": "IT" }],
                        "sortAs": "Viagenie",
                    },
                },
                "titles": { "1": { "@type": "Title", "kind": "title", "name": "Director" } },
                "emails": {
                    "1": {
                        "@type": "EmailAddress",
                        "address": "simon@example.com",
                        "contexts": { "work": true },
                        "pref": 1,
                    },
                },
                "phones": {
                    "1": {
                        "@type": "Phone",
                        "number": "tel:+1-418-262-6501",
                        "contexts": { "work": true },
                        "features": { "mobile": true, "voice": true },
                    },
                },
                "addresses": {
                    "1": {
                        "@type": "Address",
                        "components": [
                            { "@type": "AddressComponent", "kind": "name", "value": "2875 boul. Laurier" },
                            { "@type": "AddressComponent", "kind": "locality", "value": "Quebec" },
                            { "@type": "AddressComponent", "kind": "region", "value": "QC" },
                            { "@type": "AddressComponent", "kind": "postcode", "value": "G1V 2M2" },
                            { "@type": "AddressComponent", "kind": "country", "value": "Canada" },
                        ],
                        "contexts": { "private": true },
                        "full": "The full label",
                    },
                },
                "preferredLanguages": {
                    "1": { "@type": "LanguagePref", "language": "fr", "pref": 2 },
                },
                "anniversaries": {
                    "1": {
                        "@type": "Anniversary",
                        "kind": "birth",
                        "date": { "@type": "PartialDate", "month": 2, "day": 3 },
                    },
                    "2": {
                        "@type": "Anniversary",
                        "kind": "wedding",
                        "date": { "@type": "Timestamp", "utc": "2009-08-08T14:30:00Z" },
                    },
                },
                "keywords": { "developer": true, "ietf": true },
                "notes": { "1": { "@type": "Note", "note": "Hello" } },
                "links": { "1": { "@type": "Link", "uri": "https://example.com" } },
                "cryptoKeys": {
                    "1": {
                        "@type": "CryptoKey",
                        "uri": "https://example.com/key.asc",
                        "mediaType": "application/pgp-keys",
                    },
                },
                "vCardProps": [
                    ["gender", {}, "text", "M"],
                    ["x-foo", { "x-bar": "1" }, "unknown", "baz"],
                ],
            }),
        );
    }

    #[test]
    fn tags_resource_objects_with_their_rfc_type_names() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John\r\n",
            "PHOTO:https://example.com/photo.png\r\n",
            "KEY:https://example.com/key.asc\r\n",
            "CALURI:https://example.com/cal.ics\r\n",
            "CALADRURI:mailto:john@example.com\r\n",
            "URL:https://example.com\r\n",
            "SOURCE:https://example.com/john.vcf\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode().to_jscontact();

        // NOTE: RFC 9553 2.6 registers these names; the pre-RFC drafts spelled
        // them `MediaResource`, `CryptoResource` and so on, and a strict
        // server (Fastmail) rejects the draft spelling outright.
        assert_eq!(card["media"]["1"]["@type"], json!("Media"));
        assert_eq!(card["cryptoKeys"]["1"]["@type"], json!("CryptoKey"));
        assert_eq!(card["calendars"]["1"]["@type"], json!("Calendar"));
        assert_eq!(
            card["schedulingAddresses"]["1"]["@type"],
            json!("SchedulingAddress")
        );
        assert_eq!(card["links"]["1"]["@type"], json!("Link"));
        assert_eq!(card["directories"]["1"]["@type"], json!("Directory"));
    }

    #[test]
    fn reads_back_a_resource_written_with_a_draft_type_name() {
        let card = json!({
            "@type": "Card",
            "version": "1.0",
            "links": { "1": { "@type": "LinkResource", "uri": "https://example.com" } },
        });
        let vcard = Vcard::from_jscontact(&card).unwrap();

        // NOTE: import ignores `@type`, so a Card written by an older version
        // still converts back to URL rather than falling into JSPROP.
        let names: Vec<&str> = vcard.properties.iter().map(|prop| &*prop.name).collect();
        assert_eq!(names, ["URL"]);
    }

    #[test]
    fn uses_prop_id_as_the_map_key() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John\r\n",
            "EMAIL;PROP-ID=e99:john@example.com\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode().to_jscontact();

        assert_eq!(card["emails"]["e99"]["address"], json!("john@example.com"),);
    }

    #[test]
    fn keeps_unconverted_params_in_vcard_params() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John\r\n",
            "EMAIL;ALTID=1;X-CUSTOM=y:john@example.com\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode().to_jscontact();

        assert_eq!(
            card["emails"]["1"]["vCardParams"],
            json!({ "altid": "1", "x-custom": "y" }),
        );
    }

    #[test]
    fn escapes_what_cannot_be_represented() {
        // NOTE: a free-text birthday fits neither Timestamp nor PartialDate,
        // a parameterized CATEGORIES cannot ride the keywords boolean map,
        // and a grouped property keeps its group in the escaped jCard entry.
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John\r\n",
            "BDAY;VALUE=text:circa 1800\r\n",
            "CATEGORIES;PREF=1:vip\r\n",
            "ITEM1.X-ABLABEL:Nickname\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode().to_jscontact();

        assert_eq!(
            card["vCardProps"],
            json!([
                ["bday", {}, "text", "circa 1800"],
                ["categories", { "pref": "1" }, "text", "vip"],
                ["x-ablabel", { "group": "item1" }, "unknown", "Nickname"],
            ]),
        );
        assert!(card.get("anniversaries").is_none());
        assert!(card.get("keywords").is_none());
    }

    #[test]
    fn maps_group_kind_members_and_relations() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "KIND:group\r\n",
            "FN:The Does\r\n",
            "MEMBER:urn:uuid:john\r\n",
            "MEMBER:urn:uuid:jane\r\n",
            "RELATED;TYPE=friend,met:urn:uuid:jimmy\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let card = cst.decode().to_jscontact();

        assert_eq!(card["kind"], json!("group"));
        assert_eq!(
            card["members"],
            json!({ "urn:uuid:john": true, "urn:uuid:jane": true }),
        );
        assert_eq!(
            card["relatedTo"],
            json!({
                "urn:uuid:jimmy": {
                    "@type": "Relation",
                    "relation": { "friend": true, "met": true },
                },
            }),
        );
    }

    #[test]
    fn exports_the_rfc_9554_props_first_class() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:John\r\n",
            "CREATED:20240101T000000Z\r\n",
            "LANGUAGE:fr\r\n",
            "GRAMGENDER:Masculine\r\n",
            "PRONOUNS;PREF=1;PROP-ID=p1:he/him\r\n",
            "SOCIALPROFILE;SERVICE-TYPE=Mastodon:https://example.social/@john\r\n",
            "JSPROP;JSPTR=/foo/bar:{\"baz\":42}\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();

        assert_eq!(
            cst.decode().to_jscontact(),
            json!({
                "@type": "Card",
                "version": "1.0",
                "created": "2024-01-01T00:00:00Z",
                "language": "fr",
                "name": { "@type": "Name", "full": "John" },
                "speakToAs": {
                    "@type": "SpeakToAs",
                    "grammaticalGender": "masculine",
                    "pronouns": {
                        "p1": { "@type": "Pronouns", "pronouns": "he/him", "pref": 1 },
                    },
                },
                "onlineServices": {
                    "1": {
                        "@type": "OnlineService",
                        "uri": "https://example.social/@john",
                        "service": "Mastodon",
                    },
                },
                "foo": { "bar": { "baz": 42 } },
            }),
        );
    }

    #[test]
    fn imports_a_jscontact_card() {
        use alloc::borrow::ToOwned;

        use crate::{param::VcardParam, prop::VcardPropKind, value::VcardValue};

        let jscontact = json!({
            "@type": "Card",
            "version": "1.0",
            "name": {
                "full": "Jane Doe",
                "components": [
                    { "kind": "surname", "value": "Doe" },
                    { "kind": "given", "value": "Jane" },
                ],
            },
            "emails": {
                "e1": {
                    "address": "jane@example.com",
                    "contexts": { "private": true },
                    "pref": 2,
                },
            },
            "anniversaries": {
                "a1": { "kind": "birth", "date": { "month": 4, "day": 12 } },
            },
            "onlineServices": {
                "o1": { "user": "@jane", "service": "Mastodon" },
            },
            "x-custom": { "hello": "world" },
        });
        let card = Vcard::from_jscontact(&jscontact).unwrap();

        // NOTE: members convert in their (alphabetical) JSON order.
        let names: Vec<&str> = card.properties.iter().map(|prop| &*prop.name).collect();
        assert_eq!(
            names,
            ["BDAY", "EMAIL", "FN", "N", "SOCIALPROFILE", "JSPROP"],
        );

        let bday = &card.properties[0];
        assert_eq!(bday.params, vec![VcardParam::PropId(Cow::Borrowed("a1"))]);
        assert!(matches!(&bday.value, VcardValue::DateAndOrTime(date) if date.0 == "--0412"),);

        let email = &card.properties[1];
        assert_eq!(
            email.params,
            vec![
                VcardParam::PropId(Cow::Borrowed("e1")),
                VcardParam::Pref(Cow::Owned("2".to_owned())),
                VcardParam::Type(vec![Cow::Borrowed("home")]),
            ],
        );

        let profile = &card.properties[4];
        assert_eq!(
            profile.name,
            VcardPropName::Kind(VcardPropKind::SocialProfile)
        );
        assert!(
            profile
                .params
                .contains(&VcardParam::ServiceType(Cow::Borrowed("Mastodon"))),
        );
        assert!(matches!(&profile.value, VcardValue::Text(user) if user.0 == "@jane"));

        let jsprop = &card.properties[5];
        assert_eq!(
            jsprop.params,
            vec![VcardParam::Jsptr(Cow::Borrowed("/x-custom"))],
        );
        assert!(
            matches!(&jsprop.value, VcardValue::Text(json) if json.0 == "{\"hello\":\"world\"}"),
        );
    }

    #[test]
    fn export_import_export_is_a_fixpoint() {
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "KIND:group\r\n",
            "UID:urn:uuid:4fbe8971-0bc3-424c-9c26-36c3e1eff6b1\r\n",
            "FN:Simon Perreault\r\n",
            "N;SORT-AS=\"Perreault,Simon\":Perreault;Simon;;;ing. jr,M.Sc.\r\n",
            "ORG;SORT-AS=Viagenie:Viagenie;IT\r\n",
            "TITLE:Director\r\n",
            "ROLE:Project lead\r\n",
            "NICKNAME:Sim\r\n",
            "EMAIL;TYPE=work;PREF=1:simon@example.com\r\n",
            "TEL;TYPE=work,cell,voice:tel:+1-418-262-6501\r\n",
            "IMPP:xmpp:simon@example.com\r\n",
            "ADR;TYPE=home;LABEL=The full label:;;2875 boul. Laurier;Quebec;QC;G1V 2M2;Canada\r\n",
            "LANG;PREF=2:fr\r\n",
            "LANGUAGE:fr\r\n",
            "CREATED:20200101T000000Z\r\n",
            "GRAMGENDER:masculine\r\n",
            "PRONOUNS;PROP-ID=p1:he/him\r\n",
            "SOCIALPROFILE;SERVICE-TYPE=Mastodon:https://example.social/@simon\r\n",
            "BDAY:--0203\r\n",
            "ANNIVERSARY:20090808T143000Z\r\n",
            "CATEGORIES:developer,ietf\r\n",
            "NOTE:Hello\r\n",
            "URL:https://example.com\r\n",
            "SOURCE:https://directory.example.com/simon.vcf\r\n",
            "KEY;MEDIATYPE=application/pgp-keys:https://example.com/key.asc\r\n",
            "CALURI:https://example.com/cal.ics\r\n",
            "FBURL:https://example.com/fb.ifb\r\n",
            "CALADRURI:mailto:simon@example.com\r\n",
            "PHOTO;MEDIATYPE=image/jpeg:https://example.com/photo.jpg\r\n",
            "MEMBER:urn:uuid:john\r\n",
            "RELATED;TYPE=friend:urn:uuid:jimmy\r\n",
            "REV:20240101T000000Z\r\n",
            "GENDER:M\r\n",
            "JSPROP;JSPTR=/foo/bar:{\"baz\":42}\r\n",
            "X-FOO;X-BAR=1:baz\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let exported = cst.decode().to_jscontact();

        let reimported = Vcard::from_jscontact(&exported).unwrap();
        assert_eq!(reimported.to_jscontact(), exported);
    }

    #[test]
    fn converts_the_rfc_9554_address_components() {
        // NOTE: an 18-component ADR: the RFC 9554 slots map to their
        // JSContact component kinds, and the pair aliases resolve (street
        // name over street, apartment over extended address).
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:Simon\r\n",
            "ADR:;;;Quebec;QC;G1V 2M2;Canada;8th wing;;2;2875;boul. Laurier;;;;;;\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let exported = cst.decode().to_jscontact();

        assert_eq!(
            exported["addresses"]["1"]["components"],
            json!([
                { "@type": "AddressComponent", "kind": "name", "value": "boul. Laurier" },
                { "@type": "AddressComponent", "kind": "locality", "value": "Quebec" },
                { "@type": "AddressComponent", "kind": "region", "value": "QC" },
                { "@type": "AddressComponent", "kind": "postcode", "value": "G1V 2M2" },
                { "@type": "AddressComponent", "kind": "country", "value": "Canada" },
                { "@type": "AddressComponent", "kind": "room", "value": "8th wing" },
                { "@type": "AddressComponent", "kind": "floor", "value": "2" },
                { "@type": "AddressComponent", "kind": "number", "value": "2875" },
            ]),
        );

        let reimported = Vcard::from_jscontact(&exported).unwrap();
        assert_eq!(reimported.to_jscontact(), exported);

        // NOTE: a card filling both a legacy slot and its RFC 9554 alias
        // cannot pick a side, so it is escaped whole.
        let input = concat!(
            "BEGIN:VCARD\r\n",
            "VERSION:4.0\r\n",
            "FN:Simon\r\n",
            "ADR:;;2875 boul. Laurier;;;;;;;;;boul. Laurier;;;;;;\r\n",
            "END:VCARD\r\n",
        );
        let cst = VcardCst::parse(input).unwrap();
        let exported = cst.decode().to_jscontact();

        assert!(exported.get("addresses").is_none());
        assert_eq!(exported["vCardProps"][0][0], json!("adr"));
    }

    #[test]
    fn errors_only_on_a_non_object_root() {
        assert!(Vcard::from_jscontact(&json!([])).is_err());
        assert!(Vcard::from_jscontact(&json!("card")).is_err());

        let object = json!({});
        let empty = Vcard::from_jscontact(&object).unwrap();
        assert!(empty.properties.is_empty());
    }

    #[test]
    fn converts_partial_dates_and_utc_timestamps() {
        use crate::jscontact::{partial_date, utc_timestamp};

        assert_eq!(
            partial_date("19850412"),
            Some((Some(1985), Some(4), Some(12)))
        );
        assert_eq!(partial_date("1985-04"), Some((Some(1985), Some(4), None)));
        assert_eq!(partial_date("1985"), Some((Some(1985), None, None)));
        assert_eq!(partial_date("--0412"), Some((None, Some(4), Some(12))));
        assert_eq!(partial_date("--04"), Some((None, Some(4), None)));
        assert_eq!(partial_date("---12"), Some((None, None, Some(12))));
        assert_eq!(partial_date("circa 1800"), None);

        assert_eq!(
            utc_timestamp("20090808T143000Z").as_deref(),
            Some("2009-08-08T14:30:00Z"),
        );
        assert_eq!(
            utc_timestamp("2009-08-08T14:30:00Z").as_deref(),
            Some("2009-08-08T14:30:00Z"),
        );
        assert_eq!(utc_timestamp("20090808T143000-0500"), None);
        assert_eq!(utc_timestamp("20090808"), None);
    }
}
