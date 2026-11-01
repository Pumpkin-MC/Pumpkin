use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

const DEFAULT_NAMESPACE: &str = "minecraft";

#[derive(Deserialize, Debug)]
struct RawProvider {
    #[serde(rename = "type")]
    provider_type: String,
    #[serde(default)]
    enchantment: Option<String>,
    #[serde(default)]
    level: Option<serde_json::Value>,
    #[serde(default)]
    enchantments: Option<serde_json::Value>,
    #[serde(default)]
    min_cost: Option<i32>,
    #[serde(default)]
    max_cost_span: Option<i32>,
    #[serde(default)]
    cost: Option<serde_json::Value>,
}

struct ProviderEntry {
    qualified: String,
    bare: String,
    const_name: String,
    rel_path: String,
    kind_tokens: TokenStream,
}

pub fn build() -> TokenStream {
    let provider_dir = Path::new("../../assets/datapack/data/minecraft/enchantment_provider");
    let tags_dir = Path::new("../../assets/datapack/data/minecraft/tags/enchantment");

    let mut raw_files: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    if provider_dir.is_dir() {
        scan_dir(provider_dir, "", &mut raw_files);
    }

    let mut entries: Vec<ProviderEntry> = Vec::new();

    for (qualified, (bare, const_name, rel_path)) in raw_files {
        // Read and parse JSON
        let full_path = Path::new("../../assets/datapack/data/minecraft/enchantment_provider")
            .join(format!("{bare}.json"));
        let content = fs::read_to_string(&full_path).unwrap_or_else(|e| {
            panic!(
                "failed to read enchantment provider file {}: {}",
                full_path.display(),
                e
            )
        });
        let raw: RawProvider = serde_json::from_str(&content).unwrap_or_else(|e| {
            panic!(
                "failed to parse enchantment provider JSON {}: {}",
                full_path.display(),
                e
            )
        });

        let kind_tokens = match raw.provider_type.as_str() {
            "minecraft:single" | "single" => {
                let enc_str = raw
                    .enchantment
                    .expect("single provider requires enchantment");
                let enc_bare = enc_str.strip_prefix("minecraft:").unwrap_or(&enc_str);
                let enc_ident = format_ident!("{}", enc_bare.to_shouty_snake_case());
                let level = match raw.level {
                    Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(1) as i32,
                    _ => 1,
                };
                quote! {
                    EnchantmentProviderKind::Single {
                        enchantment: &Enchantment::#enc_ident,
                        level: #level,
                    }
                }
            }
            "minecraft:by_cost_with_difficulty" | "by_cost_with_difficulty" => {
                let enc_names = extract_enchantments(raw.enchantments.as_ref(), tags_dir);
                let enc_idents: Vec<_> = enc_names
                    .iter()
                    .map(|name| {
                        let bare = name.strip_prefix("minecraft:").unwrap_or(name);
                        let ident = format_ident!("{}", bare.to_shouty_snake_case());
                        quote! { &Enchantment::#ident }
                    })
                    .collect();
                let min_cost = raw.min_cost.unwrap_or(5);
                let max_cost_span = raw.max_cost_span.unwrap_or(17);
                quote! {
                    EnchantmentProviderKind::ByCostWithDifficulty {
                        enchantments: &[ #( #enc_idents ),* ],
                        min_cost: #min_cost,
                        max_cost_span: #max_cost_span,
                    }
                }
            }
            "minecraft:by_cost" | "by_cost" => {
                let enc_names = extract_enchantments(raw.enchantments.as_ref(), tags_dir);
                let enc_idents: Vec<_> = enc_names
                    .iter()
                    .map(|name| {
                        let bare = name.strip_prefix("minecraft:").unwrap_or(name);
                        let ident = format_ident!("{}", bare.to_shouty_snake_case());
                        quote! { &Enchantment::#ident }
                    })
                    .collect();
                let cost = match raw.cost {
                    Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(1) as i32,
                    _ => 1,
                };
                quote! {
                    EnchantmentProviderKind::ByCost {
                        enchantments: &[ #( #enc_idents ),* ],
                        cost: #cost,
                    }
                }
            }
            other => panic!("unknown enchantment provider type: {other}"),
        };

        entries.push(ProviderEntry {
            qualified,
            bare,
            const_name,
            rel_path,
            kind_tokens,
        });
    }

    let mut const_decls = Vec::new();
    let mut from_name_arms = Vec::new();
    let mut raw_json_arms = Vec::new();
    let mut all_const_idents = Vec::new();
    let mut all_qualified_names = Vec::new();

    for entry in &entries {
        let const_ident = format_ident!("{}", entry.const_name);
        let qualified = &entry.qualified;
        let bare = &entry.bare;
        let rel_path = &entry.rel_path;
        let kind_tokens = &entry.kind_tokens;

        const_decls.push(quote! {
            pub const #const_ident: Self = Self {
                name: #qualified,
                kind: #kind_tokens,
            };
        });

        from_name_arms.push(quote! {
            #qualified | #bare => Some(&Self::#const_ident),
        });

        raw_json_arms.push(quote! {
            #qualified | #bare => Some(include_str!(#rel_path)),
        });

        all_const_idents.push(quote! { Self::#const_ident });
        all_qualified_names.push(quote! { #qualified });
    }

    quote! {
        use rand::RngExt;
        use crate::enchantment::Enchantment;
        use crate::item::Item;
        use crate::item_stack::ItemStack;

        #[derive(Clone, Copy, PartialEq)]
        pub enum EnchantmentProviderKind {
            Single {
                enchantment: &'static Enchantment,
                level: i32,
            },
            ByCostWithDifficulty {
                enchantments: &'static [&'static Enchantment],
                min_cost: i32,
                max_cost_span: i32,
            },
            ByCost {
                enchantments: &'static [&'static Enchantment],
                cost: i32,
            },
        }

        impl core::fmt::Debug for EnchantmentProviderKind {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    Self::Single { enchantment, level } => f
                        .debug_struct("Single")
                        .field("enchantment", &enchantment.registry_key)
                        .field("level", level)
                        .finish(),
                    Self::ByCostWithDifficulty {
                        enchantments,
                        min_cost,
                        max_cost_span,
                    } => {
                        let keys: Vec<&str> = enchantments.iter().map(|e| e.registry_key).collect();
                        f.debug_struct("ByCostWithDifficulty")
                            .field("enchantments", &keys)
                            .field("min_cost", min_cost)
                            .field("max_cost_span", max_cost_span)
                            .finish()
                    }
                    Self::ByCost {
                        enchantments,
                        cost,
                    } => {
                        let keys: Vec<&str> = enchantments.iter().map(|e| e.registry_key).collect();
                        f.debug_struct("ByCost")
                            .field("enchantments", &keys)
                            .field("cost", cost)
                            .finish()
                    }
                }
            }
        }

        #[derive(Clone, Copy, PartialEq)]
        pub struct EnchantmentProvider {
            pub name: &'static str,
            pub kind: EnchantmentProviderKind,
        }

        impl core::fmt::Debug for EnchantmentProvider {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("EnchantmentProvider")
                    .field("name", &self.name)
                    .field("kind", &self.kind)
                    .finish()
            }
        }

        impl EnchantmentProvider {
            #( #const_decls )*

            pub const ALL: &'static [Self] = &[
                #( #all_const_idents ),*
            ];

            #[must_use]
            pub const fn name(&self) -> &'static str {
                self.name
            }

            #[must_use]
            pub const fn kind(&self) -> &EnchantmentProviderKind {
                &self.kind
            }

            #[allow(clippy::too_many_lines)]
            #[allow(clippy::match_same_arms)]
            #[must_use]
            pub fn from_name(name: &str) -> Option<&'static Self> {
                match name {
                    #( #from_name_arms )*
                    _ => None,
                }
            }

            #[allow(clippy::too_many_lines)]
            #[allow(clippy::match_same_arms)]
            #[must_use]
            pub fn get_raw_json(name: &str) -> Option<&'static str> {
                match name {
                    #( #raw_json_arms )*
                    _ => None,
                }
            }

            #[must_use]
            pub const fn all() -> &'static [Self] {
                Self::ALL
            }

            #[must_use]
            pub const fn all_provider_names() -> &'static [&'static str] {
                &[
                    #( #all_qualified_names ),*
                ]
            }

            /// Evaluates the provider for the given item and difficulty multiplier, returning
            /// the selected enchantments and their levels.
            #[must_use]
            pub fn select_enchantments(
                &self,
                item: &'static Item,
                special_multiplier: f32,
            ) -> Vec<(&'static Enchantment, i32)> {
                match &self.kind {
                    EnchantmentProviderKind::Single { enchantment, level } => {
                        vec![(*enchantment, *level)]
                    }
                    EnchantmentProviderKind::ByCostWithDifficulty {
                        enchantments,
                        min_cost,
                        max_cost_span,
                    } => {
                        let cost = *min_cost + (special_multiplier * *max_cost_span as f32).round() as i32;
                        Self::select_by_cost(item, enchantments, cost)
                    }
                    EnchantmentProviderKind::ByCost {
                        enchantments,
                        cost,
                    } => Self::select_by_cost(item, enchantments, *cost),
                }
            }

            /// Evaluates and applies the enchantments directly to an `ItemStack`.
            pub fn apply_to_stack(&self, stack: &mut ItemStack, special_multiplier: f32) {
                let enchantments = self.select_enchantments(stack.item, special_multiplier);
                for (enchantment, level) in enchantments {
                    stack.enchant(enchantment, level);
                }
            }

            /// Selects enchantments by cost with conflict resolution, matching vanilla
            /// `EnchantmentHelper.enchantItem`.
            #[must_use]
            pub fn select_by_cost(
                item: &'static Item,
                possible_enchantments: &[&'static Enchantment],
                initial_cost: i32,
            ) -> Vec<(&'static Enchantment, i32)> {
                let candidates: Vec<&'static Enchantment> = possible_enchantments
                    .iter()
                    .copied()
                    .filter(|e| e.can_enchant(item))
                    .collect();

                if candidates.is_empty() || initial_cost <= 0 {
                    return Vec::new();
                }

                let mut cost = initial_cost;
                let mut applied: Vec<(&'static Enchantment, i32)> = Vec::new();
                let mut rng = rand::rng();

                loop {
                    let available: Vec<&'static Enchantment> = candidates
                        .iter()
                        .copied()
                        .filter(|cand| {
                            !applied.iter().any(|(app, _)| app.id == cand.id)
                                && applied.iter().all(|(app, _)| cand.are_compatible(app))
                        })
                        .collect();

                    if available.is_empty() {
                        break;
                    }

                    let total_weight: f32 = available.iter().map(|e| e.weight as f32).sum();
                    if total_weight <= 0.0 {
                        break;
                    }

                    let mut roll = rng.random_range(0.0..total_weight);
                    let mut selected: Option<&'static Enchantment> = None;
                    for e in &available {
                        roll -= e.weight as f32;
                        if roll <= 0.0 {
                            selected = Some(e);
                            break;
                        }
                    }
                    let Some(&fallback) = available.last() else {
                        break;
                    };
                    let selected = selected.unwrap_or(fallback);

                    let mut level = 1;
                    for lvl in (1..=selected.max_level).rev() {
                        if cost >= selected.min_cost.calculate(lvl) {
                            level = lvl;
                            break;
                        }
                    }

                    applied.push((selected, level.clamp(1, selected.max_level)));

                    cost /= 2;
                    if cost < 1 {
                        break;
                    }
                }

                applied
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn all_providers_count() {
                assert!(EnchantmentProvider::ALL.len() >= 7);
            }

            #[test]
            fn lookup_by_name() {
                assert_eq!(
                    EnchantmentProvider::from_name("minecraft:mob_spawn_equipment"),
                    Some(&EnchantmentProvider::MOB_SPAWN_EQUIPMENT)
                );
                assert_eq!(
                    EnchantmentProvider::from_name("mob_spawn_equipment"),
                    Some(&EnchantmentProvider::MOB_SPAWN_EQUIPMENT)
                );
                assert_eq!(
                    EnchantmentProvider::from_name("enderman_loot_drop"),
                    Some(&EnchantmentProvider::ENDERMAN_LOOT_DROP)
                );
                assert_eq!(
                    EnchantmentProvider::from_name("raid/vindicator"),
                    Some(&EnchantmentProvider::RAID_VINDICATOR)
                );
                assert_eq!(
                    EnchantmentProvider::from_name("minecraft:raid/pillager_post_wave_5"),
                    Some(&EnchantmentProvider::RAID_PILLAGER_POST_WAVE_5)
                );
            }

            #[test]
            fn raw_json_retrieval() {
                assert!(EnchantmentProvider::get_raw_json("mob_spawn_equipment").is_some());
                assert!(EnchantmentProvider::get_raw_json("minecraft:enderman_loot_drop").is_some());
                assert!(EnchantmentProvider::get_raw_json("nonexistent").is_none());
            }

            #[test]
            fn single_enchantment_provider() {
                let encs = EnchantmentProvider::ENDERMAN_LOOT_DROP.select_enchantments(&Item::DIAMOND_SWORD, 0.0);
                assert_eq!(encs.len(), 1);
                assert_eq!(encs[0].0.registry_key, "silk_touch");
                assert_eq!(encs[0].1, 1);
            }

            #[test]
            fn mob_spawn_equipment_application() {
                let mut stack = ItemStack::new(1, &Item::DIAMOND_SWORD);
                EnchantmentProvider::MOB_SPAWN_EQUIPMENT.apply_to_stack(&mut stack, 1.0);
                assert!(stack.has_enchantments());
            }
        }
    }
}

