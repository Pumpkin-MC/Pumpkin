use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::fs;
use std::path::Path;

const fn default_spawn_range() -> i32 {
    4
}
const fn default_total_mobs() -> f32 {
    6.0
}
const fn default_simultaneous_mobs() -> f32 {
    2.0
}
const fn default_total_mobs_added_per_player() -> f32 {
    2.0
}
const fn default_simultaneous_mobs_added_per_player() -> f32 {
    1.0
}
const fn default_ticks_between_spawn() -> i32 {
    40
}
const fn default_weight() -> i32 {
    1
}

#[derive(Deserialize, Debug)]
struct TrialSpawnerJson {
    #[serde(default = "default_spawn_range")]
    spawn_range: i32,
    #[serde(default = "default_total_mobs")]
    total_mobs: f32,
    #[serde(default = "default_simultaneous_mobs")]
    simultaneous_mobs: f32,
    #[serde(default = "default_total_mobs_added_per_player")]
    total_mobs_added_per_player: f32,
    #[serde(default = "default_simultaneous_mobs_added_per_player")]
    simultaneous_mobs_added_per_player: f32,
    #[serde(default = "default_ticks_between_spawn")]
    ticks_between_spawn: i32,
    #[serde(default)]
    spawn_potentials: Vec<WeightedSpawnPotentialJson>,
    #[serde(default)]
    loot_tables_to_eject: Vec<WeightedLootTableJson>,
    #[serde(default)]
    items_to_drop_when_ominous: Option<String>,
}

#[derive(Deserialize, Debug)]
struct WeightedSpawnPotentialJson {
    data: SpawnDataJson,
    #[serde(default = "default_weight")]
    weight: i32,
}

#[derive(Deserialize, Debug)]
struct SpawnDataJson {
    entity: EntityRefJson,
    #[serde(default)]
    equipment: Option<EquipmentJson>,
}

#[derive(Deserialize, Debug)]
struct EntityRefJson {
    id: String,
}

#[derive(Deserialize, Debug)]
struct EquipmentJson {
    loot_table: String,
    #[serde(default)]
    slot_drop_chances: Option<f32>,
}

#[derive(Deserialize, Debug)]
struct WeightedLootTableJson {
    data: String,
    #[serde(default = "default_weight")]
    weight: i32,
}

fn collect_json_files(dir: &Path, base_dir: &Path, results: &mut Vec<(String, TrialSpawnerJson)>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_json_files(&path, base_dir, results);
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(relative) = path.strip_prefix(base_dir) {
                    let mut rel_str = relative.to_string_lossy().replace('\\', "/");
                    if let Some(stripped) = rel_str.strip_suffix(".json") {
                        rel_str = stripped.to_string();
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(parsed) = serde_json::from_str::<TrialSpawnerJson>(&content) {
                            results.push((rel_str, parsed));
                        }
                    }
                }
            }
        }
    }
}

