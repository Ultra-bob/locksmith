use crate::engine::DecoderEngine;

/// Public metadata shown in the UI.
pub struct DecoderInfo {
    pub id: &'static str,
    pub label: &'static str,
}

// Submodules are declared by the declare_decoders! macro below to minimize boilerplate.

/// Declare decoders, their submodules, and generate register_all, register_selected, and all_decoders_info.
macro_rules! declare_decoders {
    ( $( { module: $module:ident, id: $id:expr, label: $label:expr, ctor: $ctor:expr } ),+ $(,)? ) => {
        // Declare one file/module per decoder
        $( mod $module; )+

        /// Convenience to register all built-in decoders into a decoder engine.
        pub fn register_all(engine: &mut DecoderEngine) {
            $( engine.register($ctor); )+
        }

        /// Register only the selected decoders by their IDs.
        pub fn register_selected<'a, I>(engine: &mut DecoderEngine, selected: I)
        where
            I: IntoIterator<Item = &'a str>,
        {
            use std::collections::HashSet;
            let set: HashSet<&str> = selected.into_iter().collect();
            $(
                if set.contains($id) {
                    engine.register($ctor);
                }
            )+
        }

        /// Return user-facing metadata for all decoders.
        pub fn all_decoders_info() -> Vec<DecoderInfo> {
            vec![
                $( DecoderInfo { id: $id, label: $label }, )+
            ]
        }
    }
}

declare_decoders! {
    { module: base32, id: "base32", label: "Base32", ctor: base32::Base32Decoder },
    { module: base58, id: "base58", label: "Base58", ctor: base58::Base58Decoder },
    { module: base64, id: "base64", label: "Base64", ctor: base64::Base64Decoder },
    { module: binary, id: "binary", label: "Binary", ctor: binary::BinaryDecoder },
    { module: caesar, id: "caesar", label: "Caesar", ctor: caesar::CaesarDecoder },
    { module: hex, id: "hex", label: "Hex", ctor: hex::HexDecoder },
    { module: html_entity, id: "html_entity", label: "HTML entity", ctor: html_entity::HTMLEntityDecoder },
    { module: morse, id: "morse", label: "Morse", ctor: morse::MorseCodeDecoder::new() },
    { module: reverse, id: "reverse", label: "Reverse", ctor: reverse::ReverseDecoder },
    { module: url, id: "url", label: "URL", ctor: url::URLDecoder },
    { module: cipher312, id: "cipher312", label: "312 Cipher", ctor: cipher312::ThreeOneTwoCipher },
}
