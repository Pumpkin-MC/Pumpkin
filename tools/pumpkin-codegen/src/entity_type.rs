use std::{collections::BTreeMap, fs};

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use pumpkin_util::HeightMap;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use syn::LitInt;

/// Raw deserialization shape for a single entity type entry from `entities.json`.
#[derive(Deserialize)]
pub struct EntityType {
    /// Numeric registry ID for this entity type.
    pub id: u16,
    pub attributes: Option<Vec<BTreeMap<String, f64>>>,
    pub experience_reward: Option<u32>,
    /// Static hurt sound event name when it is safely derivable from extracted entity data.
    pub hurt_sound: Option<String>,
    /// Static death sound event name when it is safely derivable from extracted entity data.
    pub death_sound: Option<String>,
    /// Vanilla `getSoundSource`; missing only for the player.
    pub sound_source: Option<String>,
    /// Vanilla `getMovementEmission`; missing only for the player, whose emission depends on its state.
    pub movement_emission: Option<MovementEmission>,
    /// Vanilla `getSwimSound`.
    pub swim_sound: Option<String>,
    /// Vanilla `getSwimSplashSound`.
    pub swim_splash_sound: Option<String>,
    /// Vanilla `getSwimHighSpeedSplashSound`.
    pub swim_high_speed_splash_sound: Option<String>,
    /// Vanilla `getFallSounds().small()`, living entities only.
    pub small_fall_sound: Option<String>,
    /// Vanilla `getFallSounds().big()`, living entities only.
    pub big_fall_sound: Option<String>,
    /// Whether this entity can be attacked by players or other entities.
    pub attackable: Option<bool>,
    /// Whether this entity is classified as a mob (affects spawning mechanics).
    pub mob: Option<bool>,
    /// Whether the vanilla entity implements the Enemy marker interface.
    pub is_enemy: bool,
    /// Maximum number of this entity type allowed per chunk, if capped.
    pub limit_per_chunk: Option<i32>,
    /// Whether this entity can be summoned by the `/summon` command.
    pub summonable: bool,
    /// Whether this entity is immune to fire damage.
    pub fire_immune: bool,
    /// Whether this entity is saved to the world file.
    pub saveable: bool,
    /// Mob category controlling this entity's spawn cap and persistence.
    pub category: MobCategory,
    /// Whether this entity can spawn far from the player (beyond normal spawn range).
    pub can_spawn_far_from_player: bool,
    /// Client tracking range in chunks.
    pub client_tracking_range: u32,
    /// Update interval in ticks.
    pub update_interval: u32,
    /// Whether movement deltas should be tracked.
    pub track_deltas: bool,
    /// Bounding box dimensions as `[width, height]` in blocks.
    pub dimension: [f32; 2],
    /// Eye height in blocks, used for line-of-sight calculations.
    pub eye_height: f32,
    /// Scale factor for spawn dimensions (e.g. slimes/magma cubes).
    pub spawn_dimensions_scale: f32,
    /// Spawn location restrictions for natural spawning.
    pub spawn_restriction: SpawnRestriction,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum MovementEmission {
    None,
    Sounds,
    Events,
    All,
}

impl ToTokens for MovementEmission {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::None => quote! { MovementEmission::None },
            Self::Sounds => quote! { MovementEmission::Sounds },
            Self::Events => quote! { MovementEmission::Events },
            Self::All => quote! { MovementEmission::All },
        });
    }
}

/// Spawn restrictions controlling where an entity is allowed to naturally spawn.
#[derive(Deserialize)]
pub struct SpawnRestriction {
    /// Surface or fluid condition that must be satisfied at the spawn location.
    location: SpawnLocation,
    /// Height map used to determine the valid Y-range for spawning.
    heightmap: HeightMap,
}

/// Surface or fluid condition that must be present at an entity's spawn location.
#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpawnLocation {
    InLava,
    InWater,
    OnGround,
    Unrestricted,
}

/// Broad classification of a mob controlling its spawn cap and persistence rules.
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

/// Pairs a raw entity name string with its deserialized [`EntityType`] data for token generation.
pub struct NamedEntityType<'a>(&'a str, &'a EntityType);