fn scan_dir(dir: &Path, prefix: &str, registry: &mut BTreeMap<String, (String, String, String)>) {
    let mut entries = fs::read_dir(dir)
        .expect("read dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            let new_prefix = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            scan_dir(&path, &new_prefix, registry);
            continue;
        }

        let Some(stem) = name.strip_suffix(".json") else {
            continue;
        };

        let id = if prefix.is_empty() {
            stem.to_string()
        } else {
            format!("{prefix}/{stem}")
        };

        let qualified = format!("{DEFAULT_NAMESPACE}:{id}");
        let bare = id.clone();
        let const_name = id.replace(['/', '-', '.'], "_").to_uppercase();

        let rel_to_repo = path.strip_prefix("../../").expect("strip ../../");
        let rel_path = format!(
            "../../../../{}",
            rel_to_repo.to_string_lossy().replace('\\', "/")
        );

        registry.insert(qualified, (bare, const_name, rel_path));
    }
}

fn extract_enchantments(val: Option<&serde_json::Value>, tags_dir: &Path) -> Vec<String> {
    let Some(val) = val else {
        return Vec::new();
    };

    let mut result = Vec::new();
    match val {
        serde_json::Value::String(s) => {
            if s.starts_with('#') {
                let mut visited = HashSet::new();
                resolve_tag_recursive(s, tags_dir, &mut visited, &mut result);
            } else {
                let bare = s.strip_prefix("minecraft:").unwrap_or(s);
                result.push(bare.to_string());
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                if let Some(s) = item.as_str() {
                    if s.starts_with('#') {
                        let mut visited = HashSet::new();
                        resolve_tag_recursive(s, tags_dir, &mut visited, &mut result);
                    } else {
                        let bare = s.strip_prefix("minecraft:").unwrap_or(s);
                        result.push(bare.to_string());
                    }
                }
            }
        }
        _ => {}
    }

    result.sort();
    result.dedup();
    result
}

fn resolve_tag_recursive(
    tag: &str,
    tags_dir: &Path,
    visited: &mut HashSet<String>,
    output: &mut Vec<String>,
) {
    let tag_name = tag.strip_prefix('#').unwrap_or(tag);
    if !visited.insert(tag_name.to_string()) {
        return;
    }

    let bare = tag_name.strip_prefix("minecraft:").unwrap_or(tag_name);
    let tag_file = tags_dir.join(format!("{bare}.json"));
    if !tag_file.exists() {
        return;
    }

    let content = fs::read_to_string(&tag_file).unwrap_or_default();
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return;
    };

    if let Some(values) = json.get("values").and_then(|v| v.as_array()) {
        for v in values {
            if let Some(s) = v.as_str() {
                if s.starts_with('#') {
                    resolve_tag_recursive(s, tags_dir, visited, output);
                } else {
                    let enc = s.strip_prefix("minecraft:").unwrap_or(s);
                    output.push(enc.to_string());
                }
            }
        }
    }
}
