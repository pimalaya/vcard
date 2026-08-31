//! # CLIENTPIDMAP
//!
//! The `CLIENTPIDMAP` property, mapping a PID source identifier to the URI of
//! the client that produced it.
//!
//! See RFC 6350 6.7.7.

use alloc::string::{String, ToString};

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::{VcardValue, VcardValueKind},
    version::VcardVersion,
};

/// The `CLIENTPIDMAP` property marker.
pub struct CLIENTPIDMAP;

impl VcardPropSpec for CLIENTPIDMAP {
    const KIND: VcardPropKind = VcardPropKind::ClientPidMap;

    fn allowed_versions() -> &'static [VcardVersion] {
        &[VcardVersion::V4_0]
    }

    fn allowed_values(_version: VcardVersion) -> &'static [VcardValueKind] {
        &[VcardValueKind::ClientPidMap]
    }

    fn allowed_params(_version: VcardVersion) -> &'static [VcardParamKind] {
        &[]
    }

    /// RFC 6350 6.7.7: the first field is "a small integer", the source
    /// identifier a `PID` parameter names. The second is a URI, whose
    /// syntax this does not read: that is format validation, and a
    /// different appetite.
    fn invalid_value(value: &VcardValue<'_>, _version: VcardVersion) -> Option<String> {
        let VcardValue::ClientPidMap(map) = value else {
            return None;
        };

        let id = map.id.as_ref();
        let positive = !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit());

        (!positive).then(|| id.to_string())
    }
}
