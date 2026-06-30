//! # Property builder
//!
//! Strict, version-aware construction of a single property.
//!
//! [`VcardPropBuilder`] is the write-side counterpart of the lenses: keyed by
//! the same zero-sized markers, it carries the card version and accumulates
//! parameters, then emits an open [`VcardProp`]. Construction is the strict
//! half of "liberal in, strict out": the name is pinned by the marker's
//! [`VcardPropSpec`], and [`build`](VcardPropBuilder::build) runs the shared
//! per-property check ([`validate_prop`](crate::tree::validate)) so the value
//! kind and every known parameter must be allowed for the version (unknown,
//! extension parameters pass). To emit something the spec forbids, construct
//! the open [`VcardProp`] by hand. The version is a value the builder carries,
//! never a type parameter.

use core::marker::PhantomData;

use alloc::vec::Vec;

use crate::{
    param::VcardParam,
    prop::{VcardProp, VcardPropName},
    tree::{
        prop::VcardPropSpec,
        validate::{VcardValidateError, validate_prop},
    },
    value::VcardValue,
    version::VcardVersion,
};

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

    /// Finish with a value, emitting the property; its name is taken from the
    /// spec. Runs the same per-property check as
    /// [`Vcard::validate`](crate::vcard::Vcard::validate): the value kind must
    /// be allowed and every known parameter must be allowed for the version
    /// (unknown, i.e. extension, parameters pass).
    pub fn build(self, value: VcardValue<'a>) -> Result<VcardProp<'a>, Vec<VcardValidateError>> {
        let prop = VcardProp {
            name: VcardPropName::Kind(L::PROP),
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
        param::VcardParam,
        tree::{build::VcardPropBuilder, prop::r#fn::FN},
        value::{VcardValue, text::VcardText},
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

    #[test]
    fn refuses_a_value_kind_the_property_does_not_allow() {
        // FN is text-only, so a URI value is rejected.
        let built = VcardPropBuilder::<FN>::new(VcardVersion::V4_0).build(VcardValue::Uri(
            crate::value::uri::VcardUri(Cow::Borrowed("x")),
        ));

        assert!(built.is_err());
    }

    #[test]
    fn refuses_a_param_the_property_does_not_allow() {
        // FN does not allow MEDIATYPE.
        let built = VcardPropBuilder::<FN>::new(VcardVersion::V4_0)
            .param(VcardParam::MediaType(Cow::Borrowed("text/plain")))
            .build(VcardValue::Text(VcardText(Cow::Borrowed("John"))));

        assert!(built.is_err());
    }
}
