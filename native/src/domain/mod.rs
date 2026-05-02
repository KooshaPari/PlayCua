//! Domain layer — pure business types with zero external crate imports.
//!
//! All types here depend only on `std` and `serde` (for serialization contracts).
//! No adapter libraries (xcap, enigo, windows-rs) may be imported here.

#![allow(unused_imports)]

pub mod analysis;
pub mod capture;
pub mod input;
pub mod process;
pub mod window;

pub use analysis::AnalysisError;
pub use capture::CaptureError;
pub use input::{InputError, Key, KeyAction, MouseAction, MouseButton, MouseEvent, ScrollDirection};
pub use process::{ProcessError, ProcessHandle, ProcessStatus};
pub use window::{WindowError, WindowFilter, WindowInfo};
