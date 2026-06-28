//! # Properties
//!
//! A decoded property and the RFC 6350 property-name vocabulary.
//!
//! A [`VcardProp`] is a name, a list of parameters, and a decoded value. The
//! name is stored explicitly (as written on the wire) because many properties
//! share one [`VcardValue`] kind: `FN` and `TITLE` both decode to text, so the
//! value alone cannot say which property it is. The `VCARD_*` consts here are
//! the single source of truth for those names; the lens markers in
//! [`crate::tree::prop`] reference them to match and build lines, and the decode
//! registry uses them to dispatch a line onto its value kind.
//!
//! Each property has a named constructor (`VcardProp::r#fn`,
//! [`VcardProp::email`], [`VcardProp::n`], ...) that pins both the wire name and
//! the value kind, so building a regular property means neither spelling the name
//! nor knowing which [`VcardValue`] variant it takes. They are the discoverable
//! entry point: browse them under [`VcardProp`].
//!
//! This module is pure model: it has no dependency on [`crate::tree`], so the
//! decoded form can be used without the syntax layer.

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    param::VcardParam,
    value::{
        VcardValue,
        adr::VcardAdr,
        client_pid_map::VcardClientPidMap,
        datetime::{VcardDateAndOrTime, VcardTimestamp},
        gender::VcardGender,
        language::VcardLanguageTag,
        n::VcardN,
        org::VcardOrg,
        text::{VcardText, VcardTextList},
        uri::VcardUri,
    },
};

pub(crate) const VCARD_ADR: &str = "ADR";
pub(crate) const VCARD_ANNIVERSARY: &str = "ANNIVERSARY";
pub(crate) const VCARD_BDAY: &str = "BDAY";
pub(crate) const VCARD_CALADRURI: &str = "CALADRURI";
pub(crate) const VCARD_CALURI: &str = "CALURI";
pub(crate) const VCARD_CATEGORIES: &str = "CATEGORIES";
pub(crate) const VCARD_CLIENTPIDMAP: &str = "CLIENTPIDMAP";
pub(crate) const VCARD_EMAIL: &str = "EMAIL";
pub(crate) const VCARD_FBURL: &str = "FBURL";
pub(crate) const VCARD_FN: &str = "FN";
pub(crate) const VCARD_GENDER: &str = "GENDER";
pub(crate) const VCARD_GEO: &str = "GEO";
pub(crate) const VCARD_IMPP: &str = "IMPP";
pub(crate) const VCARD_KEY: &str = "KEY";
pub(crate) const VCARD_KIND: &str = "KIND";
pub(crate) const VCARD_LANG: &str = "LANG";
pub(crate) const VCARD_LOGO: &str = "LOGO";
pub(crate) const VCARD_MEMBER: &str = "MEMBER";
pub(crate) const VCARD_N: &str = "N";
pub(crate) const VCARD_NICKNAME: &str = "NICKNAME";
pub(crate) const VCARD_NOTE: &str = "NOTE";
pub(crate) const VCARD_ORG: &str = "ORG";
pub(crate) const VCARD_PHOTO: &str = "PHOTO";
pub(crate) const VCARD_PRODID: &str = "PRODID";
pub(crate) const VCARD_RELATED: &str = "RELATED";
pub(crate) const VCARD_REV: &str = "REV";
pub(crate) const VCARD_ROLE: &str = "ROLE";
pub(crate) const VCARD_SOUND: &str = "SOUND";
pub(crate) const VCARD_SOURCE: &str = "SOURCE";
pub(crate) const VCARD_TEL: &str = "TEL";
pub(crate) const VCARD_TITLE: &str = "TITLE";
pub(crate) const VCARD_TZ: &str = "TZ";
pub(crate) const VCARD_UID: &str = "UID";
pub(crate) const VCARD_URL: &str = "URL";
pub(crate) const VCARD_XML: &str = "XML";

/// A decoded property: its wire name, its parameters, and its decoded value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VcardProp<'a> {
    /// The property name, as written (e.g. `FN`, `N`, `EMAIL`).
    pub name: Cow<'a, str>,
    /// The parameters decorating the property.
    pub params: Vec<VcardParam<'a>>,
    /// The decoded value.
    pub value: VcardValue<'a>,
}

impl<'a> VcardProp<'a> {
    /// Build an `ADR` property from its parameters and value.
    pub fn adr(params: Vec<VcardParam<'a>>, value: VcardAdr<'a>) -> Self {
        Self {
            name: VCARD_ADR.into(),
            params,
            value: VcardValue::Adr(value),
        }
    }

    /// Build an `ANNIVERSARY` property from its parameters and value.
    pub fn anniversary(params: Vec<VcardParam<'a>>, value: VcardDateAndOrTime<'a>) -> Self {
        Self {
            name: VCARD_ANNIVERSARY.into(),
            params,
            value: VcardValue::DateAndOrTime(value),
        }
    }

    /// Build a `BDAY` property from its parameters and value.
    pub fn bday(params: Vec<VcardParam<'a>>, value: VcardDateAndOrTime<'a>) -> Self {
        Self {
            name: VCARD_BDAY.into(),
            params,
            value: VcardValue::DateAndOrTime(value),
        }
    }

