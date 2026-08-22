use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
};

use crate::block::BlockAssets;
use crate::entity_type::EntityType;
use crate::fluid::Fluid;
use crate::item::Item;
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

// Builder that generates an enum with `from_string` and `identifier_string` methods.
pub struct EnumCreator {
    /// Name of the enum to generate (converted to PascalCase).
    pub name: String,
    /// Set of variant names (converted to PascalCase for the enum variants).
    pub values: BTreeSet<String>,
}

impl ToTokens for EnumCreator {
    /// Emits the enum definition and its `from_string`/`identifier_string` impl block.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.name.to_pascal_case());

        let variants = self.values.iter().map(|v| {
            let variant_name = format_ident!("{}", v.to_pascal_case());
            quote! { #variant_name }
        });

        let all_variants = self.values.iter().map(|v| {
            let variant_name = format_ident!("{}", v.to_pascal_case());
            quote! { Self::#variant_name }
        });

        let is_network_synced_cat = |v: &str| -> bool {
            if v == "villager_trade" || v == "trade_set" {
                return false;
            }
            if v.starts_with("worldgen/") && v != "worldgen/biome" {
                return false;
            }
            true
        };

        let network_variants = self
            .values
            .iter()
            .filter(|v| is_network_synced_cat(v))
            .map(|v| {
                let variant_name = format_ident!("{}", v.to_pascal_case());
                quote! { Self::#variant_name }
            });

        let unsynced_arms = self
            .values
            .iter()
            .filter(|v| !is_network_synced_cat(v))
            .map(|v| {
                let variant_name = format_ident!("{}", v.to_pascal_case());
                quote! { Self::#variant_name => false }
            });

        let from_string_arms = self.values.iter().map(|v| {
            let variant_name = format_ident!("{}", v.to_pascal_case());
            quote! { #v => Some(Self::#variant_name) }
        });

        let to_string_arms = self.values.iter().map(|v| {
            let variant_name = format_ident!("{}", v.to_pascal_case());
            quote! { Self::#variant_name => #v }
        });

        tokens.extend(quote! {
            #[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
            pub enum #name {
                #(#variants),*
            }

            impl #name {
                pub const ALL: &[Self] = &[
                    #(#all_variants),*
                ];

                pub const NETWORK_KEYS: &[Self] = &[
                    #(#network_variants),*
                ];

                #[must_use]
                pub const fn is_network_synced(&self) -> bool {
                    match self {
                        #(#unsynced_arms,)*
                        _ => true,
                    }
                }

                #[must_use]
                pub fn from_string(s: &str) -> Option<Self> {
                    match s {
                        #(#from_string_arms,)*
                        _ => None,
                    }
                }

                #[must_use]
                pub const fn identifier_string(&self) -> &str {
                    match self {
                        #(#to_string_arms),*
                    }
                }
            }
        });
    }
}

fn load_datapack_registry_ids(dir: &std::path::Path) -> BTreeMap<String, u16> {
    let mut id_map = BTreeMap::new();
    if dir.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .collect();
        entries.sort_by_key(|e| e.path());
        for (i, entry) in entries.iter().enumerate() {
            let stem = entry
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            id_map.insert(format!("minecraft:{stem}"), i as u16);
            id_map.insert(stem, i as u16);
        }
    }
    id_map
}

fn load_datapack_tags(
    data_dir: &std::path::Path,
) -> BTreeMap<String, BTreeMap<String, Vec<String>>> {
    let mut raw_categories: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();

    fn walk_namespace_tags(
        dir: &std::path::Path,
        tags_dir: &std::path::Path,
        namespace: &str,
        raw: &mut BTreeMap<String, BTreeMap<String, Vec<String>>>,
    ) {
        if let Ok(entries) = fs::read_dir(dir) {
            let mut entries: Vec<_> = entries.flatten().collect();
            entries.sort_by_key(|e| e.path());
            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    walk_namespace_tags(&path, tags_dir, namespace, raw);
                } else if path.extension().is_some_and(|ext| ext == "json") {
                    let rel = path.strip_prefix(tags_dir).unwrap();
                    let components: Vec<_> = rel
                        .iter()
                        .map(|c| c.to_string_lossy().into_owned())
                        .collect();
                    if components.is_empty() {
                        continue;
                    }
                    let (category, tag_rel_path) =
                        if components[0] == "worldgen" && components.len() >= 2 {
                            let cat = format!("worldgen/{}", components[1]);
                            let rest = components[2..].join("/");
                            let tag_stem = rest.trim_end_matches(".json").to_string();
                            (cat, tag_stem)
                        } else {
                            let cat = components[0].clone();
                            let rest = components[1..].join("/");
                            let tag_stem = rest.trim_end_matches(".json").to_string();
                            (cat, tag_stem)
                        };

                    let tag_name = format!("{namespace}:{tag_rel_path}");
                    if let Ok(content) = fs::read_to_string(&path) {
                        #[derive(serde::Deserialize)]
                        struct TagFile {
                            values: Vec<serde_json::Value>,
                        }
                        if let Ok(tag_file) = serde_json::from_str::<TagFile>(&content) {
                            let mut values = Vec::new();
                            for val in tag_file.values {
                                if let Some(s) = val.as_str() {
                                    values.push(s.to_string());
                                } else if let Some(id) = val.get("id").and_then(|i| i.as_str()) {
                                    values.push(id.to_string());
                                }
                            }
                            raw.entry(category).or_default().insert(tag_name, values);
                        }
                    }
                }
            }
        }
    }

