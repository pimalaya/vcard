//! # CLIENTPIDMAP
//!
//! The `CLIENTPIDMAP` property, mapping a PID source identifier to the URI of
//! the client that produced it.
//!
//! See RFC 6350 6.7.7.

use crate::{
    param::VcardParamKind,
    prop::{VcardPropKind, spec::VcardPropSpec},
    value::VcardValueKind,
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
}
