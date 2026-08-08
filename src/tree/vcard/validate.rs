//! # Validation
//!
//! The strict half of "liberal in, strict out", as a runtime predicate rather
//! than a second data model.
//!
//! Validity and lossiness are orthogonal: a conformant card may still carry
//! extensions (`X-`/IANA properties, unknown parameters), so "valid" cannot be
//! a type with no `Unknown` arms. [`Vcard::validate`] therefore checks the
//! *known* parts of the (lossy) model against the per-property
//! [`VcardPropSpec`](crate::tree::prop::spec::VcardPropSpec) for the card
//! version (existence, value kind, parameters, cardinality) and leaves the
//! unknown parts alone. A passing check yields a [`VcardValid`] marker: it is
//! the only thing that can mint one, so holding a `VcardValid<Vcard>` is proof
//! the check passed. The same per-property check backs the
//! [`VcardPropBuilder`]'s strict construction.
//!
//! [`VcardPropBuilder`]: crate::tree::vcard::builder::VcardPropBuilder
//!
//! # Example
//!
//! Decode a parsed card, validate it into a proof, and convert that back into a
//! byte tree:
//!
//! ```rust
//! use vcard::tree::cst::VcardCst;
//!
//! let cst =
//!     VcardCst::parse("BEGIN:VCARD\r\nVERSION:4.0\r\nFN:John Doe\r\nEND:VCARD\r\n").unwrap();
//!
//! // validate consumes the card and returns the proof (or the violations).
//! let valid = cst.decode().validate().expect("a conformant 4.0 card");
//!
//! // The proof converts back into a byte tree for free.
//! let out = VcardCst::from(valid);
//! assert!(out.to_string().contains("FN:John Doe"));
//! ```

use core::{error, fmt, ops::Deref};

use alloc::vec::Vec;

use crate::{
    param::VcardParamKind,
    prop::{VcardProp, VcardPropKind, VcardPropName},
    tree::{
        cst::VcardCst,
        prop::{cardinality::VcardPropCardinality, spec::VcardPropSpecFns, spec::prop_spec},
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
    /// Check the card against RFC 6350 for its version: every known property
    /// must exist in the version, carry an allowed value kind and allowed
    /// parameters, and respect its multiplicity. Extensions (unknown
    /// properties and parameters) are conformant and pass. On success the card
    /// is yielded back as a [`VcardValid`] proof; on failure every violation is
    /// collected.
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
/// [`VcardPropBuilder`](crate::tree::vcard::builder::VcardPropBuilder).
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

    for param in &prop.params {
        if let Some(param) = param.kind()
            && !param_allowed(&spec, version, param)
        {
            errors.push(VcardValidateError::Param { prop: *kind, param });
        }
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

/// A value that has passed validation. The only way to mint one is a validating
/// conversion ([`Vcard::validate`] or its `TryFrom`), so holding a
/// `VcardValid<T>` is proof the check passed; it derefs to the inner value for
/// reads and yields it back with [`into_inner`](Self::into_inner). It is never
/// constructed directly: it is a marker, not data.
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

impl From<VcardValid<Vcard<'_>>> for VcardCst<'static> {
    fn from(card: VcardValid<Vcard<'_>>) -> Self {
        card.into_inner().encode()
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, string::ToString, vec, vec::Vec};

    use crate::{
        param::{VcardParam, VcardParamKind},
        prop::{VcardProp, VcardPropKind},
        tree::{
            cst::VcardCst,
            prop::cardinality::VcardPropCardinality,
            vcard::validate::{VcardValid, VcardValidateError},
        },
        value::{VcardValue, VcardValueKind, n::VcardN, text::VcardText, uri::VcardUri},
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

    #[test]
    fn allows_charset_in_2_1_but_not_4_0() {
        // NOTE: N and FN make the envelope conformant in both versions, so
        // only the CHARSET parameter (legal in 2.1, not 4.0) decides the
        // outcome.
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

    #[test]
    fn flags_a_required_property_that_is_absent() {
        // NOTE: 4.0 requires FN (one-or-more); a card without it fails.
        let errors = card(VcardVersion::V4_0, vec![]).validate().unwrap_err();
        assert!(errors.iter().any(|error| matches!(
            error,
            VcardValidateError::Cardinality {
                prop: VcardPropKind::Fn,
                ..
            },
        )));
    }

    #[test]
    fn flags_a_property_absent_from_the_version() {
        // NOTE: AGENT is 2.1 / 3.0 only, so in 4.0 it is undefined.
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

    #[test]
    fn flags_a_disallowed_parameter() {
        // NOTE: FN is text-only and does not allow MEDIATYPE.
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

    #[test]
    fn flags_a_property_that_appears_too_often() {
        // NOTE: 4.0 N is at-most-one; two N properties is too many.
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

    #[test]
    fn rejects_a_v4_only_parameter_in_2_1() {
        // NOTE: PID is 4.0-only, so on a 2.1 property it is disallowed.
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
    fn valid_proof_derefs_unwraps_and_converts() {
        let vcard = card(
            VcardVersion::V4_0,
            vec![prop(
                "FN",
                vec![],
                VcardValue::Text(VcardText(Cow::Borrowed("John"))),
            )],
        );

        // NOTE: TryFrom mints the proof; Deref reads through it.
        let valid = VcardValid::try_from(vcard.clone()).expect("a conformant card");
        assert_eq!(valid.version, VcardVersion::V4_0);

        // NOTE: A proof converts back into a byte tree.
        let cst = VcardCst::from(valid);
        assert!(cst.to_string().contains("FN:John"));

        // NOTE: into_inner yields the card back.
        let inner = vcard.validate().unwrap().into_inner();
        assert_eq!(inner.version, VcardVersion::V4_0);
    }
}
