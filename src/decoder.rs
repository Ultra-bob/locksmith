#![allow(deprecated)]

//! Deprecated module: the decoding system has been refactored.
//!
//! Use `crate::engine` for core decoding traits and execution,
//! and `crate::decoders` for concrete decoder implementations.
//!
//! This file is kept as a thin compatibility shim to avoid breaking imports.
//! Prefer migrating your code to:
//! - `use crate::engine::{Decoder, DecoderEngine, Step, TransformResult, Policy};`
//! - `use crate::decoders::{CaesarDecoder, Base64Decoder, register_all};`

#[deprecated(
    since = "0.1.0",
    note = "Use crate::engine and crate::decoders instead of crate::decoder"
)]
pub mod compat {
    pub use crate::decoders::{Base64Decoder, CaesarDecoder, register_all};
    pub use crate::engine::{Decoder, DecoderEngine, Policy, Step, TransformResult};
}

// Re-export for convenience: `use crate::decoder::*;`
pub use compat::*;
