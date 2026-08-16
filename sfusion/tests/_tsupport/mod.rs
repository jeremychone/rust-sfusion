//! Test support utilities for integration tests.

#![allow(unused)] // For test support

// region:    --- Modules

mod asserts;

pub use asserts::*;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

// endregion: --- Modules