impl ToTokens for NamedEntityType<'_> {
    /// Emits an `EntityType { … }` struct literal token stream for the wrapped entity.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = self.0;
        let entity = self.1;
        let id = LitInt::new(&entity.id.to_string(), proc_macro2::Span::call_site());

        let attribute_tokens = entity
            .attributes
            .as_ref()
            .map(|vec| {
                vec.iter()
                    .map(|map| {
                        let (key, value) = map.iter().next().unwrap();
                        let key = key.strip_prefix("minecraft:").unwrap_or(key);
                        // Replace dots with underscores and uppercase for Enum naming (e.g. generic.max_health -> GENERIC_MAX_HEALTH)
                        let enum_variant =
                            format_ident!("{}", key.replace('.', "_").to_uppercase());

                        quote! { (Attributes::#enum_variant, #value) }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let attributes_field = quote! { &[#(#attribute_tokens),*] };

        let attackable = if let Some(a) = entity.attackable {
            quote! { Some(#a) }
        } else {
            quote! { None }
        };

        let death_sound = optional_sound(entity.death_sound.as_ref());
        let hurt_sound = optional_sound(entity.hurt_sound.as_ref());
        let swim_sound = optional_sound(entity.swim_sound.as_ref());
        let swim_splash_sound = optional_sound(entity.swim_splash_sound.as_ref());
        let swim_high_speed_splash_sound =
            optional_sound(entity.swim_high_speed_splash_sound.as_ref());
        let small_fall_sound = optional_sound(entity.small_fall_sound.as_ref());
        let big_fall_sound = optional_sound(entity.big_fall_sound.as_ref());

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
        assert!(
            !(entity.mob.is_none() && name != "player"),
            "missing field 'mob', entity name {name}"
        );
        assert!(
            !(entity.limit_per_chunk.is_none() && name != "player"),
            "missing field 'mob', entity name {name}"
        );
        assert!(
            !(entity.movement_emission.is_none() && name != "player"),
            "missing field 'movement_emission', entity name {name}"
        );
        let movement_emission = entity.movement_emission.unwrap_or(MovementEmission::All);
        assert!(
            !(entity.sound_source.is_none() && name != "player"),
            "missing field 'sound_source', entity name {name}"
        );
        // The player overrides `sound_source` in code, like `movement_emission`.
        let sound_source = format_ident!(
            "{}",
            entity
                .sound_source
                .as_deref()
                .unwrap_or("NEUTRAL")
                .to_pascal_case()
        );
        let mob = entity.mob.unwrap_or(false);
        let is_enemy = entity.is_enemy;
        let limit_per_chunk = entity.limit_per_chunk.unwrap_or(0);
        let can_spawn_far_from_player = entity.can_spawn_far_from_player;

        let dimension0 = entity.dimension[0];
        let dimension1 = entity.dimension[1];
        let spawn_dimensions_scale = entity.spawn_dimensions_scale;
        let experience_reward = entity.experience_reward.unwrap_or(0);
        let client_tracking_range = entity.client_tracking_range;
        let update_interval = entity.update_interval;
        let track_deltas = entity.track_deltas;

        tokens.extend(quote! {
            EntityType {
                id: #id,
                attributes: #attributes_field,
                experience_reward: #experience_reward,
                hurt_sound: #hurt_sound,
                death_sound: #death_sound,
                sound_source: SoundCategory::#sound_source,
                movement_emission: #movement_emission,
                swim_sound: #swim_sound,
                swim_splash_sound: #swim_splash_sound,
                swim_high_speed_splash_sound: #swim_high_speed_splash_sound,
                small_fall_sound: #small_fall_sound,
                big_fall_sound: #big_fall_sound,
                attackable: #attackable,
                mob: #mob,
                is_enemy: #is_enemy,
                saveable: #saveable,
                limit_per_chunk: #limit_per_chunk,
                summonable: #summonable,
                fire_immune: #fire_immune,
                category: &#spawn_category,
                can_spawn_far_from_player: #can_spawn_far_from_player,
                client_tracking_range: #client_tracking_range,
                update_interval: #update_interval,
                track_deltas: #track_deltas,
                dimension: [#dimension0, #dimension1], // Correctly construct the array
                eye_height: #eye_height,
                spawn_dimensions_scale: #spawn_dimensions_scale,
                spawn_restriction: #spawn_restriction,
                resource_name: #name,
            }
        });
    }
}

fn optional_sound(name: Option<&String>) -> TokenStream {
    name.map_or_else(
        || quote! { None },
        |name| {
            let ident = format_ident!("{}", name.to_pascal_case());
            quote! { Some(Sound::#ident) }
        },
    )
}

/// Generates the `TokenStream` for the `EntityType` struct, `MobCategory`, `SpawnRestriction`,
/// and the `from_raw`/`from_name` lookup methods.
pub fn build() -> TokenStream {
    let json: BTreeMap<String, EntityType> =
        serde_json::from_str(&fs::read_to_string("../../assets/entities.json").unwrap())
            .expect("Failed to parse entities.json");

    let mut consts = TokenStream::new();
    let mut type_from_raw_id_arms = TokenStream::new();
    let mut type_from_name = TokenStream::new();
    let mut all_variants = TokenStream::new();

    for (name, entity) in &json {
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

        all_variants.extend(quote! {
            &Self::#upper_name,
        })
    }
    quote! {
        use crate::data_component_impl::IDSetContent;
        use crate::tag::Taggable;
        use crate::tag::RegistryKey;
        use crate::attributes::Attributes;
        use crate::sound::{Sound, SoundCategory};
        use pumpkin_util::HeightMap;
        use pumpkin_util::math::boundingbox::BoundingBox;
        use pumpkin_util::math::vector3::Vector3;
        use std::hash::Hash;

        #[derive(Debug, Clone)]
        pub struct EntityType {
            pub id: u16,
            pub attributes: &'static [(Attributes, f64)],
            pub experience_reward: u32,
            pub hurt_sound: Option<Sound>,
            pub death_sound: Option<Sound>,
            pub sound_source: SoundCategory,
            pub movement_emission: MovementEmission,
            pub swim_sound: Option<Sound>,
            pub swim_splash_sound: Option<Sound>,
            pub swim_high_speed_splash_sound: Option<Sound>,
            pub small_fall_sound: Option<Sound>,
            pub big_fall_sound: Option<Sound>,
            pub attackable: Option<bool>,
            pub mob: bool,
            pub is_enemy: bool,
            pub saveable: bool,
            pub limit_per_chunk: i32,
            pub summonable: bool,
            pub fire_immune: bool,
            pub category: &'static MobCategory,
            pub can_spawn_far_from_player: bool,
            pub client_tracking_range: u32,
            pub update_interval: u32,
            pub track_deltas: bool,
            pub dimension: [f32; 2],
            pub eye_height: f32,
            pub spawn_dimensions_scale: f32,
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

        impl Taggable for EntityType {
            #[inline]
            fn tag_key() -> RegistryKey {
                RegistryKey::EntityType
            }

            #[inline]
            fn registry_key(&self) -> &str {
                self.resource_name
            }

            #[inline]
            fn registry_id(&self) -> u16 {
                self.id
            }
        }


        /// Vanilla `Entity.MovementEmission`: whether moving plays step sounds, emits game events, or both.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum MovementEmission {
            None,
            Sounds,
            Events,
            All,
        }

        impl MovementEmission {
            #[must_use]
            pub const fn emits_anything(self) -> bool {
                !matches!(self, Self::None)
            }

            #[must_use]
            pub const fn emits_sounds(self) -> bool {
                matches!(self, Self::Sounds | Self::All)
            }

            #[must_use]
            pub const fn emits_events(self) -> bool {
                matches!(self, Self::Events | Self::All)
            }
        }

        #[derive(Debug, Clone)]
        pub struct SpawnRestriction {
            pub location: SpawnLocation,
            pub heightmap: HeightMap,
        }

        #[derive(Debug, Clone)]
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

            pub const ALL: &'static [&'static Self] = &[#all_variants];

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

            #[must_use]
            pub const fn width(&self) -> f32 {
                self.dimension[0]
            }

            #[must_use]
            pub const fn height(&self) -> f32 {
                self.dimension[1]
            }

            #[must_use]
            pub fn get_spawn_bounding_box(&self, x: f64, y: f64, z: f64) -> BoundingBox {
                let half_width = f64::from(self.spawn_dimensions_scale * self.dimension[0] / 2.0);
                let height = f64::from(self.spawn_dimensions_scale * self.dimension[1]);
                BoundingBox::new(
                    Vector3::new(x - half_width, y, z - half_width),
                    Vector3::new(x + half_width, y + height, z + half_width),
                )
            }

            #[must_use]
            pub fn get_spawn_aabb(&self, x: f64, y: f64, z: f64) -> BoundingBox {
                self.get_spawn_bounding_box(x, y, z)
            }
        }

        impl IDSetContent for EntityType {
            fn registry_id(&self) -> u16 {
                Taggable::registry_id(self)
            }

            fn to_string(&self) -> String {
                Taggable::registry_key(self).to_string()
            }

            fn from_id(id: u16) -> Option<&'static Self> {
                EntityType::from_raw(id)
            }

            fn from_str(name: &str) -> Option<&'static Self> {
                EntityType::from_name(name)
            }
        }
    }
}
