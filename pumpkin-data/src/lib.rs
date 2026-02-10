#![allow(unused_imports)]

#[cfg(feature = "item")]
#[rustfmt::skip]
#[path = "generated/item.rs"]
pub mod item;

#[cfg(feature = "packet")]
#[rustfmt::skip]
#[path = "generated/packet.rs"]
pub mod packet;

#[cfg(feature = "jukebox_song")]
#[rustfmt::skip]
#[path = "generated/jukebox_song.rs"]
pub mod jukebox_song;

#[cfg(feature = "translation")]
#[rustfmt::skip]
#[path = "generated/translation.rs"]
pub mod translation;

#[cfg(feature = "registry")]
#[rustfmt::skip]
#[path = "generated/registry.rs"]
pub mod registry;

#[cfg(feature = "screen")]
#[rustfmt::skip]
#[path = "generated/screen.rs"]
pub mod screen;

#[cfg(feature = "particle")]
#[rustfmt::skip]
#[path = "generated/particle.rs"]
pub mod particle;

#[cfg(feature = "sound")]
#[rustfmt::skip]
#[path = "generated/sound_category.rs"]
mod sound_category;

#[cfg(feature = "sound")]
#[rustfmt::skip]
#[path = "generated/sound.rs"]
mod sound_enum;

#[cfg(feature = "sound")]
pub mod sound {
    pub use crate::sound_category::*;
    pub use crate::sound_enum::*;
}

#[cfg(feature = "recipes")]
#[rustfmt::skip]
#[path = "generated/recipes.rs"]
pub mod recipes;

#[cfg(feature = "data_component")]
#[rustfmt::skip]
#[path = "generated/data_component.rs"]
pub mod data_component;

#[cfg(feature = "data_component")]
pub mod data_component_impl;

#[cfg(feature = "attributes")]
#[rustfmt::skip]
#[path = "generated/attributes.rs"]
pub mod attributes;

#[cfg(feature = "tracked_data")]
#[rustfmt::skip]
#[path = "generated/tracked_data.rs"]
pub mod tracked_data;

#[cfg(feature = "meta_data_type")]
#[rustfmt::skip]
#[path = "generated/meta_data_type.rs"]
pub mod meta_data_type;

#[cfg(feature = "noise_parameter")]
#[rustfmt::skip]
#[path = "generated/noise_parameter.rs"]
pub mod noise_parameter;

#[cfg(feature = "biome")]
#[rustfmt::skip]
#[path = "generated/biome.rs"]
pub mod biome;

#[cfg(feature = "chunk_status")]
#[rustfmt::skip]
#[path = "generated/chunk_status.rs"]
pub mod chunk_status;

#[cfg(feature = "chunk")]
pub mod chunk {
    #[cfg(feature = "biome")]
    pub use super::biome::*;
    #[cfg(feature = "chunk_status")]
    pub use super::chunk_status::ChunkStatus;
    #[cfg(feature = "noise_parameter")]
    pub use super::noise_parameter::*;
}

#[cfg(feature = "game_event")]
#[rustfmt::skip]
#[path = "generated/game_event.rs"]
pub mod game_event;

#[cfg(feature = "game_rules")]
#[rustfmt::skip]
#[path ="generated/game_rules.rs"]
pub mod game_rules;

#[cfg(feature = "entity_pose")]
#[rustfmt::skip]
#[path = "generated/entity_pose.rs"]
mod entity_pose;

#[cfg(feature = "entity_status")]
#[rustfmt::skip]
#[path = "generated/entity_status.rs"]
mod entity_status;

#[cfg(feature = "entity_type")]
#[rustfmt::skip]
#[path = "generated/entity_type.rs"]
mod entity_type;

#[cfg(feature = "spawn_egg")]
#[rustfmt::skip]
#[path = "generated/spawn_egg.rs"]
mod spawn_egg;

#[cfg(feature = "dimension")]
#[rustfmt::skip]
#[path = "generated/dimension.rs"]
pub mod dimension;

#[cfg(feature = "enchantment")]
#[rustfmt::skip]
#[path = "generated/enchantment.rs"]
mod enchantment;

