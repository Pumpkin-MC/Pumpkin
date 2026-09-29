#![allow(
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::undocumented_unsafe_blocks,
    clippy::if_then_some_else_none,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

#[rustfmt::skip]
#[path = "generated/chunk_view_lut.rs"]
pub mod chunk_view_lut;

#[rustfmt::skip]
#[path = "generated/loot_table.rs"]
pub mod loot_table;
pub use loot_table as chest_loot_table;

#[cfg(feature = "item")]
#[rustfmt::skip]
#[path = "generated/item.rs"]
pub mod item;

#[cfg(feature = "item")]
pub mod item_stack;

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

#[cfg(feature = "statistic")]
#[rustfmt::skip]
#[path = "generated/statistic.rs"]
pub mod statistic;

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

#[cfg(feature = "advancement")]
#[rustfmt::skip]
#[path = "generated/advancement.rs"]
pub mod advancement;

#[cfg(feature = "advancement")]
pub mod advancement_data;

#[cfg(feature = "advancement")]
pub use advancement::*;

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
#[expect(clippy::unreachable)]
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
pub mod entity_status;

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

#[cfg(feature = "environment_attribute")]
#[rustfmt::skip]
#[path = "generated/environment_attribute.rs"]
pub mod environment_attribute;

#[cfg(feature = "environment_attribute")]
pub use environment_attribute::*;

#[cfg(feature = "enchantment")]
#[rustfmt::skip]
#[path = "generated/enchantment.rs"]
pub mod enchantment;

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

#[rustfmt::skip]
#[path = "generated/placed_feature.rs"]
pub mod placed_feature;

#[rustfmt::skip]
#[path = "generated/configured_feature.rs"]
pub mod configured_feature;

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
#[expect(clippy::unreachable)]
#[rustfmt::skip]
#[path = "generated/block.rs"]
pub mod block_properties;

#[cfg(feature = "bedrock_creative")]
#[rustfmt::skip]
#[path = "generated/bedrock_creative.rs"]
pub mod bedrock_creative;

#[cfg(feature = "bedrock_biome")]
#[rustfmt::skip]
#[path = "generated/bedrock_biome.rs"]
pub mod bedrock_biome;

#[cfg(feature = "tag")]
#[rustfmt::skip]
#[path = "generated/tag.rs"]
pub mod tag;

#[cfg(feature = "noise_router")]
#[rustfmt::skip]
#[path = "generated/noise_router.rs"]
pub mod noise_router;


#[cfg(feature = "flower_pot")]
#[rustfmt::skip]
#[path = "generated/flower_pot_transformations.rs"]
pub mod flower_pot_transformations;

#[cfg(feature = "effect")]
#[rustfmt::skip]
#[path = "generated/effect.rs"]
pub mod effect;

#[cfg(feature = "effect")]
#[rustfmt::skip]
#[path = "generated/status_effect.rs"]
pub mod status_effect;

#[cfg(feature = "structures")]
#[rustfmt::skip]
#[path = "generated/structures.rs"]
pub mod structures;

#[cfg(feature = "structures")]
#[rustfmt::skip]
#[path = "generated/template_pool.rs"]
pub mod template_pool;

#[cfg(feature = "structures")]
#[rustfmt::skip]
#[path = "generated/processor_list.rs"]
pub mod processor_list;

#[cfg(feature = "structures")]
#[rustfmt::skip]
#[path = "generated/structure_metadata.rs"]
pub mod structure_metadata;

#[cfg(feature = "structures")]
#[rustfmt::skip]
#[path = "generated/template_bytes.rs"]
pub mod template_bytes;

#[cfg(feature = "test_instance")]
#[rustfmt::skip]
#[path = "generated/test_instance.rs"]
pub mod test_instance;

#[cfg(feature = "painting_variant")]
#[rustfmt::skip]
#[path = "generated/painting_variant.rs"]
pub mod painting_variant;

#[cfg(feature = "context_provider")]
#[rustfmt::skip]
#[path = "generated/context_provider.rs"]
pub mod context_provider;

#[cfg(feature = "instrument")]
#[rustfmt::skip]
#[path = "generated/instrument.rs"]
pub mod instrument;

#[cfg(feature = "wolf_variant")]
#[rustfmt::skip]
#[path = "generated/wolf_variant.rs"]
pub mod wolf_variant;

#[cfg(feature = "cat_variant")]
#[rustfmt::skip]
#[path = "generated/cat_variant.rs"]
pub mod cat_variant;

#[cfg(feature = "frog_variant")]
#[rustfmt::skip]
#[path = "generated/frog_variant.rs"]
pub mod frog_variant;

#[cfg(feature = "cow_variant")]
#[rustfmt::skip]
#[path = "generated/cow_variant.rs"]
pub mod cow_variant;

