//! # Property lenses
//!
//! The property lens contract and one hand-written module per RFC 6350
//! property.
//!
//! [`VcardPropLens`] ties a wire name to a decoded value type plus the `decode`
//! projection and an edit cursor; each property implements
//! it in its own submodule, where the marker is the type-level key for
//! [`VcardCst::prop`](crate::tree::cst::VcardCst::prop). Scalar, list and URI
//! properties share the generic
//! [`VcardValueCursor`](crate::tree::cursor::VcardValueCursor); the structured
//! ones (`N`, `ADR`, `GENDER`, `CLIENTPIDMAP`) carry a cursor that names their
//! components. The name dispatch for whole-card decoding lives in
//! [`crate::tree::codec::decode`].

pub mod adr;
pub mod agent;
pub mod anniversary;
pub mod bday;
pub mod caladruri;
pub mod caluri;
pub mod categories;
pub mod class;
pub mod client_pid_map;
pub mod email;
pub mod fburl;
pub mod r#fn;
pub mod gender;
pub mod geo;
pub mod impp;
pub mod key;
pub mod kind;
pub mod label;
pub mod lang;
pub mod logo;
pub mod mailer;
pub mod member;
pub mod n;
pub mod name;
pub mod nickname;
pub mod note;
pub mod org;
pub mod photo;
pub mod prodid;
pub mod profile;
pub mod related;
pub mod rev;
pub mod role;
pub mod sort_string;
pub mod sound;
pub mod source;
pub mod tel;
pub mod title;
pub mod tz;
pub mod uid;
pub mod url;
pub mod xml;

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{codec::value::Codec, line::VcardLine},
    value::VcardValueKind,
    version::VcardVersion,
};

/// A property identified by type: its decoded value type, edit cursor, and the
/// `decode` projection from the generic syntax node onto the type. The wire name
/// comes from its [`VcardPropSpec::KIND`] (a supertrait), so the two stay in
/// sync.
pub trait VcardPropLens: VcardPropSpec {
    /// The decoded value type, borrowing the syntax node for reads. Its
    /// [`Codec`] impl is what the default `decode` delegates to.
    type Target<'v>: Codec<'v>;

    /// The typed edit cursor over a content line.
    type Cursor<'c, 'a>
    where
        'a: 'c;

    /// Project a content line onto the decoded type, consulting the card
    /// version where the value's shape is version-specific (`GEO`, the binary
    /// props). The default ignores the version and decodes the value node
    /// through the target's [`Codec`]; the version-specific lenses override it.
    fn decode<'v>(line: &'v VcardLine<'_>, _version: VcardVersion) -> Self::Target<'v> {
        <Self::Target<'v> as Codec<'v>>::decode(&line.value)
    }

    /// Wrap a content line in the typed cursor for in-place editing.
    fn cursor<'c, 'a>(line: &'c mut VcardLine<'a>) -> Self::Cursor<'c, 'a>;
}

/// The RFC 6350 section 6 property multiplicity: how many times a property may
/// appear in a card. Prop multiplicity, not value structure, so it is not
/// derivable from the value kind (`FN` and `NOTE` are both text but differ).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VcardPropCardinality {
    /// Exactly one (required, single).
    ExactlyOne,
    /// At most one (optional, single).
    AtMostOne,
    /// One or more (required, repeatable).
    OneOrMore,
    /// Any number, including zero (optional, repeatable).
    Any,
}

/// The default parameters a property may carry, used by the spec for the
/// uniform majority. Per-property sets refine this where a property allows more
/// or fewer.
const COMMON_PARAMS: &[VcardParamKind] = &[
    VcardParamKind::Value,
    VcardParamKind::Language,
    VcardParamKind::Pref,
    VcardParamKind::AltId,
    VcardParamKind::Pid,
    VcardParamKind::Type,
];

/// The per-property contract: the versions it lives in, its multiplicity, the
/// value-types and parameters it may carry (all per version), and the
/// value-type in force for a given version and optionally declared `VALUE`.
///
/// Implemented on the zero-sized lens markers. The defaults cover the uniform
/// majority (a single text value, valid in every version), so a property
/// overrides only where it diverges; the only required item is
/// [`KIND`](Self::KIND). The value axis and the `VALUE` axis resolve together
/// in [`value`](Self::value), which is what the decoder consults to pick a
/// value kind.
pub trait VcardPropSpec {
    /// The property this spec describes.
    const KIND: VcardPropKind;

