use std::{collections::HashMap, fs};

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
    pub loot_table: Option<LootTableStruct>,
    pub summonable: bool,
    pub fire_immune: bool,
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

        let summonable = entity.summonable;
        let fire_immune = entity.fire_immune;
        let eye_height = entity.eye_height;

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
                summonable: #summonable,
                fire_immune: #fire_immune,
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

    let json: HashMap<String, EntityType> =
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
            #id_lit => Some(Self::#upper_name),
        });

        type_from_name.extend(quote! {
            #name => Some(Self::#upper_name),
        });
    }
    quote! {
        use pumpkin_util::loot_table::*;
        use pumpkin_util::HeightMap;

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct EntityType {
            pub id: u16,
            pub max_health: Option<f32>,
            pub attackable: Option<bool>,
            pub summonable: bool,
            pub fire_immune: bool,
            pub loot_table: Option<LootTable>,
            pub dimension: [f32; 2],
            pub eye_height: f32,
            pub spawn_restriction: SpawnRestriction,
            pub resource_name: &'static str,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct SpawnRestriction {
            location: SpawnLocation,
            heightmap: HeightMap,
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
         pub enum SpawnLocation {
              InLava,
             InWater,
             OnGround,
             Unrestricted
         }

        impl EntityType {
            #consts

            pub const fn from_raw(id: u16) -> Option<Self> {
                match id {
                    #type_from_raw_id_arms
                    _ => None
                }
            }

            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #type_from_name
                    _ => None
                }
            }
        }
    }
}
