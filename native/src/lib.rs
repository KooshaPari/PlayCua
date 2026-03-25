//! bare-cua-native library entry point.
//!
//! Re-exports all public modules so integration tests and external crates
//! can import domain types, port traits, and adapter implementations.

pub mod adapters;
pub mod app;
pub mod domain;
pub mod ipc;
pub mod plugins;
pub mod ports;
