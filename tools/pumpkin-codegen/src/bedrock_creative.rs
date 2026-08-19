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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreativeItemsJson {
    groups: Vec<CreativeGroupJson>,
    items: Vec<CreativeItemJson>,
}

#[derive(Deserialize)]
struct CreativeGroupJson {
    name: String,
    category: String,
    icon: CreativeIconJson,
}

#[derive(Deserialize)]
struct CreativeIconJson {
    id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreativeItemJson {
    id: String,
    group_id: u32,
    #[serde(default)]
    damage: u32,
}

pub fn build() -> TokenStream {
    let be_runtime_item_states: Vec<BedrockRuntimeItemState> = serde_json::from_str(
        &fs::read_to_string("../../assets/bedrock/runtime_item_states.json").unwrap(),
    )
    .expect("Failed to parse bedrock/runtime_item_states.json");

    let bedrock_items_map: HashMap<String, i16> = be_runtime_item_states
        .into_iter()
        .map(|item| (item.name, item.id))
        .collect();

    let json_path = "../../assets/bedrock/creative_items.json";
    if std::path::Path::new(json_path).exists() {
        let creative: CreativeItemsJson = serde_json::from_str(
            &fs::read_to_string(json_path).expect("Failed to read bedrock/creative_items.json"),
        )
        .expect("Failed to parse bedrock/creative_items.json");

        let groups = creative.groups.into_iter().map(|group| {
            let category = match group.category.as_str() {
                "construction" => 1,
                "nature" => 2,
                "equipment" => 3,
                "items" => 4,
                "itemCommandOnly" => 5,
                _ => 6,
            };
            let name = group.name;
            let icon_item_id = bedrock_items_map
                .get(&group.icon.id)
                .copied()
                .unwrap_or_default();
            quote! {
                CreativeGroup {
                    category: #category,
                    name: #name,
                    icon_item_id: #icon_item_id,
                    icon_item_aux_value: 0,
                }
            }
        });
        let entries = creative.items.into_iter().filter_map(|item| {
            let item_id = bedrock_items_map.get(&item.id).copied()?;
            let item_aux_value = item.damage;
            let group_index = item.group_id;
            Some(quote! {
                CreativeEntry {
                    item_id: #item_id,
                    item_aux_value: #item_aux_value,
                    group_index: #group_index,
                }
            })
        });
        return creative_tokens(groups.collect(), entries.collect());
    }

    let nbt_path = "../../assets/bedrock/creative_items.nbt";
    if !std::path::Path::new(nbt_path).exists() {
        let generated_path = "../../crates/pumpkin-data/src/generated/bedrock_creative.rs";
        if let Ok(content) = fs::read_to_string(generated_path) {
            return content.parse().unwrap_or_else(|_| TokenStream::new());
        }
        return TokenStream::new();
    }

    let nbt_bytes = fs::read(nbt_path).expect("Failed to read bedrock/creative_items.nbt");
    let mut cursor = Cursor::new(nbt_bytes);

    let mut reader = pumpkin_nbt::deserializer::NbtReadHelperBedrock::new(
        pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
    );
    let nbt = pumpkin_nbt::Nbt::read(&mut reader).expect("Failed to read creative_items.nbt");

    let mut group_tokens = Vec::new();
    if let Some(pumpkin_nbt::tag::NbtTag::List(groups_list)) = nbt.get("groups") {
        for g_tag in groups_list {
            if let pumpkin_nbt::tag::NbtTag::Compound(g) = g_tag {
                let category = g.get_int("category").unwrap_or(0);
                let name = g.get_string("name").unwrap_or_default().to_string();
                let mut icon_item_id = 0i16;
                let mut icon_item_aux_value = 0u32;

                if let Some(pumpkin_nbt::tag::NbtTag::Compound(icon)) = g.get("icon") {
                    if let Some(icon_name) = icon.get_string("name") {
                        if let Some(&id) = bedrock_items_map.get(icon_name) {
                            icon_item_id = id;
                            icon_item_aux_value = icon.get_short("meta").unwrap_or(0) as u32;
                        }
                    }
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
        }
    }

    let mut entry_tokens = Vec::new();
    if let Some(pumpkin_nbt::tag::NbtTag::List(items_list)) = nbt.get("items") {
        for item_tag in items_list {
            if let pumpkin_nbt::tag::NbtTag::Compound(item) = item_tag {
                if let Some(item_name) = item.get_string("name") {
                    if let Some(&id) = bedrock_items_map.get(item_name) {
                        let item_aux_value = item.get_short("meta").unwrap_or(0) as u32;
                        let group_index = item.get_int("group_index").unwrap_or(0) as u32;

                        entry_tokens.push(quote! {
                            CreativeEntry {
                                item_id: #id,
                                item_aux_value: #item_aux_value,
                                group_index: #group_index,
                            }
                        });
                    }
                }
            }
        }
    }

    creative_tokens(group_tokens, entry_tokens)
}

fn creative_tokens(group_tokens: Vec<TokenStream>, entry_tokens: Vec<TokenStream>) -> TokenStream {
    let groups_len = group_tokens.len();
    let entries_len = entry_tokens.len();
    quote! {
        #[derive(Clone, Copy)]
        pub struct CreativeGroup {
            pub category: i32,
            pub name: &'static str,
            pub icon_item_id: i16,
            pub icon_item_aux_value: u32,
        }

        #[derive(Clone, Copy)]
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
