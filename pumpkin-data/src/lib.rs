#![allow(unused_imports)]

#[rustfmt::skip]
#[path = "generated/item.rs"]
pub mod item;

#[rustfmt::skip]
#[path = "generated/packet.rs"]
pub mod packet;

#[rustfmt::skip]
#[path = "generated/screen.rs"]
pub mod screen;

#[rustfmt::skip]
#[path = "generated/particle.rs"]
pub mod particle;

#[rustfmt::skip]
#[path = "generated/sound_category.rs"]
mod sound_category;

#[rustfmt::skip]
#[path = "generated/sound.rs"]
mod sound_enum;

pub mod sound {
    pub use crate::sound_category::*;
    pub use crate::sound_enum::*;
}

#[rustfmt::skip]
#[path = "generated/noise_parameter.rs"]
pub mod noise_parameter;

#[rustfmt::skip]
#[path = "generated/biome.rs"]
pub mod biome;

#[rustfmt::skip]
#[path = "generated/chunk_status.rs"]
pub mod chunk_status;

pub mod chunk {
    pub use super::biome::*;
    pub use super::chunk_status::ChunkStatus;
    pub use super::noise_parameter::*;
}

#[rustfmt::skip]
#[path = "generated/game_event.rs"]
pub mod game_event;

#[rustfmt::skip]
#[path = "generated/entity_pose.rs"]
mod entity_pose;

#[rustfmt::skip]
#[path = "generated/entity_status.rs"]
mod entity_status;

#[rustfmt::skip]
#[path = "generated/entity_type.rs"]
mod entity_type;

#[rustfmt::skip]
#[path = "generated/spawn_egg.rs"]
mod spawn_egg;

#[rustfmt::skip]
#[path = "generated/status_effect.rs"]
mod status_effect;

pub mod entity {
    pub use super::entity_pose::*;
    pub use super::entity_status::*;
    pub use super::entity_type::*;
    pub use super::spawn_egg::*;
    pub use super::status_effect::*;
}

#[rustfmt::skip]
#[path = "generated/world_event.rs"]
mod world_event;

#[rustfmt::skip]
#[path = "generated/message_type.rs"]
mod message_type;

pub mod world {
    pub use super::message_type::*;
    pub use super::world_event::*;
}

#[rustfmt::skip]
#[path = "generated/scoreboard_slot.rs"]
pub mod scoreboard;

#[rustfmt::skip]
#[path = "generated/damage_type.rs"]
pub mod damage;

#[rustfmt::skip]
#[path = "generated/fluid.rs"]
pub mod fluid;

#[rustfmt::skip]
#[path = "generated/block.rs"]
pub mod block_properties;

#[rustfmt::skip]
#[path = "generated/tag.rs"]
pub mod tag;

#[rustfmt::skip]
#[path = "generated/noise_router.rs"]
pub mod noise_router;

mod block_direction;
pub mod block_state;
mod blocks;
mod collision_shape;

pub use block_direction::BlockDirection;
pub use block_direction::FacingExt;
pub use block_direction::HorizontalFacingExt;
pub use block_state::BlockState;
pub use block_state::BlockStateRef;
pub use blocks::Block;
pub use collision_shape::CollisionShape;
