use std::{collections::HashMap, fs, io::Cursor};

use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Deserialize_repr, Debug)]
#[repr(i32)]
enum BedrockItemVersion {
    Legacy = 0,
    DataDriven = 1,
    None = 2,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct BedrockRuntimeItemState {
    name: String,
    id: i16,
    version: BedrockItemVersion,
    component_based: bool,
}

pub fn build() -> TokenStream {
    let be_runtime_item_states: Vec<BedrockRuntimeItemState> = serde_json::from_str(
        &fs::read_to_string("../assets/bedrock/runtime_item_states.json").unwrap(),
    )
    .expect("Failed to parse bedrock/runtime_item_states.json");

    let bedrock_items_map: HashMap<String, i16> = be_runtime_item_states
        .into_iter()
        .map(|item| (item.name, item.id))
        .collect();

    let nbt_path = "../assets/bedrock/creative_items.nbt";
    if !std::path::Path::new(nbt_path).exists() {
        let generated_path = "../pumpkin-data/src/generated/bedrock_creative.rs";
        if let Ok(content) = fs::read_to_string(generated_path) {
            return content.parse().unwrap_or_else(|_| TokenStream::new());
        }
        return TokenStream::new();
    }

    let nbt_bytes = fs::read(nbt_path).expect("Failed to read bedrock/creative_items.nbt");
    let mut cursor = Cursor::new(nbt_bytes);

    #[derive(serde::Deserialize)]
    struct CreativeGroupEntry {
        category: i32,
        name: String,
        icon: Option<CreativeItemEntry>,
    }

    #[derive(serde::Deserialize)]
    struct CreativeItemEntry {
        name: String,
        meta: Option<i16>,
        group_index: Option<i32>,
    }

    #[derive(serde::Deserialize)]
    struct CreativeData {
        groups: Vec<CreativeGroupEntry>,
        items: Vec<CreativeItemEntry>,
    }

    let data: CreativeData = pumpkin_nbt::deserializer::from_bytes_bedrock(&mut cursor)
        .expect("Failed to deserialize creative_items.nbt");

    let mut group_tokens = Vec::new();
    for g in &data.groups {
        let category = g.category;
        let name = &g.name;
        let mut icon_item_id = 0i16;
        let mut icon_item_aux_value = 0u32;

        if let Some(ref icon) = g.icon
            && let Some(&id) = bedrock_items_map.get(icon.name.as_str())
        {
            icon_item_id = id;
            icon_item_aux_value = icon.meta.unwrap_or(0) as u32;
        }

        group_tokens.push(quote! {
            CreativeGroup {
                category: #category,
                name: #name,
                icon_item_id: #icon_item_id,
                icon_item_aux_value: #icon_item_aux_value,
            }
        });
    }

    // 4. Build entry definitions TokenStream
    let mut entry_tokens = Vec::new();
    for item in &data.items {
        if let Some(&id) = bedrock_items_map.get(item.name.as_str()) {
            let item_aux_value = item.meta.unwrap_or(0) as u32;
            let group_index = item.group_index.unwrap_or(0) as u32;

            entry_tokens.push(quote! {
                CreativeEntry {
                    item_id: #id,
                    item_aux_value: #item_aux_value,
                    group_index: #group_index,
                }
            });
        }
    }

    let groups_len = group_tokens.len();
    let entries_len = entry_tokens.len();

    quote! {
        #[derive(Clone, Copy, Debug)]
        pub struct CreativeGroup {
            pub category: i32,
            pub name: &'static str,
            pub icon_item_id: i16,
            pub icon_item_aux_value: u32,
        }

        #[derive(Clone, Copy, Debug)]
        pub struct CreativeEntry {
            pub item_id: i16,
            pub item_aux_value: u32,
            pub group_index: u32,
        }

        pub const CREATIVE_GROUPS: &[CreativeGroup; #groups_len] = &[
            #(#group_tokens),*
        ];

        pub const CREATIVE_ENTRIES: &[CreativeEntry; #entries_len] = &[
            #(#entry_tokens),*
        ];
    }
}
