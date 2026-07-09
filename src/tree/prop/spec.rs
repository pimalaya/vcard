//! # Property spec
//!
//! The per-property contract on the lens markers, and the runtime vtable that
//! bridges the open [`VcardPropKind`] back to those static impls.

use crate::{
    param::VcardParamKind,
    prop::VcardPropKind,
    tree::{
        param::COMMON_PARAMS,
        prop::{
            VcardPropCardinality, adr, agent, anniversary, bday, caladruri, caluri, categories,
            class, client_pid_map, created, email, fburl, r#fn, gender, geo, gramgender, impp,
            jsprop, key, kind, label, lang, language, logo, mailer, member, n, name, nickname,
            note, org, photo, prodid, profile, pronouns, related, rev, role, socialprofile,
            sort_string, sound, source, tel, title, tz, uid, url, xml,
        },
    },
    value::VcardValueKind,
    version::VcardVersion,
};

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
        Created => spec_fns::<created::CREATED>(),
        Email => spec_fns::<email::EMAIL>(),
        FbUrl => spec_fns::<fburl::FBURL>(),
        Fn => spec_fns::<r#fn::FN>(),
        Gender => spec_fns::<gender::GENDER>(),
        Geo => spec_fns::<geo::GEO>(),
        GramGender => spec_fns::<gramgender::GRAMGENDER>(),
        Impp => spec_fns::<impp::IMPP>(),
        JsProp => spec_fns::<jsprop::JSPROP>(),
        Key => spec_fns::<key::KEY>(),
        Kind => spec_fns::<kind::KIND>(),
        Label => spec_fns::<label::LABEL>(),
        Lang => spec_fns::<lang::LANG>(),
        Language => spec_fns::<language::LANGUAGE>(),
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
        Pronouns => spec_fns::<pronouns::PRONOUNS>(),
        Related => spec_fns::<related::RELATED>(),
        Rev => spec_fns::<rev::REV>(),
        Role => spec_fns::<role::ROLE>(),
        SocialProfile => spec_fns::<socialprofile::SOCIALPROFILE>(),
        SortString => spec_fns::<sort_string::SORT_STRING>(),
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

#[cfg(test)]
mod tests {
    use crate::{
        prop::VcardPropKind, tree::prop::prop_spec, value::VcardValueKind, version::VcardVersion,
    };

    #[test]
    fn every_property_spec_answers_for_every_version() {
        for kind in VcardPropKind::ALL {
            let spec = prop_spec(kind);

            for version in [VcardVersion::V2_1, VcardVersion::V3_0, VcardVersion::V4_0] {
                let _ = (spec.allowed_versions)();
                let _ = (spec.cardinality)(version);
                let _ = (spec.allowed_values)(version);
                let _ = (spec.allowed_params)(version);
                let _ = (spec.value)(version, None);
                let _ = (spec.value)(version, Some(VcardValueKind::Uri));
            }
        }
    }
}