#[cfg(feature = "enchantment")]
pub use enchantment::*;

#[cfg(feature = "entity")]
pub mod entity {
    #[cfg(feature = "entity_pose")]
    pub use super::entity_pose::*;
    #[cfg(feature = "entity_status")]
    pub use super::entity_status::*;
    #[cfg(feature = "entity_type")]
    pub use super::entity_type::*;
    #[cfg(feature = "spawn_egg")]
    pub use super::spawn_egg::*;
}

#[cfg(feature = "world_event")]
#[rustfmt::skip]
#[path = "generated/world_event.rs"]
mod world_event;

#[cfg(feature = "message_type")]
#[rustfmt::skip]
#[path = "generated/message_type.rs"]
mod message_type;

#[cfg(feature = "world")]
pub mod world {
    #[cfg(feature = "message_type")]
    pub use super::message_type::*;
    #[cfg(feature = "world_event")]
    pub use super::world_event::*;
}

#[cfg(feature = "scoreboard")]
#[rustfmt::skip]
#[path = "generated/scoreboard_slot.rs"]
pub mod scoreboard;

#[cfg(feature = "damage")]
#[rustfmt::skip]
#[path = "generated/damage_type.rs"]
pub mod damage;

#[cfg(feature = "fluid")]
#[rustfmt::skip]
#[path = "generated/fluid.rs"]
pub mod fluid;

#[cfg(feature = "block")]
#[rustfmt::skip]
#[path = "generated/block.rs"]
pub mod block_properties;

#[cfg(feature = "block")]
#[rustfmt::skip]
#[path = "generated/block_state_remap.rs"]
pub mod block_state_remap;

#[cfg(feature = "item_id_remap")]
#[rustfmt::skip]
#[path = "generated/item_id_remap.rs"]
pub mod item_id_remap;

#[cfg(feature = "tag")]
#[rustfmt::skip]
#[path = "generated/tag.rs"]
pub mod tag;

#[cfg(feature = "noise_router")]
#[rustfmt::skip]
#[path = "generated/noise_router.rs"]
pub mod noise_router;

#[cfg(feature = "composter")]
#[rustfmt::skip]
#[path = "generated/composter_increase_chance.rs"]
pub mod composter_increase_chance;

#[cfg(feature = "flower_pot")]
#[rustfmt::skip]
#[path = "generated/flower_pot_transformations.rs"]
pub mod flower_pot_transformations;

#[cfg(feature = "fuels")]
#[rustfmt::skip]
#[path = "generated/fuels.rs"]
pub mod fuels;

#[cfg(feature = "effect")]
#[rustfmt::skip]
#[path = "generated/effect.rs"]
pub mod effect;

#[cfg(feature = "structures")]
#[rustfmt::skip]
#[path = "generated/structures.rs"]
pub mod structures;

#[cfg(feature = "potion")]
#[rustfmt::skip]
#[path = "generated/potion.rs"]
pub mod potion;

#[cfg(feature = "potion_brewing")]
#[rustfmt::skip]
#[path = "generated/potion_brewing.rs"]
pub mod potion_brewing;

#[cfg(feature = "recipe_remainder")]
#[rustfmt::skip]
#[path = "generated/recipe_remainder.rs"]
pub mod recipe_remainder;

#[cfg(feature = "block")]
mod block_direction;
#[cfg(feature = "block")]
pub mod block_state;
#[cfg(feature = "block")]
mod blocks;

#[cfg(feature = "block")]
pub use block_direction::BlockDirection;
#[cfg(feature = "block")]
pub use block_direction::FacingExt;
#[cfg(feature = "block")]
pub use block_direction::HorizontalFacingExt;
#[cfg(feature = "block")]
pub use block_state::BlockState;
#[cfg(feature = "block")]
pub use block_state::BlockStateRef;
#[cfg(feature = "block")]
pub use blocks::Block;

#[cfg(feature = "chunk_gen_settings")]
#[rustfmt::skip]
#[path = "generated/chunk_gen_settings.rs"]
pub mod chunk_gen_settings;
