use std::{collections::BTreeMap, fs};

use proc_macro2::TokenStream;
use pumpkin_util::HeightMap;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use syn::LitInt;

use crate::loot::LootTableStruct;

#[derive(Deserialize)]
pub struct EntityType {
    pub id: u16,
    pub max_health: Option<f32>,
    pub attackable: Option<bool>,
    pub mob: Option<bool>,
    pub limit_per_chunk: Option<i32>,
    pub loot_table: Option<LootTableStruct>,
    pub summonable: bool,
    pub fire_immune: bool,
    pub saveable: bool,
    pub category: MobCategory,
    pub can_spawn_far_from_player: bool,
    pub dimension: [f32; 2],
    pub eye_height: f32,
    pub spawn_restriction: SpawnRestriction,
}

#[derive(Deserialize)]
pub struct SpawnRestriction {
    location: SpawnLocation,
    heightmap: HeightMap,
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpawnLocation {
    InLava,
    InWater,
    OnGround,
    Unrestricted,
}

#[derive(Deserialize)]
#[expect(non_camel_case_types)]
#[expect(clippy::upper_case_acronyms)]
pub enum MobCategory {
    MONSTER,
    CREATURE,
    AMBIENT,
    AXOLOTLS,
    UNDERGROUND_WATER_CREATURE,
    WATER_CREATURE,
    WATER_AMBIENT,
    MISC,
}

pub struct NamedEntityType<'a>(&'a str, &'a EntityType);

impl ToTokens for NamedEntityType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.0;
        let entity = self.1;
        let id = LitInt::new(&entity.id.to_string(), proc_macro2::Span::call_site());

