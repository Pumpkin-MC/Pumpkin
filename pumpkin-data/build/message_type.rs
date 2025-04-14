use std::collections::HashMap;

use proc_macro2::TokenStream;
use pumpkin_util::text::style::Style;
use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawChatType {
    id: u32,
    components: ChatType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatType {
    chat: Decoration,
    narration: Decoration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decoration {
    translation_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    style: Option<Style>,
    parameters: Vec<String>,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/message_type.json");

    let json: HashMap<String, RawChatType> =
        serde_json::from_str(include_str!("../../assets/message_type.json"))
            .expect("Failed to parse message_type.json");
    let mut variants = TokenStream::new();

    for (name, typee) in json.iter() {
        let i = typee.id as u8;
        let name = format_ident!("{}", name.to_uppercase());
        variants.extend([quote! {
            pub const #name: u8 = #i;
        }]);
    }

    let raw_id = json.len() as u8;
    variants.extend([quote! {
        pub const RAW: u8 = #raw_id; // One higher than highest vanilla id
    }]);

    quote! {
        #variants
    }
}
