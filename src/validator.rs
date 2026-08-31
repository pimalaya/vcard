//! # Validator
//!
//! The strict half of "liberal in, strict out", as a runtime predicate rather
//! than a second data model.
//!
//! Validity and lossiness are orthogonal: a conformant card may still carry
//! extensions (`X-`/IANA properties, unknown parameters), so "valid" cannot be
//! a type with no `Unknown` arms.
//!
//! [`Vcard::validate`] therefore checks the *known* parts of the (lossy) model
//! against the per-property
//! [`VcardPropSpec`](crate::prop::spec::VcardPropSpec) for the card
//! version, and leaves the unknown parts alone.
//!
//! The check covers existence, value kind, parameters and cardinality, which
//! are a property's shape, plus the content of the few values whose RFC closes
//! it: `GENDER`'s sex code, `PROFILE`'s single value, `CLIENTPIDMAP`'s
//! identifier, and the `PREF`, `PID` and `DERIVED` parameters.
//!
//! Content stops there. A vocabulary ending in `iana-token / x-name` is open
//! and nothing to check against, and a date, a URI or a language tag is a
//! grammar rather than a set: reading those is a different appetite, and one
//! that would have this crate carrying a URI parser to answer a question no
//! caller asked.
//!
//! A passing check mints a [`VcardValid`] marker, the only way to obtain one,
//! so holding a `VcardValid<Vcard>` is proof the check passed. The same
//! per-property check backs the [`VcardPropBuilder`]'s strict construction.
//!
//! [`VcardPropBuilder`]: crate::builder::VcardPropBuilder
//!
//! ## Example
//!
//! Decode a parsed card, validate it into a proof, and convert that back into a
//! byte tree:
//!
//! ```rust
//! # #[cfg(feature = "parser")] {
//! use vcard::tree::cst::VcardCst;
//!
//! let raw = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nEND:VCARD\r\n";
//! let cst = VcardCst::parse(raw).unwrap();
//!
//! // validate consumes the card and returns the proof (or the violations).
//! let valid = cst.decode().validate().expect("a conformant 4.0 card");
//!
//! // The proof converts back into a byte tree for free.
//! let out = VcardCst::from(valid);
//! assert!(out.to_string().contains("FN:John Doe"));
//! # }
//! ```

use core::{error, fmt, ops::Deref};

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::{
    param::{VcardParam, VcardParamKind},
    prop::{
        VcardProp, VcardPropKind, VcardPropName,
        cardinality::VcardPropCardinality,
        spec::{VcardPropSpecFns, prop_spec},
    },
    value::VcardValueKind,
    vcard::Vcard,
    version::VcardVersion,
};

/// A way a known property breaks its RFC 6350 contract for the card version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VcardValidateError {
    /// The property is not defined in this version.
    PropVersion {
        /// The offending property.
        prop: VcardPropKind,
        /// The card version.
        version: VcardVersion,
    },
    /// The value kind is not allowed for the property (a `None` is an undecoded
    /// value on a known property).
    ValueKind {
        /// The offending property.
        prop: VcardPropKind,
        /// The value kind found, if any.
        found: Option<VcardValueKind>,
    },
    /// The parameter is not allowed for the property in this version.
    Param {
        /// The offending property.
        prop: VcardPropKind,
        /// The disallowed parameter.
        param: VcardParamKind,
    },
    /// The value holds something the property's own definition forbids.
    ///
    /// Only the few properties whose RFC closes their content raise this:
    /// `GENDER`'s sex code, `PROFILE`'s single value, `CLIENTPIDMAP`'s
    /// identifier. A vocabulary ending in `iana-token / x-name` is open,
    /// and nothing to check against.
    Value {
        /// The offending property.
        prop: VcardPropKind,
        /// The value found, as the card wrote it.
        found: String,
    },
    /// The parameter's value is outside what the parameter accepts.
    ///
    /// `PREF` outside 1 to 100, a `PID` that is not a small integer or a
    /// pair of them, a `DERIVED` that is neither `true` nor `false`.
    ParamValue {
        /// The offending parameter.
        param: VcardParamKind,
        /// The value found, as the card wrote it.
        found: String,
    },
    /// The property appears a number of times its multiplicity forbids.
    Cardinality {
        /// The offending property.
        prop: VcardPropKind,
        /// The required multiplicity.
        cardinality: VcardPropCardinality,
        /// How many times it actually appears.
        count: usize,
    },
}

