//! # Builders
//!
//! Strict, version-aware construction of a whole card and of a single property.
//!
//! [`VcardBuilder`] assembles a card in one fluent chain: `new(version)` fixes
//! the version once, then `prop::<L>()` opens a property segment keyed by a
//! lens marker, `param` decorates it, and `value` closes it and returns to the
//! card level.
//!
//! Since `value` yields the card builder and `prop` consumes it, the phases are
//! enforced by the type system: a `param` before any `prop`, or a `build` with
//! a property left half-open, does not compile.
//!
//! [`build`](VcardBuilder::build) runs the same whole-card
//! [`Vcard::validate`](crate::vcard::Vcard::validate) as any other decoded
//! card and hands back the [`VcardValid`] proof, while
//! [`build_unchecked`](VcardBuilder::build_unchecked) skips the check.
//!
//! [`VcardPropBuilder`] is the single-property piece underneath, the write-side
//! counterpart of the lenses: keyed by the same zero-sized markers, it carries
//! the card version, accumulates parameters, and emits an open [`VcardProp`].
//!
//! Its name is pinned by the marker's [`VcardPropSpec`], and
//! [`build`](VcardPropBuilder::build) runs the shared per-property check
//! ([`validate_prop`](crate::validator)), so the value kind and every
//! known parameter must be allowed for the version.
//!
//! Unknown, extension parameters pass. To emit something the spec forbids,
//! construct the open [`VcardProp`] by hand. The version is a value the
//! builders carry, never a type parameter.
//!
//! ## Example
//!
//! ```rust
//! use vcard::builder::VcardBuilder;
//! use vcard::prop::{r#fn::FN, note::NOTE};
//! use vcard::param::VcardParam;
//! use vcard::value::VcardValue;
//! use vcard::value::text::VcardText;
//! use vcard::version::VcardVersion;
//! use std::borrow::Cow;
//!
//! let valid = VcardBuilder::new(VcardVersion::V4_0)
//!     .prop::<FN>()
//!     .param(VcardParam::Pref(Cow::Borrowed("1")))
//!     .value(VcardValue::Text(VcardText(Cow::Borrowed("John Doe"))))
//!     .prop::<NOTE>()
//!     .value(VcardValue::Text(VcardText(Cow::Borrowed("a note"))))
//!     .build()
//!     .expect("a conformant 4.0 card");
//!
//! assert_eq!(valid.properties.len(), 2);
//! ```

use core::marker::PhantomData;

use alloc::vec::Vec;

use crate::{
    param::VcardParam,
    prop::{VcardProp, VcardPropName, spec::VcardPropSpec},
    validator::{VcardValid, VcardValidateError, validate_prop},
    value::VcardValue,
    vcard::Vcard,
    version::VcardVersion,
};

/// A version-aware builder for a whole card, chaining property segments.
///
/// Start with [`new`](Self::new), open each property with
/// [`prop`](Self::prop), and finish with [`build`](Self::build). The public
/// fields also allow assembling one by hand or inspecting the accumulation.
pub struct VcardBuilder<'a> {
    /// The card version every property is built for.
    pub version: VcardVersion,
    /// The properties accumulated so far, in order.
    pub properties: Vec<VcardProp<'a>>,
}

impl<'a> VcardBuilder<'a> {
    /// Start an empty card builder for the given version.
    pub fn new(version: VcardVersion) -> Self {
        Self {
            version,
            properties: Vec::new(),
        }
    }

    /// Open a property segment keyed by its lens marker; its name is pinned by
    /// the marker's spec. Chain [`param`](VcardPropInProgress::param) then
    /// [`value`](VcardPropInProgress::value) to close it and return here.
    pub fn prop<L: VcardPropSpec>(self) -> VcardPropInProgress<'a, L> {
        let inner = VcardPropBuilder::new(self.version);
        VcardPropInProgress { card: self, inner }
    }

    /// Finish, checking the assembled card with the same
    /// [`Vcard::validate`](crate::vcard::Vcard::validate) as any decoded card
    /// and yielding the [`VcardValid`] proof (or every violation).
    pub fn build(self) -> Result<VcardValid<Vcard<'a>>, Vec<VcardValidateError>> {
        self.build_unchecked().validate()
    }

    /// Finish without validating, returning the open card (the escape hatch,
    /// mirroring building a [`VcardProp`] by hand).
    pub fn build_unchecked(self) -> Vcard<'a> {
        Vcard {
            version: self.version,
            properties: self.properties,
        }
    }
}

/// A property segment being built inside a [`VcardBuilder`] chain, keyed by its
/// lens marker.
///
/// [`value`](Self::value) closes the segment and returns the card builder, so
/// the marker is discharged at that point and the chain stays flat.
pub struct VcardPropInProgress<'a, L: VcardPropSpec> {
    card: VcardBuilder<'a>,
    inner: VcardPropBuilder<'a, L>,
}