        let max_health = match entity.max_health {
            Some(mh) => quote! { Some(#mh) },
            None => quote! { None },
        };

        let attackable = match entity.attackable {
            Some(a) => quote! { Some(#a) },
            None => quote! { None },
        };

        let spawn_restriction_location = match entity.spawn_restriction.location {
            SpawnLocation::InLava => quote! {SpawnLocation::InLava},
            SpawnLocation::InWater => quote! {SpawnLocation::InWater},
            SpawnLocation::OnGround => quote! {SpawnLocation::OnGround},
            SpawnLocation::Unrestricted => quote! {SpawnLocation::Unrestricted},
        };

        let spawn_restriction_heightmap = match entity.spawn_restriction.heightmap {
            HeightMap::WorldSurfaceWg => quote! { HeightMap::WorldSurfaceWg },
            HeightMap::WorldSurface => quote! { HeightMap::WorldSurface },
            HeightMap::OceanFloorWg => quote! { HeightMap::OceanFloorWg },
            HeightMap::OceanFloor => quote! { HeightMap::OceanFloor },
            HeightMap::MotionBlocking => quote! { HeightMap::MotionBlocking },
            HeightMap::MotionBlockingNoLeaves => quote! { HeightMap::MotionBlockingNoLeaves },
        };

        let spawn_restriction = quote! { SpawnRestriction {
            location: #spawn_restriction_location,
            heightmap: #spawn_restriction_heightmap,
        }};

        let spawn_category = match entity.category {
            MobCategory::MONSTER => quote! { MobCategory::MONSTER },
            MobCategory::CREATURE => quote! { MobCategory::CREATURE },
            MobCategory::AMBIENT => quote! { MobCategory::AMBIENT },
            MobCategory::AXOLOTLS => quote! { MobCategory::AXOLOTLS },
            MobCategory::UNDERGROUND_WATER_CREATURE => {
                quote! { MobCategory::UNDERGROUND_WATER_CREATURE }
            }
            MobCategory::WATER_CREATURE => quote! { MobCategory::WATER_CREATURE },
            MobCategory::WATER_AMBIENT => quote! { MobCategory::WATER_AMBIENT },
            MobCategory::MISC => quote! { MobCategory::MISC },
        };

        let saveable = entity.saveable;
        let summonable = entity.summonable;
        let fire_immune = entity.fire_immune;
        let eye_height = entity.eye_height;
        if entity.mob.is_none() && name != "player" {
            panic!("missing field 'mob', entity name {name}");
        }
        if entity.limit_per_chunk.is_none() && name != "player" {
            panic!("missing field 'mob', entity name {name}");
        }
        let mob = entity.mob.unwrap_or(false);
        let limit_per_chunk = entity.limit_per_chunk.unwrap_or(0);
        let can_spawn_far_from_player = entity.can_spawn_far_from_player;

        let dimension0 = entity.dimension[0];
        let dimension1 = entity.dimension[1];

        let loot_table = match &entity.loot_table {
            Some(table) => {
                let table_tokens = table.to_token_stream();
                quote! { Some(#table_tokens) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            EntityType {
                id: #id,
                max_health: #max_health,
                attackable: #attackable,
                mob: #mob,
                saveable: #saveable,
                limit_per_chunk: #limit_per_chunk,
                summonable: #summonable,
                fire_immune: #fire_immune,
                category: &#spawn_category,
                can_spawn_far_from_player: #can_spawn_far_from_player,
                loot_table: #loot_table,
                dimension: [#dimension0, #dimension1], // Correctly construct the array
                eye_height: #eye_height,
                spawn_restriction: #spawn_restriction,
                resource_name: #name,
            }
        });
    }
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/entities.json");

    let json: BTreeMap<String, EntityType> =
        serde_json::from_str(&fs::read_to_string("../assets/entities.json").unwrap())
            .expect("Failed to parse entities.json");

    let mut consts = TokenStream::new();
    let mut type_from_raw_id_arms = TokenStream::new();
    let mut type_from_name = TokenStream::new();

    for (name, entity) in json.iter() {
        let id = entity.id as u8;
        let id_lit = LitInt::new(&id.to_string(), proc_macro2::Span::call_site());
        let upper_name = format_ident!("{}", name.to_uppercase());

        let entity_tokens = NamedEntityType(name, entity).to_token_stream();

        consts.extend(quote! {
            pub const #upper_name: EntityType = #entity_tokens;
        });

        type_from_raw_id_arms.extend(quote! {
            #id_lit => Some(&Self::#upper_name),
        });

        type_from_name.extend(quote! {
            #name => Some(&Self::#upper_name),
        });
    }
    quote! {
        use pumpkin_util::loot_table::*;
        use pumpkin_util::HeightMap;
        use std::hash::Hash;

        #[derive(Debug)]
        pub struct EntityType {
            pub id: u16,
            pub max_health: Option<f32>,
            pub attackable: Option<bool>,
            pub mob: bool,
            pub saveable: bool,
            pub limit_per_chunk: i32,
            pub summonable: bool,
            pub fire_immune: bool,
            pub category: &'static MobCategory,
            pub can_spawn_far_from_player: bool,
            pub loot_table: Option<LootTable>,
            pub dimension: [f32; 2],
            pub eye_height: f32,
            pub spawn_restriction: SpawnRestriction,
            pub resource_name: &'static str,
        }

        impl Hash for EntityType {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        impl PartialEq for EntityType {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        impl Eq for EntityType {}

        #[derive(Debug)]
        pub struct SpawnRestriction {
            pub location: SpawnLocation,
            pub heightmap: HeightMap,
        }

        #[derive(Debug)]
        pub enum SpawnLocation {
            InLava,
            InWater,
            OnGround,
            Unrestricted,
        }

        #[derive(Debug)]
        pub struct MobCategory {
            pub id: usize, // mojang don't have this field
            pub max: i32,
            pub is_friendly: bool,
            pub is_persistent: bool,
            pub despawn_distance: i32,
        }

        impl PartialEq for MobCategory {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        impl MobCategory {
            pub const NO_DESPAWN_DISTANCE: i32 = 32;
            pub const MONSTER: MobCategory = MobCategory {
                id: 0,
                max: 70,
                is_friendly: false,
                is_persistent: false,
                despawn_distance: 128,
            };
            pub const CREATURE: MobCategory = MobCategory {
                id: 1,
                max: 10,
                is_friendly: true,
                is_persistent: true,
                despawn_distance: 128,
            };
            pub const AMBIENT: MobCategory = MobCategory {
                id: 2,
                max: 15,
                is_friendly: true,
                is_persistent: false,
                despawn_distance: 128,
            };
            pub const AXOLOTLS: MobCategory = MobCategory {
                id: 3,
                max: 5,
                is_friendly: true,
                is_persistent: false,
                despawn_distance: 128,
            };
            pub const UNDERGROUND_WATER_CREATURE: MobCategory = MobCategory {
                id: 4,
                max: 5,
                is_friendly: true,
                is_persistent: false,
                despawn_distance: 128,
            };
            pub const WATER_CREATURE: MobCategory = MobCategory {
                id: 5,
                max: 5,
                is_friendly: true,
                is_persistent: true,
                despawn_distance: 128,
            };
            pub const WATER_AMBIENT: MobCategory = MobCategory {
                id: 6,
                max: 20,
                is_friendly: true,
                is_persistent: false,
                despawn_distance: 64,
            };
            pub const MISC: MobCategory = MobCategory {
                id: 7,
                max: -1,
                is_friendly: true,
                is_persistent: true,
                despawn_distance: 128,
            };
            pub const SPAWNING_CATEGORIES: [&'static Self; 8] = [
                &Self::MONSTER,
                &Self::CREATURE,
                &Self::AMBIENT,
                &Self::AXOLOTLS,
                &Self::UNDERGROUND_WATER_CREATURE,
                &Self::WATER_CREATURE,
                &Self::WATER_AMBIENT,
                &Self::MISC,
            ];
        }

        impl EntityType {
            #consts

            pub const fn from_raw(id: u16) -> Option<&'static Self> {
                match id {
                    #type_from_raw_id_arms
                    _ => None
                }
            }

            pub fn from_name(name: &str) -> Option<&'static Self> {
                let name = name.strip_prefix("minecraft:").unwrap_or(name);
                match name {
                    #type_from_name
                    _ => None
                }
            }
        }
    }
}
