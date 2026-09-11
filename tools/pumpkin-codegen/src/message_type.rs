use std::{collections::BTreeMap, fs};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

/// Raw deserialization shape for a single chat type entry from `message_type.json`.
#[derive(Deserialize)]
pub struct RawChatType {
    /// Numeric ID assigned to this chat type in the vanilla registry.
    id: u32,
    //    components: ChatType,
}

// #[derive(Deserialize)]
// pub struct ChatType {
//     chat: Decoration,
//     narration: Decoration,
// }

// #[derive(Deserialize)]
// pub struct Decoration {
//     translation_key: String,
//     #[serde(default, skip_serializing_if = "Option::is_none")]
//     style: Option<Style>,
//     parameters: Vec<String>,
// }

/// Generates the `TokenStream` for message type `u8` constants from 26.2 datapack, including a synthetic `RAW` variant.
pub fn build() -> TokenStream {
    let dir = std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/chat_type");
    let mut entries: Vec<_> = fs::read_dir(dir)
        .expect("Missing chat_type directory")
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by_key(|e| e.path());

    let mut variants = TokenStream::new();

    for (i, entry) in entries.iter().enumerate() {
        let stem = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let i = i as u8;
        let name = format_ident!("{}", stem.to_uppercase());
        variants.extend([quote! {
            pub const #name: u8 = #i;
        }]);
    }

    let raw_id = entries.len() as u8;
    variants.extend([quote! {
        pub const RAW: u8 = #raw_id; // One higher than the highest vanilla id
    }]);

    quote! {
        #variants
    }
}