impl<'a, L: VcardPropSpec> VcardPropInProgress<'a, L> {
    /// Add a parameter to the open property (checked by the final
    /// [`build`](VcardBuilder::build)).
    pub fn param(mut self, param: VcardParam<'a>) -> Self {
        self.inner = self.inner.param(param);
        self
    }

    /// Close the property with a value and return to the card builder. Its name
    /// is taken from the marker; the value and parameters are checked once, by
    /// the final [`build`](VcardBuilder::build).
    pub fn value(mut self, value: VcardValue<'a>) -> VcardBuilder<'a> {
        self.card.properties.push(VcardProp {
            name: VcardPropName::Kind(L::KIND),
            params: self.inner.params,
            value,
        });

        self.card
    }
}

/// A version-aware builder for one property, keyed by its lens marker.
pub struct VcardPropBuilder<'a, L: VcardPropSpec> {
    /// The card version the property is built for.
    pub version: VcardVersion,
    /// The parameters accumulated so far.
    pub params: Vec<VcardParam<'a>>,
    lens: PhantomData<L>,
}

impl<'a, L: VcardPropSpec> VcardPropBuilder<'a, L> {
    /// Start a builder for the given card version.
    pub fn new(version: VcardVersion) -> Self {
        Self {
            version,
            params: Vec::new(),
            lens: PhantomData,
        }
    }

    /// Add a parameter (validated against the spec on [`build`](Self::build)).
    pub fn param(mut self, param: VcardParam<'a>) -> Self {
        self.params.push(param);
        self
    }

    /// Finish with a value, emitting the property named by the spec.
    ///
    /// Runs the same per-property check as
    /// [`Vcard::validate`](crate::vcard::Vcard::validate): the value kind and
    /// every known parameter must be allowed for the version, extensions pass.
    pub fn build(self, value: VcardValue<'a>) -> Result<VcardProp<'a>, Vec<VcardValidateError>> {
        let prop = VcardProp {
            name: VcardPropName::Kind(L::KIND),
            params: self.params,
            value,
        };

        let mut errors = Vec::new();

        validate_prop(&prop, self.version, &mut errors);

        if errors.is_empty() {
            Ok(prop)
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, vec};

    use crate::{
        builder::VcardPropBuilder,
        param::VcardParam,
        prop::r#fn::FN,
        value::{VcardValue, text::VcardText, uri::VcardUri},
        version::VcardVersion,
    };

    #[test]
    fn builds_a_property_pinning_the_name_from_the_spec() {
        let prop = VcardPropBuilder::<FN>::new(VcardVersion::V4_0)
            .param(VcardParam::Pref(Cow::Borrowed("1")))
            .build(VcardValue::Text(VcardText(Cow::Borrowed("John"))))
            .expect("FN takes text with a PREF param");

        assert_eq!(&*prop.name, "FN");
        assert_eq!(prop.params, vec![VcardParam::Pref(Cow::Borrowed("1"))]);
        assert_eq!(
            prop.value,
            VcardValue::Text(VcardText(Cow::Borrowed("John"))),
        );
    }

    /// FN is text-only, so a URI value is rejected.
    #[test]
    fn refuses_a_value_kind_the_property_does_not_allow() {
        let built = VcardPropBuilder::<FN>::new(VcardVersion::V4_0)
            .build(VcardValue::Uri(VcardUri(Cow::Borrowed("x"))));

        assert!(built.is_err());
    }

    /// FN does not allow MEDIATYPE.
    #[test]
    fn refuses_a_param_the_property_does_not_allow() {
        let built = VcardPropBuilder::<FN>::new(VcardVersion::V4_0)
            .param(VcardParam::MediaType(Cow::Borrowed("text/plain")))
            .build(VcardValue::Text(VcardText(Cow::Borrowed("John"))));

        assert!(built.is_err());
    }

    #[test]
    fn card_builder_chains_props_and_validates() {
        use crate::{builder::VcardBuilder, prop::note::NOTE};

        let valid = VcardBuilder::new(VcardVersion::V4_0)
            .prop::<FN>()
            .param(VcardParam::Pref(Cow::Borrowed("1")))
            .value(VcardValue::Text(VcardText(Cow::Borrowed("John Doe"))))
            .prop::<NOTE>()
            .value(VcardValue::Text(VcardText(Cow::Borrowed("a note"))))
            .build()
            .expect("a conformant 4.0 card");

        assert_eq!(valid.properties.len(), 2);
        assert_eq!(&*valid.properties[0].name, "FN");
        assert_eq!(&*valid.properties[1].name, "NOTE");
    }

    /// FN does not allow MEDIATYPE, and value() stays infallible, so the error
    /// only surfaces from the final build().
    #[test]
    fn card_builder_defers_a_violation_to_build() {
        use crate::builder::VcardBuilder;

        let built = VcardBuilder::new(VcardVersion::V4_0)
            .prop::<FN>()
            .param(VcardParam::MediaType(Cow::Borrowed("text/plain")))
            .value(VcardValue::Text(VcardText(Cow::Borrowed("John"))))
            .build();

        assert!(built.is_err());
    }
}
