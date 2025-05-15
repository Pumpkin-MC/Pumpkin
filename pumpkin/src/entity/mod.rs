use crate::server::Server;
use async_trait::async_trait;
use bytes::BufMut;
use core::f32;
use crossbeam::atomic::AtomicCell;
use player::Player;
use pumpkin_data::{
    block_properties::{Facing, HorizontalFacing},
    damage::DamageType,
    entity::{EntityPose, EntityType},
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::{
    client::play::{
        CEntityPositionSync, CEntityVelocity, CHeadRot, CSetEntityMetadata, CSpawnEntity,
        CUpdateEntityRot, MetaDataType, Metadata,
    },
    codec::var_int::VarInt,
    ser::serializer::Serializer,
};
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    get_section_cord,
    position::BlockPos,
    vector2::Vector2,
    vector3::Vector3,
    wrap_degrees,
};
use pumpkin_world::entity::{Entity, EntityBase};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{
            AtomicBool, AtomicI32,
            Ordering::{Relaxed, SeqCst},
        },
    },
};
use enum_dispatch::enum_dispatch;
use tokio::sync::RwLock;
use r#type::entity_base_from_type;
use uuid::Uuid;

use crate::world::World;

pub mod ai;
pub mod effect;
pub mod experience_orb;
pub mod hunger;
pub mod item;
pub mod living;
pub mod non_living;
pub mod player;
pub mod projectile;
pub mod tnt;
pub mod r#type;

mod combat;

pub type EntityId = i32;

#[enum_dispatch]
#[async_trait]
pub trait WorldEntityExt: EntityBase {
    /// Gets Called every tick
    async fn tick(&mut self, server: &Server, world: &World);

    /// Returns if damage was successful or not
    async fn damage(&self, amount: f32, damage_type: DamageType, world: &World) -> bool;

    async fn init_data_tracker(&mut self, world: &World);

    /// Called when a player collides with a entity
    async fn on_player_collision(&self, player: Arc<Player>, world: &World);

    async fn send_meta_data<T>(&self, meta: &[Metadata<T>], world: &World)
    where
        T: Serialize,
    {
        let mut buf = Vec::new();
        for meta in meta {
            let mut serializer_buf = Vec::new();
            let mut serializer = Serializer::new(&mut serializer_buf);
            meta.serialize(&mut serializer).unwrap();
            buf.extend(serializer_buf);
        }
        buf.put_u8(255);
        // TODO: don't broadcast to all
        world
            .broadcast_packet_all(&CSetEntityMetadata::new(self.id().into(), buf.into()))
            .await;
    }

    async fn global_tick(&mut self, server: &Server, world: &World) {
        // Movement
        let velocity = self.get_velocity();
        let pos = self.pos();

        world
            .level
            .entity_manager
            .write()
            .await
            .set_position(self, pos + velocity)
            .await;
        // TODO: Update velocity

        self.tick(server, world).await;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Represents various entity flags that are sent in entity metadata.
///
/// These flags are used by the client to modify the rendering of entities based on their current state.
///
/// **Purpose:**
///
/// This enum provides a more type-safe and readable way to represent entity flags compared to using raw integer values.
pub enum Flag {
    /// Indicates if the entity is on fire.
    OnFire = 0,
    /// Indicates if the entity is sneaking.
    Sneaking = 1,
    /// Indicates if the entity is sprinting.
    Sprinting = 3,
    /// Indicates if the entity is swimming.
    Swimming = 4,
    /// Indicates if the entity is invisible.
    Invisible = 5,
    /// Indicates if the entity is glowing.
    Glowing = 6,
    /// Indicates if the entity is flying due to a fall.
    FallFlying = 7,
}