    /// The versions in which the property is defined (the existence axis).
    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V2_1, VcardVersion::V3_0, VcardVersion::V4_0]
    }

    /// How many times the property may appear in a card, in the given version.
    /// Most properties are repeatable; the single-valued ones override this.
    fn cardinality(_version: VcardVersion) -> VcardPropCardinality {
        VcardPropCardinality::Any
    }

    /// The value-types the property may take, in default-first order, for the
    /// given version. Index 0 is the type used when no `VALUE` is declared.
    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::Text]
    }

    /// The parameters the property may carry, in the given version.
    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        COMMON_PARAMS
    }

    /// The value-type in force: the declared `VALUE` kind if any, else the
    /// version default ([`allowed_values`](Self::allowed_values)'s first), else
    /// [`Text`](VcardValueKind::Text). Liberal: a declared kind outside
    /// `allowed_values` is honoured here (membership is a validation concern).
    fn value(version: VcardVersion, declared: Option<VcardValueKind>) -> VcardValueKind {
        declared
            .or_else(|| Self::allowed_values(version).first().copied())
            .unwrap_or(VcardValueKind::Text)
    }
}

/// The spec of a property as function pointers, the runtime bridge from the
/// open [`VcardPropKind`] back to the static per-marker [`VcardPropSpec`]
/// impls. The decoder and the validator dispatch a prop kind through
/// [`prop_spec`] and then call these, instead of each owning a 42-arm match.
pub(crate) struct VcardPropSpecFns {
    /// See [`VcardPropSpec::allowed_versions`].
    pub allowed_versions: fn() -> &'static [VcardVersion],
    /// See [`VcardPropSpec::cardinality`].
    pub cardinality: fn(VcardVersion) -> VcardPropCardinality,
    /// See [`VcardPropSpec::allowed_values`].
    pub allowed_values: fn(VcardVersion) -> &'static [VcardValueKind],
    /// See [`VcardPropSpec::allowed_params`].
    pub allowed_params: fn(VcardVersion) -> &'static [VcardParamKind],
    /// See [`VcardPropSpec::value`].
    pub value: fn(VcardVersion, Option<VcardValueKind>) -> VcardValueKind,
}

/// Collect the spec function pointers of a marker type.
fn spec_fns<L: VcardPropSpec>() -> VcardPropSpecFns {
    VcardPropSpecFns {
        allowed_versions: L::allowed_versions,
        cardinality: L::cardinality,
        allowed_values: L::allowed_values,
        allowed_params: L::allowed_params,
        value: L::value,
    }
}

/// Dispatch a property kind onto its marker spec.
pub(crate) fn prop_spec(prop: VcardPropKind) -> VcardPropSpecFns {
    use VcardPropKind::*;

    match prop {
        Adr => spec_fns::<adr::ADR>(),
        Agent => spec_fns::<agent::AGENT>(),
        Anniversary => spec_fns::<anniversary::ANNIVERSARY>(),
        Bday => spec_fns::<bday::BDAY>(),
        CalAdrUri => spec_fns::<caladruri::CALADRURI>(),
        CalUri => spec_fns::<caluri::CALURI>(),
        Categories => spec_fns::<categories::CATEGORIES>(),
        Class => spec_fns::<class::CLASS>(),
        ClientPidMap => spec_fns::<client_pid_map::CLIENTPIDMAP>(),
        Email => spec_fns::<email::EMAIL>(),
        FbUrl => spec_fns::<fburl::FBURL>(),
        Fn => spec_fns::<r#fn::FN>(),
        Gender => spec_fns::<gender::GENDER>(),
        Geo => spec_fns::<geo::GEO>(),
        Impp => spec_fns::<impp::IMPP>(),
        Key => spec_fns::<key::KEY>(),
        Kind => spec_fns::<kind::KIND>(),
        Label => spec_fns::<label::LABEL>(),
        Lang => spec_fns::<lang::LANG>(),
        Logo => spec_fns::<logo::LOGO>(),
        Mailer => spec_fns::<mailer::MAILER>(),
        Member => spec_fns::<member::MEMBER>(),
        N => spec_fns::<n::N>(),
        Name => spec_fns::<name::NAME>(),
        Nickname => spec_fns::<nickname::NICKNAME>(),
        Note => spec_fns::<note::NOTE>(),
        Org => spec_fns::<org::ORG>(),
        Photo => spec_fns::<photo::PHOTO>(),
        ProdId => spec_fns::<prodid::PRODID>(),
        Profile => spec_fns::<profile::PROFILE>(),
        Related => spec_fns::<related::RELATED>(),
        Rev => spec_fns::<rev::REV>(),
        Role => spec_fns::<role::ROLE>(),
        SortString => spec_fns::<sort_string::SORTSTRING>(),
        Sound => spec_fns::<sound::SOUND>(),
        Source => spec_fns::<source::SOURCE>(),
        Tel => spec_fns::<tel::TEL>(),
        Title => spec_fns::<title::TITLE>(),
        Tz => spec_fns::<tz::TZ>(),
        Uid => spec_fns::<uid::UID>(),
        Url => spec_fns::<url::URL>(),
        Xml => spec_fns::<xml::XML>(),
    }
}