    /// Build a `CALADRURI` property from its parameters and value.
    pub fn caladruri(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_CALADRURI.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build a `CALURI` property from its parameters and value.
    pub fn caluri(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_CALURI.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build a `CATEGORIES` property from its parameters and value.
    pub fn categories(params: Vec<VcardParam<'a>>, value: VcardTextList<'a>) -> Self {
        Self {
            name: VCARD_CATEGORIES.into(),
            params,
            value: VcardValue::TextList(value),
        }
    }

    /// Build a `CLIENTPIDMAP` property from its parameters and value.
    pub fn clientpidmap(params: Vec<VcardParam<'a>>, value: VcardClientPidMap<'a>) -> Self {
        Self {
            name: VCARD_CLIENTPIDMAP.into(),
            params,
            value: VcardValue::ClientPidMap(value),
        }
    }

    /// Build an `EMAIL` property from its parameters and value.
    pub fn email(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_EMAIL.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build an `FBURL` property from its parameters and value.
    pub fn fburl(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_FBURL.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build an `FN` property from its parameters and value.
    pub fn r#fn(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_FN.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build a `GENDER` property from its parameters and value.
    pub fn gender(params: Vec<VcardParam<'a>>, value: VcardGender<'a>) -> Self {
        Self {
            name: VCARD_GENDER.into(),
            params,
            value: VcardValue::Gender(value),
        }
    }

    /// Build a `GEO` property from its parameters and value.
    pub fn geo(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_GEO.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build an `IMPP` property from its parameters and value.
    pub fn impp(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_IMPP.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build a `KEY` property from its parameters and value.
    pub fn key(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_KEY.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build a `KIND` property from its parameters and value.
    pub fn kind(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_KIND.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build a `LANG` property from its parameters and value.
    pub fn lang(params: Vec<VcardParam<'a>>, value: VcardLanguageTag<'a>) -> Self {
        Self {
            name: VCARD_LANG.into(),
            params,
            value: VcardValue::LanguageTag(value),
        }
    }

    /// Build a `LOGO` property from its parameters and value.
    pub fn logo(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_LOGO.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build a `MEMBER` property from its parameters and value.
    pub fn member(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_MEMBER.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build an `N` property from its parameters and value.
    pub fn n(params: Vec<VcardParam<'a>>, value: VcardN<'a>) -> Self {
        Self {
            name: VCARD_N.into(),
            params,
            value: VcardValue::N(value),
        }
    }

    /// Build a `NICKNAME` property from its parameters and value.
    pub fn nickname(params: Vec<VcardParam<'a>>, value: VcardTextList<'a>) -> Self {
        Self {
            name: VCARD_NICKNAME.into(),
            params,
            value: VcardValue::TextList(value),
        }
    }

    /// Build a `NOTE` property from its parameters and value.
    pub fn note(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_NOTE.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build an `ORG` property from its parameters and value.
    pub fn org(params: Vec<VcardParam<'a>>, value: VcardOrg<'a>) -> Self {
        Self {
            name: VCARD_ORG.into(),
            params,
            value: VcardValue::Org(value),
        }
    }

    /// Build a `PHOTO` property from its parameters and value.
    pub fn photo(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_PHOTO.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build a `PRODID` property from its parameters and value.
    pub fn prodid(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_PRODID.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build a `RELATED` property from its parameters and value.
    pub fn related(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_RELATED.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build a `REV` property from its parameters and value.
    pub fn rev(params: Vec<VcardParam<'a>>, value: VcardTimestamp<'a>) -> Self {
        Self {
            name: VCARD_REV.into(),
            params,
            value: VcardValue::Timestamp(value),
        }
    }

    /// Build a `ROLE` property from its parameters and value.
    pub fn role(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_ROLE.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build a `SOUND` property from its parameters and value.
    pub fn sound(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_SOUND.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build a `SOURCE` property from its parameters and value.
    pub fn source(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_SOURCE.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build a `TEL` property from its parameters and value.
    pub fn tel(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_TEL.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build a `TITLE` property from its parameters and value.
    pub fn title(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_TITLE.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build a `TZ` property from its parameters and value.
    pub fn tz(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_TZ.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build a `UID` property from its parameters and value.
    pub fn uid(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_UID.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build a `URL` property from its parameters and value.
    pub fn url(params: Vec<VcardParam<'a>>, value: VcardUri<'a>) -> Self {
        Self {
            name: VCARD_URL.into(),
            params,
            value: VcardValue::Uri(value),
        }
    }

    /// Build an `XML` property from its parameters and value.
    pub fn xml(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_XML.into(),
            params,
            value: VcardValue::Text(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, vec, vec::Vec};

    use crate::{
        prop::VcardProp,
        value::{VcardValue, text::VcardText},
    };

    #[test]
    fn names_the_property_and_wraps_the_value() {
        let prop = VcardProp::title(Vec::new(), "Developer".into());
        assert_eq!(prop.name, "TITLE");
        assert_eq!(
            prop.value,
            VcardValue::Text(VcardText(Cow::Borrowed("Developer"))),
        );
        assert!(prop.params.is_empty());
    }

    #[test]
    fn carries_the_given_parameters() {
        let prop = VcardProp::r#fn(vec![], VcardText(Cow::Borrowed("John")));
        assert_eq!(prop.name, "FN");
    }
}
