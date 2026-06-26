//! vCard extensions for expertise, hobbies and interests (RFC 6715).
//!
//! The LEVEL and INDEX parameters these properties introduce ride along as
//! ordinary [`crate::rfc6350::param::parameter::VcardParameter`] entries in the `params`
//! list.

pub mod expertise;
pub mod hobby;
pub mod interest;
pub mod org_directory;
pub mod param;