impl fmt::Display for VcardValidateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PropVersion {
                prop: p,
                version: v,
            } => {
                write!(f, "Property `{}` is not defined in vCard {}", &**p, &**v)
            }
            Self::ValueKind {
                prop: p,
                found: Some(k),
            } => {
                write!(f, "Value kind `{}` is not allowed for `{}`", &**k, &**p)
            }
            Self::ValueKind {
                prop: p,
                found: None,
            } => {
                write!(f, "An undecoded value is not allowed for `{}`", &**p)
            }
            Self::Param {
                prop: pp,
                param: pm,
            } => {
                write!(f, "Parameter `{}` is not allowed for `{}`", &**pm, &**pp)
            }
            Self::Value { prop: p, found } => {
                write!(f, "Value `{found}` is not allowed for `{}`", &**p)
            }
            Self::ParamValue { param: p, found } => {
                write!(f, "Value `{found}` is not allowed for parameter `{}`", &**p)
            }
            Self::Cardinality {
                prop: p,
                cardinality: cd,
                count: cn,
            } => {
                write!(f, "Property `{}` appears {cn} times but is {cd:?}", &**p)
            }
        }
    }
}

impl error::Error for VcardValidateError {}

impl<'a> Vcard<'a> {
    /// Check the card against RFC 6350 for its version.
    ///
    /// Every known property must exist in the version, carry an allowed value
    /// kind and parameters, and respect its multiplicity; extensions pass. The
    /// card comes back as a [`VcardValid`] proof, or every violation does.
    pub fn validate(self) -> Result<VcardValid<Vcard<'a>>, Vec<VcardValidateError>> {
        let mut errors = Vec::new();
        let mut counts: Vec<(VcardPropKind, usize)> = Vec::new();

        for prop in &self.properties {
            validate_prop(prop, self.version, &mut errors);
            if let VcardPropName::Kind(kind) = &prop.name {
                match counts.iter_mut().find(|(seen, _)| *seen == *kind) {
                    Some((_, count)) => *count += 1,
                    None => counts.push((*kind, 1)),
                }
            }
        }

        // NOTE: scan every kind defined in this version, so a required property
        // that is absent (count 0) is caught alongside one that appears too
        // often.
        for prop in VcardPropKind::ALL {
            let spec = prop_spec(prop);
            if !(spec.allowed_versions)().contains(&self.version) {
                continue;
            }
            let count = counts
                .iter()
                .find(|(seen, _)| *seen == prop)
                .map_or(0, |(_, count)| *count);
            let cardinality = (spec.cardinality)(self.version);
            if !cardinality_ok(cardinality, count) {
                errors.push(VcardValidateError::Cardinality {
                    prop,
                    cardinality,
                    count,
                });
            }
        }

        if errors.is_empty() {
            Ok(VcardValid(self))
        } else {
            Err(errors)
        }
    }
}

/// Check one property against its spec for the version, pushing any violations.
/// An unknown (extension) property is always conformant. Shared by
/// [`Vcard::validate`] and the
/// [`VcardPropBuilder`](crate::builder::VcardPropBuilder).
pub(crate) fn validate_prop(
    prop: &VcardProp<'_>,
    version: VcardVersion,
    errors: &mut Vec<VcardValidateError>,
) {
    let VcardPropName::Kind(kind) = &prop.name else {
        return;
    };

    let spec = prop_spec(*kind);

    if !(spec.allowed_versions)().contains(&version) {
        errors.push(VcardValidateError::PropVersion {
            prop: *kind,
            version,
        });
    }

    let value_kind = prop.value.kind();
    if !value_kind.is_some_and(|kind| (spec.allowed_values)(version).contains(&kind)) {
        errors.push(VcardValidateError::ValueKind {
            prop: *kind,
            found: value_kind,
        });
    }

    if let Some(found) = (spec.invalid_value)(&prop.value, version) {
        errors.push(VcardValidateError::Value { prop: *kind, found });
    }

    for param in &prop.params {
        let Some(param_kind) = param.kind() else {
            continue;
        };

        if !param_allowed(&spec, version, param_kind) {
            errors.push(VcardValidateError::Param {
                prop: *kind,
                param: param_kind,
            });
        }

        for found in invalid_param_values(param) {
            errors.push(VcardValidateError::ParamValue {
                param: param_kind,
                found,
            });
        }
    }
}