pub fn build() -> TokenStream {
    let base_dir = Path::new("../../assets/datapacks/26_2/data/minecraft/trial_spawner");
    let mut files = Vec::new();
    collect_json_files(base_dir, base_dir, &mut files);
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut const_defs = TokenStream::new();
    let mut match_arms = TokenStream::new();

    let chamber_keys = [
        ("breeze", "Breeze", "trial_chamber/breeze"),
        ("melee_husk", "MeleeHusk", "trial_chamber/melee/husk"),
        ("melee_spider", "MeleeSpider", "trial_chamber/melee/spider"),
        ("melee_zombie", "MeleeZombie", "trial_chamber/melee/zombie"),
        (
            "ranged_poison_skeleton",
            "RangedPoisonSkeleton",
            "trial_chamber/ranged/poison_skeleton",
        ),
        (
            "ranged_skeleton",
            "RangedSkeleton",
            "trial_chamber/ranged/skeleton",
        ),
        ("ranged_stray", "RangedStray", "trial_chamber/ranged/stray"),
        (
            "slow_ranged_poison_skeleton",
            "SlowRangedPoisonSkeleton",
            "trial_chamber/slow_ranged/poison_skeleton",
        ),
        (
            "slow_ranged_skeleton",
            "SlowRangedSkeleton",
            "trial_chamber/slow_ranged/skeleton",
        ),
        (
            "slow_ranged_stray",
            "SlowRangedStray",
            "trial_chamber/slow_ranged/stray",
        ),
        (
            "small_melee_baby_zombie",
            "SmallMeleeBabyZombie",
            "trial_chamber/small_melee/baby_zombie",
        ),
        (
            "small_melee_cave_spider",
            "SmallMeleeCaveSpider",
            "trial_chamber/small_melee/cave_spider",
        ),
        (
            "small_melee_silverfish",
            "SmallMeleeSilverfish",
            "trial_chamber/small_melee/silverfish",
        ),
        (
            "small_melee_slime",
            "SmallMeleeSlime",
            "trial_chamber/small_melee/slime",
        ),
    ];

    for (rel_key, cfg) in &files {
        let const_name = rel_key.replace('/', "_").to_shouty_snake_case();
        let const_ident = format_ident!("{}", const_name);

        let spawn_range = cfg.spawn_range;
        let total_mobs = cfg.total_mobs;
        let simultaneous_mobs = cfg.simultaneous_mobs;
        let total_mobs_added_per_player = cfg.total_mobs_added_per_player;
        let simultaneous_mobs_added_per_player = cfg.simultaneous_mobs_added_per_player;
        let ticks_between_spawn = cfg.ticks_between_spawn;

        let spawn_potentials: Vec<_> = cfg
            .spawn_potentials
            .iter()
            .map(|sp| {
                let entity_id = &sp.data.entity.id;
                let weight = sp.weight;
                let (has_eq, eq_table, has_chance, eq_chance) = match &sp.data.equipment {
                    Some(eq) => (
                        true,
                        eq.loot_table.as_str(),
                        eq.slot_drop_chances.is_some(),
                        eq.slot_drop_chances.unwrap_or(0.0),
                    ),
                    None => (false, "", false, 0.0),
                };

                let eq_loot_stream = if has_eq {
                    quote! { Some(#eq_table) }
                } else {
                    quote! { None }
                };

                let eq_chance_stream = if has_chance {
                    quote! { Some(#eq_chance) }
                } else {
                    quote! { None }
                };

                quote! {
                    WeightedSpawnData {
                        entity_id: #entity_id,
                        equipment_loot_table: #eq_loot_stream,
                        equipment_slot_drop_chances: #eq_chance_stream,
                        weight: #weight,
                    }
                }
            })
            .collect();

        let loot_tables: Vec<_> = cfg
            .loot_tables_to_eject
            .iter()
            .map(|lt| {
                let table = &lt.data;
                let weight = lt.weight;
                quote! {
                    WeightedLootTable {
                        loot_table: #table,
                        weight: #weight,
                    }
                }
            })
            .collect();

        let ominous_items = match &cfg.items_to_drop_when_ominous {
            Some(s) => quote! { Some(#s) },
            None => quote! { None },
        };

        const_defs.extend(quote! {
            pub const #const_ident: TrialSpawnerConfigData = TrialSpawnerConfigData {
                spawn_range: #spawn_range,
                total_mobs: #total_mobs,
                simultaneous_mobs: #simultaneous_mobs,
                total_mobs_added_per_player: #total_mobs_added_per_player,
                simultaneous_mobs_added_per_player: #simultaneous_mobs_added_per_player,
                ticks_between_spawn: #ticks_between_spawn,
                spawn_potentials: &[
                    #(#spawn_potentials),*
                ],
                loot_tables_to_eject: &[
                    #(#loot_tables),*
                ],
                items_to_drop_when_ominous: #ominous_items,
            };
        });

        let mc_key = format!("minecraft:{rel_key}");
        match_arms.extend(quote! {
            #rel_key | #mc_key => Some(&#const_ident),
        });
    }

    let mut key_variants = TokenStream::new();
    let mut key_name_match = TokenStream::new();
    let mut key_normal_match = TokenStream::new();
    let mut key_ominous_match = TokenStream::new();
    let mut key_from_name_match = TokenStream::new();

    for (_, variant_name, path_str) in chamber_keys {
        let variant_ident = format_ident!("{}", variant_name);
        let normal_ident = format_ident!(
            "{}_NORMAL",
            path_str.replace('/', "_").to_shouty_snake_case()
        );
        let ominous_ident = format_ident!(
            "{}_OMINOUS",
            path_str.replace('/', "_").to_shouty_snake_case()
        );
        let mc_path = format!("minecraft:{path_str}");

        key_variants.extend(quote! {
            #variant_ident,
        });

        key_name_match.extend(quote! {
            Self::#variant_ident => #path_str,
        });

        key_normal_match.extend(quote! {
            Self::#variant_ident => &#normal_ident,
        });

        key_ominous_match.extend(quote! {
            Self::#variant_ident => &#ominous_ident,
        });

        key_from_name_match.extend(quote! {
            #path_str | #mc_path => Some(Self::#variant_ident),
        });
    }

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct TrialSpawnerConfigData {
            pub spawn_range: i32,
            pub total_mobs: f32,
            pub simultaneous_mobs: f32,
            pub total_mobs_added_per_player: f32,
            pub simultaneous_mobs_added_per_player: f32,
            pub ticks_between_spawn: i32,
            pub spawn_potentials: &'static [WeightedSpawnData],
            pub loot_tables_to_eject: &'static [WeightedLootTable],
            pub items_to_drop_when_ominous: Option<&'static str>,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct WeightedSpawnData {
            pub entity_id: &'static str,
            pub equipment_loot_table: Option<&'static str>,
            pub equipment_slot_drop_chances: Option<f32>,
            pub weight: i32,
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct WeightedLootTable {
            pub loot_table: &'static str,
            pub weight: i32,
        }

        #const_defs

        #[must_use]
        pub fn get_trial_spawner_config(key: &str) -> Option<&'static TrialSpawnerConfigData> {
            match key {
                #match_arms
                _ => None,
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum TrialSpawnerKey {
            #key_variants
        }

        impl TrialSpawnerKey {
            #[must_use]
            pub const fn name(&self) -> &'static str {
                match self {
                    #key_name_match
                }
            }

            #[must_use]
            pub const fn normal_config(&self) -> &'static TrialSpawnerConfigData {
                match self {
                    #key_normal_match
                }
            }

            #[must_use]
            pub const fn ominous_config(&self) -> &'static TrialSpawnerConfigData {
                match self {
                    #key_ominous_match
                }
            }

            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #key_from_name_match
                    _ => None,
                }
            }
        }
    }
}