    if let Ok(entries) = fs::read_dir(data_dir) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let namespace = entry.file_name().to_string_lossy().into_owned();
            let tags_dir = entry.path().join("tags");
            if tags_dir.is_dir() {
                walk_namespace_tags(&tags_dir, &tags_dir, &namespace, &mut raw_categories);
            }
        }
    }

    // Recursively resolve references for each category
    let mut resolved_categories: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();

    for (cat, tag_map) in &raw_categories {
        let mut cat_resolved = BTreeMap::new();

        fn resolve_tag(
            tag_name: &str,
            tag_map: &BTreeMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
        ) -> Vec<String> {
            if visited.contains(tag_name) {
                return Vec::new();
            }
            visited.insert(tag_name.to_string());
            let mut result = Vec::new();
            if let Some(raw_values) = tag_map.get(tag_name) {
                for v in raw_values {
                    if let Some(sub_tag) = v.strip_prefix('#') {
                        let full_sub = if sub_tag.contains(':') {
                            sub_tag.to_string()
                        } else {
                            format!("minecraft:{sub_tag}")
                        };
                        let mut sub_visited = visited.clone();
                        for child in resolve_tag(&full_sub, tag_map, &mut sub_visited) {
                            if !result.contains(&child) {
                                result.push(child);
                            }
                        }
                    } else {
                        let clean = v.strip_prefix("minecraft:").unwrap_or(v).to_string();
                        if !result.contains(&clean) {
                            result.push(clean);
                        }
                    }
                }
            }
            result
        }

        for tag_name in tag_map.keys() {
            let mut visited = HashSet::new();
            let resolved = resolve_tag(tag_name, tag_map, &mut visited);
            cat_resolved.insert(tag_name.clone(), resolved);
        }

        resolved_categories.insert(cat.clone(), cat_resolved);
    }

    resolved_categories
}