/// The values a parameter carries that its own definition forbids.
///
/// These constraints do not vary by the property carrying the parameter, so
/// one check serves every appearance rather than each property restating it.
///
/// `PREF` is an integer from 1 to 100 (RFC 6350 5.3), `PID` one or more
/// digits optionally followed by a dot and more digits (5.5), and `DERIVED`
/// either `true` or `false` (RFC 9554 3.4). Every other parameter is free
/// text, a language tag, a media type or an open vocabulary, and none of
/// those is a set to check against.
fn invalid_param_values(param: &VcardParam<'_>) -> Vec<String> {
    match param {
        VcardParam::Pref(value) => {
            let pref = value.parse::<u8>();
            offending(pref.is_ok_and(|pref| (1..=100).contains(&pref)), value)
        }
        VcardParam::Derived(value) => {
            let boolean = value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false");
            offending(boolean, value)
        }
        VcardParam::Pid(values) => values
            .iter()
            .filter(|value| !is_pid(value))
            .map(|value| value.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

/// A single-valued parameter's value when it is not allowed, nothing when it
/// is.
fn offending(allowed: bool, value: &str) -> Vec<String> {
    match allowed {
        true => Vec::new(),
        false => vec![value.to_string()],
    }
}

/// Whether a `PID` value is `1*DIGIT ["." 1*DIGIT]` (RFC 6350 5.5).
fn is_pid(value: &str) -> bool {
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());

    match value.split_once('.') {
        Some((source, client)) => digits(source) && digits(client),
        None => digits(value),
    }
}

/// Whether a known parameter is allowed on a property in a version. 4.0 uses
/// the spec's set directly; 2.1 / 3.0 drop the parameters introduced in 4.0 and
/// allow the legacy `ENCODING` / `CHARSET`.
fn param_allowed(spec: &VcardPropSpecFns, version: VcardVersion, kind: VcardParamKind) -> bool {
    let allowed = (spec.allowed_params)(version);
    match version {
        VcardVersion::V4_0 => allowed.contains(&kind) || is_universal(kind),
        _ => {
            matches!(kind, VcardParamKind::Charset | VcardParamKind::Encoding)
                || (allowed.contains(&kind) && !is_v4_only(kind))
        }
    }
}

/// Whether an RFC 9554 parameter is defined for any property, so 4.0 allows
/// it without each spec listing it.
fn is_universal(kind: VcardParamKind) -> bool {
    use VcardParamKind::*;

    matches!(kind, Author | AuthorName | Created | Derived | PropId)
}

/// Whether a parameter was introduced in vCard 4.0 or later (so 2.1 / 3.0
/// disallow it).
fn is_v4_only(kind: VcardParamKind) -> bool {
    use VcardParamKind::*;

    matches!(
        kind,
        Pid | Pref
            | AltId
            | MediaType
            | CalScale
            | SortAs
            | Geo
            | Tz
            | Label
            | Author
            | AuthorName
            | Created
            | Derived
            | Jsptr
            | Phonetic
            | PropId
            | Script
            | ServiceType
            | Username
    )
}

/// Whether `count` occurrences satisfy the multiplicity.
fn cardinality_ok(cardinality: VcardPropCardinality, count: usize) -> bool {
    match cardinality {
        VcardPropCardinality::ExactlyOne => count == 1,
        VcardPropCardinality::AtMostOne => count <= 1,
        VcardPropCardinality::OneOrMore => count >= 1,
        VcardPropCardinality::Any => true,
    }
}

/// A value that has passed validation.
///
/// The only way to mint one is a validating conversion ([`Vcard::validate`] or
/// its `TryFrom`), so holding a `VcardValid<T>` is proof the check passed. It
/// derefs for reads and yields the value with [`into_inner`](Self::into_inner).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VcardValid<T>(T);

impl<T> VcardValid<T> {
    /// Take the validated value back out.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for VcardValid<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<'a> TryFrom<Vcard<'a>> for VcardValid<Vcard<'a>> {
    type Error = Vec<VcardValidateError>;

    fn try_from(card: Vcard<'a>) -> Result<Self, Self::Error> {
        card.validate()
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec, vec::Vec};

    use crate::{
        param::{VcardParam, VcardParamKind},
        prop::{VcardProp, VcardPropKind, cardinality::VcardPropCardinality},
        validator::{VcardValid, VcardValidateError},
        value::{
            VcardValue, VcardValueKind, client_pid_map::VcardClientPidMap, gender::VcardGender,
            n::VcardN, text::VcardText, uri::VcardUri,
        },
        vcard::Vcard,
        version::VcardVersion,
    };

    fn prop(
        name: &'static str,
        params: Vec<VcardParam<'static>>,
        value: VcardValue<'static>,
    ) -> VcardProp<'static> {
        VcardProp {
            name: name.into(),
            params,
            value,
        }
    }

    fn card(version: VcardVersion, properties: Vec<VcardProp<'static>>) -> Vcard<'static> {
        Vcard {
            version,
            properties,
        }
    }

    #[test]
    fn accepts_a_conformant_card_and_extensions() {
        let vcard = card(
            VcardVersion::V4_0,
            vec![
                prop(
                    "FN",
                    vec![],
                    VcardValue::Text(VcardText(Cow::Borrowed("John"))),
                ),
                // NOTE: An X- extension property is conformant.
                prop("X-FOO", vec![], VcardValue::Unknown(Default::default())),
            ],
        );
        assert!(vcard.validate().is_ok());
    }

    /// A 4.0 card carrying `FN` and one more property, the minimum that
    /// passes cardinality.
    fn card_with(other: VcardProp<'static>) -> Vcard<'static> {
        card(
            VcardVersion::V4_0,
            vec![
                prop(
                    "FN",
                    vec![],
                    VcardValue::Text(VcardText(Cow::Borrowed("John"))),
                ),
                other,
            ],
        )
    }

    fn gender(sex: &'static str, identity: &'static str) -> VcardProp<'static> {
        prop(
            "GENDER",
            vec![],
            VcardValue::Gender(VcardGender {
                sex: Cow::Borrowed(sex),
                identity: Cow::Borrowed(identity),
            }),
        )
    }

    /// A property carrying one parameter, for the parameter checks.
    fn with_param(param: VcardParam<'static>) -> VcardProp<'static> {
        prop(
            "NICKNAME",
            vec![param],
            VcardValue::TextList(Default::default()),
        )
    }

    #[test]
    fn accepts_every_sex_code_the_rfc_defines() {
        // RFC 6350 6.2.7: sex = "" / "M" / "F" / "O" / "N" / "U", and RFC
        // 5234 makes a quoted ABNF literal case-insensitive.
        for sex in ["", "M", "F", "O", "N", "U", "m", "f"] {
            assert!(
                card_with(gender(sex, "")).validate().is_ok(),
                "rejected the sex code {sex:?}",
            );
        }
    }

    #[test]
    fn flags_a_sex_code_outside_the_vocabulary() {
        let errors = card_with(gender("X", "")).validate().unwrap_err();

        assert!(errors.contains(&VcardValidateError::Value {
            prop: VcardPropKind::Gender,
            found: "X".to_string(),
        }));
    }

    #[test]
    fn a_gender_outside_the_vocabulary_belongs_in_the_identity() {
        // Which is what the RFC's own `GENDER:;it's complicated` does.
        assert!(card_with(gender("", "it's complicated")).validate().is_ok());
    }

    #[test]
    fn flags_a_profile_that_is_not_vcard() {
        let profile = |value| {
            card(
                VcardVersion::V3_0,
                vec![
                    prop(
                        "FN",
                        vec![],
                        VcardValue::Text(VcardText(Cow::Borrowed("John"))),
                    ),
                    prop("N", vec![], VcardValue::N(VcardN::default())),
                    prop("PROFILE", vec![], VcardValue::Text(VcardText(value))),
                ],
            )
        };

        assert!(profile(Cow::Borrowed("VCARD")).validate().is_ok());
        assert!(profile(Cow::Borrowed("vcard")).validate().is_ok());
        assert!(profile(Cow::Borrowed("ICAL")).validate().is_err());
    }

    #[test]
    fn flags_a_client_pid_map_identifier_that_is_not_an_integer() {
        let map = |id| {
            card_with(prop(
                "CLIENTPIDMAP",
                vec![],
                VcardValue::ClientPidMap(VcardClientPidMap {
                    id,
                    uri: Cow::Borrowed("urn:uuid:1f"),
                }),
            ))
        };

        assert!(map(Cow::Borrowed("1")).validate().is_ok());
        assert!(map(Cow::Borrowed("")).validate().is_err());
        assert!(map(Cow::Borrowed("one")).validate().is_err());
    }

    #[test]
    fn flags_a_pref_outside_one_to_a_hundred() {
        let pref = |value| card_with(with_param(VcardParam::Pref(Cow::Borrowed(value))));

        assert!(pref("1").validate().is_ok());
        assert!(pref("100").validate().is_ok());
        assert!(pref("0").validate().is_err());
        assert!(pref("101").validate().is_err());
        assert!(pref("high").validate().is_err());
    }

    #[test]
    fn flags_a_pid_that_is_not_a_small_integer_pair() {
        let pid = |values| card_with(with_param(VcardParam::Pid(values)));

        assert!(pid(vec![Cow::Borrowed("1")]).validate().is_ok());
        assert!(pid(vec![Cow::Borrowed("1.1")]).validate().is_ok());
        assert!(pid(vec![Cow::Borrowed("1.")]).validate().is_err());
        assert!(pid(vec![Cow::Borrowed("a")]).validate().is_err());
    }

    #[test]
    fn flags_a_derived_that_is_not_a_boolean() {
        let derived = |value| card_with(with_param(VcardParam::Derived(Cow::Borrowed(value))));

        assert!(derived("true").validate().is_ok());
        assert!(derived("FALSE").validate().is_ok());
        assert!(derived("yes").validate().is_err());
    }

    #[test]
    fn an_open_vocabulary_is_not_checked() {
        // RFC 6350 6.1.4 ends KIND's grammar in `iana-token / x-name`, so a
        // value outside the listed ones still conforms.
        let kind = prop(
            "KIND",
            vec![],
            VcardValue::Text(VcardText(Cow::Borrowed("x-android-custom"))),
        );

        assert!(card_with(kind).validate().is_ok());
    }

    #[test]
    fn flags_a_value_kind_the_property_forbids() {
        let vcard = card(
            VcardVersion::V4_0,
            vec![prop(
                "FN",
                vec![],
                VcardValue::Uri(VcardUri(Cow::Borrowed("x"))),
            )],
        );
        let errors = vcard.validate().unwrap_err();
        assert!(matches!(errors[0], VcardValidateError::ValueKind { .. }));
    }

    /// N and FN make the envelope conformant in both versions, so only the
    /// CHARSET parameter, legal in 2.1 but not in 4.0, decides the outcome.
    #[test]
    fn allows_charset_in_2_1_but_not_4_0() {
        let with_charset = |version| {
            card(
                version,
                vec![
                    prop("N", vec![], VcardValue::N(VcardN::default())),
                    prop(
                        "FN",
                        vec![],
                        VcardValue::Text(VcardText(Cow::Borrowed("X"))),
                    ),
                    prop(
                        "NOTE",
                        vec![VcardParam::Charset(Cow::Borrowed("UTF-8"))],
                        VcardValue::Text(VcardText(Cow::Borrowed("hi"))),
                    ),
                ],
            )
            .validate()
        };

        assert!(with_charset(VcardVersion::V2_1).is_ok());
        assert!(with_charset(VcardVersion::V4_0).is_err());
    }

    /// 4.0 requires FN one or more times, so a card without it fails.
    #[test]
    fn flags_a_required_property_that_is_absent() {
        let errors = card(VcardVersion::V4_0, vec![]).validate().unwrap_err();
        assert!(errors.iter().any(|error| matches!(
            error,
            VcardValidateError::Cardinality {
                prop: VcardPropKind::Fn,
                ..
            },
        )));
    }

    /// AGENT is 2.1 and 3.0 only, so in 4.0 it is undefined.
    #[test]
    fn flags_a_property_absent_from_the_version() {
        let errors = card(
            VcardVersion::V4_0,
            vec![
                prop(
                    "FN",
                    vec![],
                    VcardValue::Text(VcardText(Cow::Borrowed("X"))),
                ),
                prop(
                    "AGENT",
                    vec![],
                    VcardValue::Text(VcardText(Cow::Borrowed("a"))),
                ),
            ],
        )
        .validate()
        .unwrap_err();
        assert!(errors.iter().any(|error| matches!(
            error,
            VcardValidateError::PropVersion {
                prop: VcardPropKind::Agent,
                ..
            },
        )));
    }

    /// FN is text-only and does not allow MEDIATYPE.
    #[test]
    fn flags_a_disallowed_parameter() {
        let errors = card(
            VcardVersion::V4_0,
            vec![prop(
                "FN",
                vec![VcardParam::MediaType(Cow::Borrowed("text/plain"))],
                VcardValue::Text(VcardText(Cow::Borrowed("X"))),
            )],
        )
        .validate()
        .unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| matches!(error, VcardValidateError::Param { .. },))
        );
    }

    /// 4.0 makes N at most one, so two N properties are too many.
    #[test]
    fn flags_a_property_that_appears_too_often() {
        let errors = card(
            VcardVersion::V4_0,
            vec![
                prop(
                    "FN",
                    vec![],
                    VcardValue::Text(VcardText(Cow::Borrowed("X"))),
                ),
                prop("N", vec![], VcardValue::N(VcardN::default())),
                prop("N", vec![], VcardValue::N(VcardN::default())),
            ],
        )
        .validate()
        .unwrap_err();
        assert!(errors.iter().any(|error| matches!(
            error,
            VcardValidateError::Cardinality {
                prop: VcardPropKind::N,
                count: 2,
                ..
            },
        )));
    }

    /// PID is 4.0 only, so on a 2.1 property it is disallowed.
    #[test]
    fn rejects_a_v4_only_parameter_in_2_1() {
        let errors = card(
            VcardVersion::V2_1,
            vec![
                prop("N", vec![], VcardValue::N(VcardN::default())),
                prop(
                    "FN",
                    vec![],
                    VcardValue::Text(VcardText(Cow::Borrowed("X"))),
                ),
                prop(
                    "TEL",
                    vec![VcardParam::Pid(vec![Cow::Borrowed("1")])],
                    VcardValue::Text(VcardText(Cow::Borrowed("123"))),
                ),
            ],
        )
        .validate()
        .unwrap_err();
        assert!(errors.iter().any(|error| matches!(
            error,
            VcardValidateError::Param {
                param: VcardParamKind::Pid,
                ..
            },
        )));
    }

    #[test]
    fn displays_every_validate_error_variant() {
        let errors = [
            VcardValidateError::PropVersion {
                prop: VcardPropKind::Agent,
                version: VcardVersion::V4_0,
            },
            VcardValidateError::ValueKind {
                prop: VcardPropKind::Fn,
                found: Some(VcardValueKind::Uri),
            },
            VcardValidateError::ValueKind {
                prop: VcardPropKind::Fn,
                found: None,
            },
            VcardValidateError::Param {
                prop: VcardPropKind::Fn,
                param: VcardParamKind::MediaType,
            },
            VcardValidateError::Cardinality {
                prop: VcardPropKind::N,
                cardinality: VcardPropCardinality::AtMostOne,
                count: 2,
            },
        ];
        for error in errors {
            assert!(!error.to_string().is_empty());
        }
    }

    #[test]
    fn valid_proof_derefs_and_unwraps() {
        let vcard = card(
            VcardVersion::V4_0,
            vec![prop(
                "FN",
                vec![],
                VcardValue::Text(VcardText(Cow::Borrowed("John"))),
            )],
        );

        let valid = VcardValid::try_from(vcard.clone()).expect("a conformant card");
        assert_eq!(valid.version, VcardVersion::V4_0);

        let inner = vcard.validate().unwrap().into_inner();
        assert_eq!(inner.version, VcardVersion::V4_0);
    }

    /// The proof converts back into a byte tree, which only a build carrying
    /// the syntax layer can do.
    #[cfg(feature = "parser")]
    #[test]
    fn valid_proof_converts_into_a_byte_tree() {
        use crate::tree::cst::VcardCst;

        let vcard = card(
            VcardVersion::V4_0,
            vec![prop(
                "FN",
                vec![],
                VcardValue::Text(VcardText(Cow::Borrowed("John"))),
            )],
        );

        let valid = VcardValid::try_from(vcard).expect("a conformant card");
        let cst = VcardCst::from(valid);

        assert!(cst.to_string().contains("FN:John"));
    }
}
