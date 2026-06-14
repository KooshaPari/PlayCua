//! Concrete wire adapter for the Sandbox port.
//!
//! Re-exports the production-ready [`WireSandboxAdapter`] from the port
//! module so the adapters layer is a single discovery point.

#![allow(unused_imports)]

pub use crate::ports::sandbox::WireSandboxAdapter;