#[cfg(feature = "cow_sound_variant")]
#[rustfmt::skip]
#[path = "generated/cow_sound_variant.rs"]
pub mod cow_sound_variant;

#[cfg(feature = "pig_variant")]
#[rustfmt::skip]
#[path = "generated/pig_variant.rs"]
pub mod pig_variant;

#[cfg(feature = "pig_sound_variant")]
#[rustfmt::skip]
#[path = "generated/pig_sound_variant.rs"]
pub mod pig_sound_variant;

#[cfg(feature = "chicken_variant")]
#[rustfmt::skip]
#[path = "generated/chicken_variant.rs"]
pub mod chicken_variant;

#[cfg(feature = "chicken_sound_variant")]
#[rustfmt::skip]
#[path = "generated/chicken_sound_variant.rs"]
pub mod chicken_sound_variant;

#[cfg(feature = "cat_sound_variant")]
#[rustfmt::skip]
#[path = "generated/cat_sound_variant.rs"]
pub mod cat_sound_variant;

#[cfg(feature = "wolf_sound_variant")]
#[rustfmt::skip]
#[path = "generated/wolf_sound_variant.rs"]
pub mod wolf_sound_variant;

#[cfg(feature = "zombie_nautilus_variant")]
#[rustfmt::skip]
#[path = "generated/zombie_nautilus_variant.rs"]
pub mod zombie_nautilus_variant;

#[cfg(feature = "trim_material")]
#[rustfmt::skip]
#[path = "generated/trim_material.rs"]
pub mod trim_material;

#[cfg(feature = "trim_pattern")]
#[rustfmt::skip]
#[path = "generated/trim_pattern.rs"]
pub mod trim_pattern;

#[cfg(feature = "banner_pattern")]
#[rustfmt::skip]
#[path = "generated/banner_pattern.rs"]
pub mod banner_pattern;

#[cfg(feature = "decorated_pot_pattern")]
#[rustfmt::skip]
#[path = "generated/decorated_pot_pattern.rs"]
pub mod decorated_pot_pattern;

#[cfg(feature = "chat_type")]
#[rustfmt::skip]
#[path = "generated/chat_type.rs"]
pub mod chat_type;

#[cfg(feature = "enchantment_provider")]
#[rustfmt::skip]
#[path = "generated/enchantment_provider.rs"]
pub mod enchantment_provider;

#[cfg(feature = "potion")]
#[rustfmt::skip]
#[path = "generated/potion.rs"]
pub mod potion;

#[cfg(feature = "potion_brewing")]
#[rustfmt::skip]
#[path = "generated/potion_brewing.rs"]
pub mod potion_brewing;


#[cfg(feature = "block")]
mod block_direction;
#[cfg(feature = "block")]
pub mod block_rotation;
#[cfg(feature = "block")]
pub mod block_state;
#[cfg(feature = "block")]
mod blocks;

#[cfg(feature = "block")]
pub use block_direction::{BlockDirection, FacingExt, HorizontalFacingExt};
#[cfg(feature = "block")]
pub use block_rotation::{Mirror, Rotation, transform_block_properties, transform_rail_shape};
#[cfg(feature = "block")]
pub use block_state::{BlockState, BlockStateId};
#[cfg(feature = "block")]
pub use blocks::{Block, BlockId, SpawnFloorPredicate};

#[cfg(feature = "material_rule")]
#[rustfmt::skip]
#[path = "generated/material_rule.rs"]
pub mod material_rule;

#[cfg(feature = "noise_settings")]
#[rustfmt::skip]
#[path = "generated/noise_settings.rs"]
pub mod noise_settings;

#[cfg(feature = "chunk_gen_settings")]
pub use noise_settings as chunk_gen_settings;

#[cfg(feature = "carver")]
#[rustfmt::skip]
#[path = "generated/carver.rs"]
pub mod carver;

#[cfg(feature = "villager")]
#[rustfmt::skip]
#[path = "generated/villager.rs"]
pub mod villager;

#[cfg(feature = "slot_ranges")]
#[rustfmt::skip]
#[path = "generated/slot_ranges.rs"]
pub mod slot_ranges;

#[cfg(feature = "map_color")]
#[rustfmt::skip]
#[path = "generated/map_color.rs"]
pub mod map_color;

#[cfg(feature = "map_decoration")]
#[rustfmt::skip]
#[path = "generated/map_decoration.rs"]
pub mod map_decoration;

#[cfg(feature = "dye_color")]
#[rustfmt::skip]
#[path = "generated/dye_color.rs"]
pub mod dye_color;

#[cfg(feature = "block_transformer")]
#[rustfmt::skip]
#[path = "generated/block_transformer.rs"]
pub mod block_transformer;

#[cfg(feature = "trial_spawner")]
#[rustfmt::skip]
#[path = "generated/trial_spawner.rs"]
pub mod trial_spawner;
