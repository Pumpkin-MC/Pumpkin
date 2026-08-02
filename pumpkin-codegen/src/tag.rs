use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
};

use crate::block::BlockAssets;
use crate::enchantments::Enchantment;
use crate::entity_type::EntityType;
use crate::fluid::Fluid;
use crate::item::Item;
use crate::{biome::Biome, version::JavaMinecraftVersion};
use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

/// Builder that generates an enum with `from_string` and `identifier_string` methods.
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

fn make_doc(strs: &[String]) -> String {
    let mut doc = "Contains: `[ ".to_string();
    strs.iter().for_each(|s| {
        doc.push('"');
        doc.push_str(s);
        doc.push_str("\", ")
    });
    doc.push_str("]`; `");
    doc.push_str(&strs.len().to_string());
    doc.push_str("` Entries");
    doc
}

struct DiffTag {
    values: Vec<String>,
    tag_type: Ident,
    name: Ident,
    key: String,
    old: TokenStream,
}

impl DiffTag {
    fn compute(&mut self, other: &[String], mapper: &IdMapper, module: TokenStream) -> TokenStream {
        // Note: I tried a smarter implementation before that could also just take a prefix from an existing tag,
        // basically emitting `pub const #name: Tag = #other_tag.first_chunk::<#len>().unwrap()`,
        // but that resulted in a pretty big compile time regression... So I suppose in LLVM we trust for binary size.

        if other == self.values {
            let old = &self.old;
            quote! { pub use #old; }
        } else {
            let name = &self.name;
            let tag_type = &self.tag_type;
            let doc = make_doc(other);
            let ids = other
                .iter()
                .filter_map(|v| mapper.resolve(self.key.as_ref(), v));

            self.values = other.to_vec();
            self.old = quote! { #module::#tag_type::#name };

            quote! {
                #[doc = #doc]
                pub const #name: Tag = &[#(#ids),*];
            }
        }
    }
}

struct IdMapper {
    blocks: BTreeMap<String, u16>,
    items: BTreeMap<String, u16>,
    biomes: BTreeMap<String, u16>,
    fluids: BTreeMap<String, u16>,
    enchantments: BTreeMap<String, u16>,
    entities: BTreeMap<String, u16>,
    dimensions: BTreeMap<String, u16>,
    timeline_id_map: BTreeMap<String, u16>,
}

impl IdMapper {
    pub fn new() -> Self {
        let block_assets: BlockAssets =
            serde_json::from_str(&fs::read_to_string("../assets/blocks.json").unwrap())
                .expect("Failed to parse blocks.json");
        let items: BTreeMap<String, Item> =
            serde_json::from_str(&fs::read_to_string("../assets/items.json").unwrap())
                .expect("Failed to parse items.json");
        let biomes: BTreeMap<String, Biome> =
            serde_json::from_str(&fs::read_to_string("../assets/biome.json").unwrap())
                .expect("Failed to parse biome.json");
        let fluids: Vec<Fluid> =
            serde_json::from_str(&fs::read_to_string("../assets/fluids.json").unwrap())
                .expect("Failed to parse fluids.json");
        let enchantments: BTreeMap<String, Enchantment> =
            serde_json::from_str(&fs::read_to_string("../assets/enchantments.json").unwrap())
                .expect("Failed to parse enchantments.json");
        let entities: BTreeMap<String, EntityType> =
            serde_json::from_str(&fs::read_to_string("../assets/entities.json").unwrap())
                .expect("Failed to parse entities.json");

        // build a map of dimension name -> numeric id
        let dimension_json: BTreeMap<String, serde_json::Value> =
            serde_json::from_str(&fs::read_to_string("../assets/dimension.json").unwrap())
                .expect("Failed to parse dimension.json");

        // also build timeline id map from registry file so timeline tags carry numbers
        let mut timeline_id_map: BTreeMap<String, u16> = BTreeMap::new();
        if let Ok(registries) = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string("../assets/registry/1_21_11_synced_registries.json").unwrap(),
        ) && let Some(timelines) = registries.get("timeline")
            && let Some(obj) = timelines.as_object()
        {
            for (i, name) in obj.keys().enumerate() {
                timeline_id_map.insert(name.clone(), i as u16);
            }
        }

        Self {
            blocks: block_assets
                .blocks
                .into_iter()
                .map(|b| (b.name, b.id.0))
                .collect(),
            items: items.into_iter().map(|(k, v)| (k, v.id)).collect(),
            biomes: biomes.into_iter().map(|(k, v)| (k, v.id as u16)).collect(),
            fluids: fluids.into_iter().map(|f| (f.name, f.id)).collect(),
            enchantments: enchantments
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.strip_prefix("minecraft:")
                            .map(|v| v.to_string())
                            .unwrap_or(k),
                        v.id as u16,
                    )
                })
                .collect(),
            entities: entities.into_iter().map(|(k, v)| (k, v.id)).collect(),
            dimensions: dimension_json
                .into_keys()
                .enumerate()
                .map(|(i, k)| (k, i as u16))
                .collect(),
            timeline_id_map,
        }
    }

    pub fn resolve(&self, r#type: &str, key: &str) -> Option<u16> {
        let map = match r#type {
            "worldgen/biome" => &self.biomes,
            "fluid" => &self.fluids,
            "item" => &self.items,
            "block" => &self.blocks,
            "enchantment" => &self.enchantments,
            "entity_type" => &self.entities,
            "dimension_type" => &self.dimensions,
            "timeline" => &self.timeline_id_map,
            _ => return None,
        };

        map.get(key).copied()
    }
}