/// Generates the `TokenStream` for the `Tag` type, `RegistryKey` enum, tag
/// modules, and the `Taggable` trait with its lookup helpers.
pub(crate) fn build() -> TokenStream {
    // --- Load Global Assets ---
    let blocks_assets: BlockAssets =
        serde_json::from_str(&fs::read_to_string("../../assets/blocks.json").unwrap())
            .expect("Failed to parse blocks.json");
    let mut block_id_map: BTreeMap<String, u16> = BTreeMap::new();
    for b in &blocks_assets.blocks {
        block_id_map.insert(b.name.clone(), b.id.0);
        block_id_map.insert(format!("minecraft:{}", b.name), b.id.0);
    }

    let items: BTreeMap<String, Item> =
        serde_json::from_str(&fs::read_to_string("../../assets/items.json").unwrap())
            .expect("Failed to parse items.json");
    let mut item_id_map: BTreeMap<String, u16> = BTreeMap::new();
    for (name, item) in &items {
        item_id_map.insert(name.clone(), item.id);
        item_id_map.insert(format!("minecraft:{name}"), item.id);
    }

    let fluids: Vec<Fluid> =
        serde_json::from_str(&fs::read_to_string("../../assets/fluids.json").unwrap())
            .expect("Failed to parse fluids.json");
    let mut fluid_id_map: BTreeMap<String, u16> = BTreeMap::new();
    for f in &fluids {
        fluid_id_map.insert(f.name.clone(), f.id);
        fluid_id_map.insert(format!("minecraft:{}", f.name), f.id);
    }

    let entities: BTreeMap<String, EntityType> =
        serde_json::from_str(&fs::read_to_string("../../assets/entities.json").unwrap())
            .expect("Failed to parse entities.json");
    let mut entity_id_map: BTreeMap<String, u16> = BTreeMap::new();
    for (name, entity) in &entities {
        entity_id_map.insert(name.clone(), entity.id);
        entity_id_map.insert(format!("minecraft:{name}"), entity.id);
    }

    let game_events: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../../assets/game_event.json").unwrap())
            .expect("Failed to parse game_event.json");
    let mut game_event_id_map: BTreeMap<String, u16> = BTreeMap::new();
    for (i, name) in game_events.iter().enumerate() {
        game_event_id_map.insert(name.clone(), i as u16);
        game_event_id_map.insert(format!("minecraft:{name}"), i as u16);
    }

    #[derive(serde::Deserialize)]
    struct PotionEntry {
        id: u8,
    }
    let potions: BTreeMap<String, PotionEntry> =
        serde_json::from_str(&fs::read_to_string("../../assets/potion.json").unwrap())
            .expect("Failed to parse potion.json");
    let mut potion_id_map: BTreeMap<String, u16> = BTreeMap::new();
    for (name, potion) in &potions {
        potion_id_map.insert(name.clone(), u16::from(potion.id));
        potion_id_map.insert(format!("minecraft:{name}"), u16::from(potion.id));
    }

    const POI_TYPES: &[&str] = &[
        "armorer",
        "butcher",
        "cartographer",
        "cleric",
        "farmer",
        "fisherman",
        "fletcher",
        "leatherworker",
        "librarian",
        "mason",
        "shepherd",
        "toolsmith",
        "weaponsmith",
        "home",
        "meeting",
        "beehive",
        "bee_nest",
        "nether_portal",
        "lodestone",
        "lightning_rod",
        "trial_spawner",
        "vault",
    ];
    let mut poi_id_map: BTreeMap<String, u16> = BTreeMap::new();
    for (i, &name) in POI_TYPES.iter().enumerate() {
        poi_id_map.insert(name.to_string(), i as u16);
        poi_id_map.insert(format!("minecraft:{name}"), i as u16);
    }

    let datapack_data_dir = std::path::Path::new("../../assets/datapacks/26_2/data");
    let datapack_base = datapack_data_dir.join("minecraft");
    let mut datapack_id_maps: BTreeMap<String, BTreeMap<String, u16>> = BTreeMap::new();

    let mut all_registry_keys = HashSet::new();
    all_registry_keys.insert("dimension_type".to_string());

    let tags = load_datapack_tags(datapack_data_dir);

    let mut tag_dicts = Vec::new();
    let mut match_local_map = Vec::new();

    for (key, tag_map) in tags {
        all_registry_keys.insert(key.clone());
        let key_pascal = format_ident!("{}", key.to_pascal_case());
        let dict_name = format_ident!("{}_TAGS", key.to_pascal_case().to_uppercase());

        let mut tag_entries = Vec::new();
        let mut tag_map_entries = Vec::new();

        if !datapack_id_maps.contains_key(&key) {
            let dir = datapack_base.join(&key);
            if dir.is_dir() {
                datapack_id_maps.insert(key.clone(), load_datapack_registry_ids(&dir));
            }
        }

        for (tag_name, values) in tag_map {
            let ids: Vec<u16> = values
                .iter()
                .filter_map(|v| match key.as_str() {
                    "block" => block_id_map.get(v).copied(),
                    "item" => item_id_map.get(v).copied(),
                    "fluid" => fluid_id_map.get(v).copied(),
                    "entity_type" => entity_id_map.get(v).copied(),
                    "game_event" => game_event_id_map.get(v).copied(),
                    "potion" => potion_id_map.get(v).copied(),
                    "point_of_interest_type" => poi_id_map.get(v).copied(),
                    _ => datapack_id_maps.get(&key).and_then(|m| m.get(v).copied()),
                })
                .collect();

            let tag_const_name = format_ident!(
                "{}",
                tag_name.replace([':', '/', '.', '-'], "_").to_uppercase()
            );

            tag_entries.push(quote! {
                pub const #tag_const_name: Tag = (&[#(#values),*], &[#(#ids),*]);
            });
            tag_map_entries.push(quote! { #tag_name => &#key_pascal::#tag_const_name });
        }

        tag_dicts.push(quote! {
            #[allow(non_snake_case)]
            pub mod #key_pascal {
                use super::Tag;
                #(#tag_entries)*
            }
            static #dict_name: phf::Map<&'static str, &'static Tag> = phf::phf_map! {
                #(#tag_map_entries),*
            };
        });

        match_local_map.push(quote! { RegistryKey::#key_pascal => Some(&#dict_name) });
    }

    // --- Generate RegistryKey Enum ---
    let registry_key_enum = EnumCreator {
        name: "RegistryKey".to_string(),
        values: all_registry_keys.into_iter().collect(),
    }
    .to_token_stream();

    quote! {
        use pumpkin_util::version::JavaMinecraftVersion;

        pub type Tag = (&'static [&'static str], &'static [u16]);

        #registry_key_enum

        #(#tag_dicts)*

        #[allow(unreachable_patterns)]
        #[must_use]
        pub const fn get_latest_map(key: RegistryKey) -> Option<&'static phf::Map<&'static str, &'static Tag>> {
            match key {
                #(#match_local_map,)*
                _ => None,
            }
        }

        #[must_use]
        pub fn get_tag_values(tag_category: RegistryKey, tag: &str) -> Option<&'static [&'static str]> {
            get_latest_map(tag_category).and_then(|m| m.get(tag)).map(|t| t.0)
        }

        #[must_use]
        pub fn get_tag_ids(tag_category: RegistryKey, tag: &str) -> Option<&'static [u16]> {
            get_latest_map(tag_category).and_then(|m| m.get(tag)).map(|t| t.1)
        }

        #[must_use]
        pub const fn get_registry_key_tags(_version: JavaMinecraftVersion, tag_category: RegistryKey) -> Option<&'static phf::Map<&'static str, &'static Tag>> {
            get_latest_map(tag_category)
        }

        pub trait Taggable {
            fn tag_key() -> RegistryKey;
            fn registry_key(&self) -> &str;
            fn registry_id(&self) -> u16;

           #[must_use]
           fn is_tagged_with(&self, tag: &str) -> Option<bool> {
                let tag = tag.strip_prefix("#").unwrap_or(tag);
                let items = get_tag_ids(Self::tag_key(), tag)?;
                Some(items.contains(&self.registry_id()))
            }

            #[must_use]
            fn has_tag(&self, tag: &'static Tag) -> bool {
                tag.1.contains(&self.registry_id())
            }

            #[must_use]
            fn get_tag_values(tag: &str) -> Option<&'static [&'static str]> {
                let tag = tag.strip_prefix("#").unwrap_or(tag);
                get_tag_values(Self::tag_key(), tag)
            }
        }
    }
}
