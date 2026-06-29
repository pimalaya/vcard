//! # Properties
//!
//! A decoded property and the RFC 2426 property-name vocabulary.
//!
//! A [`VcardProp`] is a name, a list of parameters, and a decoded value. The
//! name is stored explicitly (as written on the wire) because many properties
//! share one [`VcardValue`] kind: `FN` and `TITLE` both decode to text, so the
//! value alone cannot say which property it is. The `VCARD_*` consts here are
//! the single source of truth for those names; the lens markers in
//! [`crate::v30::tree::prop`] reference them to match and build lines, and the decode
//! registry uses them to dispatch a line onto its value kind.
//!
//! Each property has a named constructor (`VcardProp::r#fn`,
//! [`VcardProp::email`], [`VcardProp::n`], ...) that pins both the wire name and
//! the value kind, so building a regular property means neither spelling the name
//! nor knowing which [`VcardValue`] variant it takes. They are the discoverable
//! entry point: browse them under [`VcardProp`].
//!
//! This module is pure model: it has no dependency on [`crate::v30::tree`], so the
//! decoded form can be used without the syntax layer.

use alloc::{borrow::Cow, vec::Vec};

use crate::v30::{
    param::VcardParam,
    value::{
        VcardValue,
        adr::VcardAdr,
        binary::VcardBinary,
        datetime::{VcardDateAndOrTime, VcardTimestamp},
        geo::VcardGeo,
        n::VcardN,
        org::VcardOrg,
        text::{VcardText, VcardTextList},
        uri::VcardUri,
        utc_offset::VcardUtcOffset,
    },
};

pub const VCARD_ADR: &str = "ADR";
pub const VCARD_AGENT: &str = "AGENT";
pub const VCARD_BDAY: &str = "BDAY";
pub const VCARD_CATEGORIES: &str = "CATEGORIES";
pub const VCARD_CLASS: &str = "CLASS";
pub const VCARD_EMAIL: &str = "EMAIL";
pub const VCARD_FN: &str = "FN";
pub const VCARD_GEO: &str = "GEO";
pub const VCARD_KEY: &str = "KEY";
pub const VCARD_LABEL: &str = "LABEL";
pub const VCARD_LOGO: &str = "LOGO";
pub const VCARD_MAILER: &str = "MAILER";
pub const VCARD_N: &str = "N";
pub const VCARD_NAME: &str = "NAME";
pub const VCARD_NICKNAME: &str = "NICKNAME";
pub const VCARD_NOTE: &str = "NOTE";
pub const VCARD_ORG: &str = "ORG";
pub const VCARD_PHOTO: &str = "PHOTO";
pub const VCARD_PRODID: &str = "PRODID";
pub const VCARD_PROFILE: &str = "PROFILE";
pub const VCARD_REV: &str = "REV";
pub const VCARD_ROLE: &str = "ROLE";
pub const VCARD_SORT_STRING: &str = "SORT-STRING";
pub const VCARD_SOUND: &str = "SOUND";
pub const VCARD_SOURCE: &str = "SOURCE";
pub const VCARD_TEL: &str = "TEL";
pub const VCARD_TITLE: &str = "TITLE";
pub const VCARD_TZ: &str = "TZ";
pub const VCARD_UID: &str = "UID";
pub const VCARD_URL: &str = "URL";

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

    /// Build an `AGENT` property from its parameters and value.
    pub fn agent(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_AGENT.into(),
            params,
            value: VcardValue::Text(value),
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

    /// Build a `CATEGORIES` property from its parameters and value.
    pub fn categories(params: Vec<VcardParam<'a>>, value: VcardTextList<'a>) -> Self {
        Self {
            name: VCARD_CATEGORIES.into(),
            params,
            value: VcardValue::TextList(value),
        }
    }

    /// Build a `CLASS` property from its parameters and value.
    pub fn class(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_CLASS.into(),
            params,
            value: VcardValue::Text(value),
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

    /// Build an `FN` property from its parameters and value.
    pub fn r#fn(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_FN.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build a `GEO` property from its parameters and value.
    pub fn geo(params: Vec<VcardParam<'a>>, value: VcardGeo<'a>) -> Self {
        Self {
            name: VCARD_GEO.into(),
            params,
            value: VcardValue::Geo(value),
        }
    }

    /// Build a `KEY` property from its parameters and value.
    pub fn key(params: Vec<VcardParam<'a>>, value: VcardBinary<'a>) -> Self {
        Self {
            name: VCARD_KEY.into(),
            params,
            value: VcardValue::Binary(value),
        }
    }

    /// Build a `LABEL` property from its parameters and value.
    pub fn label(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_LABEL.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build a `LOGO` property from its parameters and value.
    pub fn logo(params: Vec<VcardParam<'a>>, value: VcardBinary<'a>) -> Self {
        Self {
            name: VCARD_LOGO.into(),
            params,
            value: VcardValue::Binary(value),
        }
    }

    /// Build a `MAILER` property from its parameters and value.
    pub fn mailer(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_MAILER.into(),
            params,
            value: VcardValue::Text(value),
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

    /// Build a `NAME` property from its parameters and value.
    pub fn name(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_NAME.into(),
            params,
            value: VcardValue::Text(value),
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
    pub fn photo(params: Vec<VcardParam<'a>>, value: VcardBinary<'a>) -> Self {
        Self {
            name: VCARD_PHOTO.into(),
            params,
            value: VcardValue::Binary(value),
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

    /// Build a `PROFILE` property from its parameters and value.
    pub fn profile(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_PROFILE.into(),
            params,
            value: VcardValue::Text(value),
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

    /// Build a `SORT-STRING` property from its parameters and value.
    pub fn sort_string(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_SORT_STRING.into(),
            params,
            value: VcardValue::Text(value),
        }
    }

    /// Build a `SOUND` property from its parameters and value.
    pub fn sound(params: Vec<VcardParam<'a>>, value: VcardBinary<'a>) -> Self {
        Self {
            name: VCARD_SOUND.into(),
            params,
            value: VcardValue::Binary(value),
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
    pub fn tz(params: Vec<VcardParam<'a>>, value: VcardUtcOffset<'a>) -> Self {
        Self {
            name: VCARD_TZ.into(),
            params,
            value: VcardValue::UtcOffset(value),
        }
    }

    /// Build a `UID` property from its parameters and value.
    pub fn uid(params: Vec<VcardParam<'a>>, value: VcardText<'a>) -> Self {
        Self {
            name: VCARD_UID.into(),
            params,
            value: VcardValue::Text(value),
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
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, vec, vec::Vec};

    use crate::v30::{
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