/// The newest protocol version whose tag data is served as the latest-version fallback.
const LATEST_VERSION: JavaMinecraftVersion = JavaMinecraftVersion::V_26_2;

/// Generates the `TokenStream` for the `Tag` type, `RegistryKey` enum, all per-version tag
/// modules, and the `Taggable` trait with its lookup helpers.
pub(crate) fn build() -> TokenStream {
    // Watch specific tag versions
    let assets = [
        // latest: (JavaMinecraftVersion::V_26_2, "26_2_tags.json"),
        (JavaMinecraftVersion::V_26_1, "26_1_tags.json"),
        (JavaMinecraftVersion::V_1_21_11, "1_21_11_tags.json"),
        (JavaMinecraftVersion::V_1_21_9, "1_21_9_tags.json"),
        (JavaMinecraftVersion::V_1_21_7, "1_21_7_tags.json"),
        (JavaMinecraftVersion::V_1_21_6, "1_21_6_tags.json"),
        (JavaMinecraftVersion::V_1_21_5, "1_21_5_tags.json"),
        (JavaMinecraftVersion::V_1_21_4, "1_21_4_tags.json"),
        (JavaMinecraftVersion::V_1_21_2, "1_21_2_tags.json"),
        // TODO: upload 1_21_tags.json
        (JavaMinecraftVersion::V_1_21, "1_21_2_tags.json"),
        (JavaMinecraftVersion::V_1_20_5, "1_21_2_tags.json"),
    ];
    let latest_assets = (JavaMinecraftVersion::V_26_2, "26_2_tags.json");

    let id_mapper = IdMapper::new();

    let mut all_registry_keys = HashSet::new();
    all_registry_keys.insert("dimension_type".to_string());

    let mut latest_modules = Vec::new();
    let mut legacy_modules = Vec::new();

    let mut match_get_map = Vec::new();

    let mut diff_tags: BTreeMap<String, BTreeMap<String, DiffTag>> = BTreeMap::new();
    // latest tags
    let (_, file) = latest_assets;
    let file_path = format!("../assets/tags/{file}");

    let tags: BTreeMap<String, BTreeMap<String, Vec<String>>> =
        serde_json::from_str(&fs::read_to_string(&file_path).unwrap()).unwrap();

    let mut tag_dicts = Vec::new();
    let mut match_local_map = Vec::new();

    for (key, tag_map) in tags {
        all_registry_keys.insert(key.clone());
        let key_pascal = format_ident!("{}", key.to_pascal_case());
        let dict_name = format_ident!("{}_TAGS", key.to_shouty_snake_case());

        let mut tag_entries = Vec::new();
        let mut tag_map_entries = Vec::new();

        for (tag_name, values) in tag_map {
            if values.is_empty() {
                continue;
            }
            let tag_const_name =
                format_ident!("{}", tag_name.replace([':', '/'], "_").to_uppercase());
            let mut diff_tag = DiffTag {
                values: Vec::new(),
                tag_type: key_pascal.clone(),
                name: tag_const_name.clone(),
                key: key.clone(),
                old: quote! {},
            };
            let tag = diff_tag.compute(&values, &id_mapper, quote! { crate::tag });

            tag_entries.push(tag);
            tag_map_entries.push(quote! { #tag_name => #key_pascal::#tag_const_name });

            diff_tags
                .entry(key.clone())
                .or_default()
                .insert(tag_name, diff_tag);
        }

        tag_dicts.push(quote! {
            #[allow(non_snake_case)]
            pub mod #key_pascal {
                use crate::tag::Tag;
                #(#tag_entries)*
            }
            static #dict_name: phf::Map<&'static str, Tag> = phf::phf_map! {
                #(#tag_map_entries),* };
        });

        match_local_map.push(quote! { RegistryKey::#key_pascal => Some(&#dict_name) });
    }

    latest_modules.push(quote! {
        #(#tag_dicts)*
        #[allow(unreachable_patterns)]
        #[must_use]
        pub const fn get_latest_map(key: RegistryKey) -> Option<&'static phf::Map<&'static str, Tag>> {
            match key { #(#match_local_map,)* _ => None }
        }
    });
    match_get_map.push(quote! { #LATEST_VERSION => get_latest_map(tag_category) });

    // legacy tags
    for (ver, file) in assets {
        let file_path = format!("../assets/tags/{file}");

        let tags: BTreeMap<String, BTreeMap<String, Vec<String>>> =
            serde_json::from_str(&fs::read_to_string(&file_path).unwrap()).unwrap();

        let mut tag_dicts = Vec::new();
        let mut match_local_map = Vec::new();

        let mod_name = format_ident!("tags_{}", ver.to_field_ident());
        let module = quote! { crate::tag::#mod_name };

        for (key, tag_map) in tags {
            let diff = diff_tags.entry(key.clone()).or_default();
            all_registry_keys.insert(key.clone());
            let key_pascal = format_ident!("{}", key.to_pascal_case());
            let dict_name = format_ident!("{}_TAGS", key.to_pascal_case().to_uppercase());

            let mut tag_entries = Vec::new();
            let mut tag_map_entries = Vec::new();

            for (tag_name, values) in tag_map {
                if values.is_empty() {
                    continue;
                }
                let tag_const_name =
                    format_ident!("{}", tag_name.replace([':', '/'], "_").to_uppercase());

                let tag_entry = diff
                    .entry(tag_name.clone())
                    .or_insert_with(|| DiffTag {
                        values: Vec::new(),
                        tag_type: key_pascal.clone(),
                        name: tag_const_name.clone(),
                        key: key.clone(),
                        old: quote! {},
                    })
                    .compute(&values, &id_mapper, module.clone());

                tag_entries.push(tag_entry);

                tag_map_entries.push(quote! { #tag_name => #key_pascal::#tag_const_name });
            }

            tag_dicts.push(quote! {
                #[allow(non_snake_case)]
                pub mod #key_pascal {
                    use crate::tag::Tag;
                    #(#tag_entries)*
                }
                static #dict_name: phf::Map<&'static str, Tag> = phf::phf_map! {
                    #(#tag_map_entries),* };
            });

            match_local_map.push(quote! { RegistryKey::#key_pascal => Some(&#dict_name) });
        }

        let mod_name = format_ident!("tags_{}", ver.to_field_ident());
        legacy_modules.push(quote! {
                mod #mod_name {
                    use crate::tag::{Tag, RegistryKey};
                    #(#tag_dicts)*
                    #[must_use]
                    pub const fn get_map(key: RegistryKey) -> Option<&'static phf::Map<&'static str, Tag>> {
                        match key { #(#match_local_map,)* _ => None }
                    }
                }
            });
        match_get_map.push(quote! { #ver => #mod_name::get_map(tag_category) });
    }

    // --- Generate RegistryKey Enum ---
    let registry_key_enum = EnumCreator {
        name: "RegistryKey".to_string(),
        values: all_registry_keys.into_iter().collect(),
    }
    .to_token_stream();

    quote! {
        use pumpkin_util::version::JavaMinecraftVersion;

        pub type Tag = &'static [u16];

        #registry_key_enum

        // Latest tags are exported directly here
        #(#latest_modules)*

        // Legacy tags are hidden in their own module
        #(#legacy_modules)*

        #[must_use]
        pub fn get_tag_ids(tag_category: RegistryKey, tag: &str) -> Option<Tag> {
            get_latest_map(tag_category).and_then(|m| m.get(tag)).copied()
        }

        #[must_use]
        pub const fn get_registry_key_tags(version: JavaMinecraftVersion, tag_category: RegistryKey) -> Option<&'static phf::Map<&'static str, Tag>> {
            match version {
                #(#match_get_map),*,
                _ => get_latest_map(tag_category)
            }
        }

        pub trait Taggable {
            fn tag_key() -> RegistryKey;
            fn registry_key(&self) -> & /*'static*/ str;
            fn registry_id(&self) -> u16;

            // fn from_id(u16) -> Self;

            #[must_use]
            fn is_tagged_with(&self, tag: &str) -> Option<bool> {
                let tag = tag.strip_prefix("#").unwrap_or(tag);
                let items = get_tag_ids(Self::tag_key(), tag)?;
                Some(items.contains(&self.registry_id()))
            }

            #[must_use]
            fn has_tag(&self, tag: Tag) -> bool {
                tag.contains(&self.registry_id())
            }

            // #[must_use]
            // fn get_tag_values(tag: &str) -> Option<Vec<&'static str>> {
            //    let tag = tag.strip_prefix("#").unwrap_or(tag);
            //    Some(get_tag_ids(Self::tag_key(), tag)?.iter().copied().map(Self::from_id).collect())
            // }
        }
    }
}
