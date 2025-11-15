#![allow(clippy::module_name_repetitions)]

/*!
Decoders module

This module organizes each decoder into its own submodule and re-exports the
decoder types for ergonomic use:

- use crate::decoders::Base64Decoder;
- use crate::decoders::register_all;
- use crate::decoders::register_selected;
- use crate::decoders::all_decoders_info;

To add a new decoder:
1. Create a new file in this directory (e.g., `rot47.rs`) that implements `crate::engine::Decoder`.
2. Add `mod rot47;` below and `pub use rot47::Rot47Decoder;`.
3. Update `register_all`, `all_decoders_info`, and `register_selected` as needed.
*/

use crate::engine::DecoderEngine;

// Submodules (one file per decoder)
mod base32;
mod base58;
mod base64;
mod binary;
mod caesar;
mod hex;
mod html_entity;
mod morse;
mod reverse;
mod url;

// Re-exports so callers can `use crate::decoders::XxxDecoder`.
pub use base32::Base32Decoder;
pub use base58::Base58Decoder;
pub use base64::Base64Decoder;
pub use binary::BinaryDecoder;
pub use caesar::CaesarDecoder;
pub use hex::HexDecoder;
pub use html_entity::HTMLEntityDecoder;
pub use morse::MorseCodeDecoder;
pub use reverse::ReverseDecoder;
pub use url::URLDecoder;

/// Metadata for available decoders (for UI selection and display).
#[derive(Clone, Debug)]
pub struct DecoderInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub group: &'static str,
}

/// Returns static metadata for all built-in decoders.
pub fn all_decoders_info() -> &'static [DecoderInfo] {
    // Keep this list in sync with `register_all` and `register_selected`.
    static INFOS: [DecoderInfo; 10] = [
        DecoderInfo {
            id: "caesar",
            label: "Caesar",
            group: "shift",
        },
        DecoderInfo {
            id: "binary",
            label: "Binary to ASCII",
            group: "binary",
        },
        DecoderInfo {
            id: "reverse",
            label: "Reverse",
            group: "reverse",
        },
        DecoderInfo {
            id: "morse",
            label: "Morse code",
            group: "morse",
        },
        DecoderInfo {
            id: "hex",
            label: "Hex",
            group: "hex",
        },
        DecoderInfo {
            id: "html_entity",
            label: "HTML entity",
            group: "html",
        },
        DecoderInfo {
            id: "base32",
            label: "Base32",
            group: "base",
        },
        DecoderInfo {
            id: "base64",
            label: "Base64",
            group: "base",
        },
        DecoderInfo {
            id: "base58",
            label: "Base58",
            group: "base",
        },
        DecoderInfo {
            id: "url",
            label: "URL",
            group: "url",
        },
    ];
    &INFOS
}

/// Convenience to register all built-in decoders into a decoder engine.
pub fn register_all(engine: &mut DecoderEngine) {
    engine.register(CaesarDecoder);
    engine.register(BinaryDecoder);
    engine.register(ReverseDecoder);
    engine.register(MorseCodeDecoder::new());
    engine.register(HexDecoder);
    engine.register(HTMLEntityDecoder);
    engine.register(Base32Decoder);
    engine.register(Base64Decoder);
    engine.register(Base58Decoder);
    engine.register(URLDecoder);
}

/// Register only the selected decoders by their IDs.
pub fn register_selected<'a, I>(engine: &mut DecoderEngine, selected: I)
where
    I: IntoIterator<Item = &'a str>,
{
    use std::collections::HashSet;
    let set: HashSet<&str> = selected.into_iter().collect();

    if set.contains("caesar") {
        engine.register(CaesarDecoder);
    }
    if set.contains("binary") {
        engine.register(BinaryDecoder);
    }
    if set.contains("reverse") {
        engine.register(ReverseDecoder);
    }
    if set.contains("morse") {
        engine.register(MorseCodeDecoder::new());
    }
    if set.contains("hex") {
        engine.register(HexDecoder);
    }
    if set.contains("html_entity") {
        engine.register(HTMLEntityDecoder);
    }
    if set.contains("base32") {
        engine.register(Base32Decoder);
    }
    if set.contains("base64") {
        engine.register(Base64Decoder);
    }
    if set.contains("base58") {
        engine.register(Base58Decoder);
    }
    if set.contains("url") {
        engine.register(URLDecoder);
    }
}
