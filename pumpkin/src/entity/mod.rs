use core::f32;
use std::sync::{atomic::AtomicBool, Arc};

use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::{
    entity::{EntityPose, EntityType},
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::{
    client::play::{
        CHeadRot, CSetEntityMetadata, CSpawnEntity, CTeleportEntity, CUpdateEntityRot, Metadata,
    },
    codec::var_int::VarInt,
};
use pumpkin_util::math::{
    boundingbox::{BoundingBox, BoundingBoxSize},
    get_section_cord,
    position::BlockPos,
    vector2::Vector2,
    vector3::Vector3,
    wrap_degrees,
};
use uuid::Uuid;

use crate::world::World;

pub mod ai;
pub mod mob;

pub mod living;
pub mod player;

pub type EntityId = i32;

/// Represents a not living Entity (e.g. Item, Egg, Snowball...)
pub struct Entity {
    /// A unique identifier for the entity
    pub entity_id: EntityId,
    /// A persistent, unique identifier for the entity
    pub entity_uuid: uuid::Uuid,
    /// The type of entity (e.g., player, zombie, item)
    pub entity_type: EntityType,
    /// The world in which the entity exists.
    pub world: Arc<World>,
    /// The entity's current position in the world
    pub pos: AtomicCell<Vector3<f64>>,
    /// The entity's position rounded to the nearest block coordinates
    pub block_pos: AtomicCell<BlockPos>,
    /// The chunk coordinates of the entity's current position
    pub chunk_pos: AtomicCell<Vector2<i32>>,
    /// Indicates whether the entity is sneaking
    pub sneaking: AtomicBool,
    /// Indicates whether the entity is sprinting
    pub sprinting: AtomicBool,
    /// Indicates whether the entity is flying due to a fall
    pub fall_flying: AtomicBool,
    /// The entity's current velocity vector, aka Knockback
    pub velocity: AtomicCell<Vector3<f64>>,
    /// Indicates whether the entity is on the ground (may not always be accurate).
    pub on_ground: AtomicBool,
    /// The entity's yaw rotation (horizontal rotation) ← →
    pub yaw: AtomicCell<f32>,
    /// The entity's head yaw rotation (horizontal rotation of the head)
    pub head_yaw: AtomicCell<f32>,
    /// The entity's pitch rotation (vertical rotation) ↑ ↓
    pub pitch: AtomicCell<f32>,
    /// The height of the entity's eyes from the ground.
    pub standing_eye_height: f32,
    /// The entity's current pose (e.g., standing, sitting, swimming).
    pub pose: AtomicCell<EntityPose>,
    /// The bounding box of an entity (hitbox)
    pub bounding_box: AtomicCell<BoundingBox>,
    ///The size (width and height) of the bounding box
    pub bounding_box_size: AtomicCell<BoundingBoxSize>,
}

impl Entity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity_id: EntityId,
        entity_uuid: uuid::Uuid,
        world: Arc<World>,
        position: Vector3<f64>,
        entity_type: EntityType,
        standing_eye_height: f32,
        bounding_box: AtomicCell<BoundingBox>,
        bounding_box_size: AtomicCell<BoundingBoxSize>,
    ) -> Self {
        let floor_x = position.x.floor() as i32;
        let floor_y = position.y.floor() as i32;
        let floor_z = position.z.floor() as i32;

        Self {
            entity_id,
            entity_uuid,
            entity_type,
            on_ground: AtomicBool::new(false),
            pos: AtomicCell::new(position),
            block_pos: AtomicCell::new(BlockPos(Vector3::new(floor_x, floor_y, floor_z))),
            chunk_pos: AtomicCell::new(Vector2::new(floor_x, floor_z)),
            sneaking: AtomicBool::new(false),
            world,
            // TODO: Load this from previous instance
            sprinting: AtomicBool::new(false),
            fall_flying: AtomicBool::new(false),
            yaw: AtomicCell::new(0.0),
            head_yaw: AtomicCell::new(0.0),
            pitch: AtomicCell::new(0.0),
            velocity: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            standing_eye_height,
            pose: AtomicCell::new(EntityPose::Standing),
            bounding_box,
            bounding_box_size,
        }
    }

    /// Updates the entity's position, block position, and chunk position.
    ///
    /// This function calculates the new position, block position, and chunk position based on the provided coordinates. If any of these values change, the corresponding fields are updated.
    pub fn set_pos(&self, new_position: Vector3<f64>) {
        let pos = self.pos.load();
        if pos != new_position {
            self.pos.store(new_position);
            self.bounding_box.store(BoundingBox::new_from_pos(
                pos.x,
                pos.y,
                pos.z,
                &self.bounding_box_size.load(),
            ));

            let floor_x = new_position.x.floor() as i32;
            let floor_y = new_position.y.floor() as i32;
            let floor_z = new_position.z.floor() as i32;

            let block_pos = self.block_pos.load();
            let block_pos_vec = block_pos.0;
            if floor_x != block_pos_vec.x
                || floor_y != block_pos_vec.y
                || floor_z != block_pos_vec.z
            {
                let new_block_pos = Vector3::new(floor_x, floor_y, floor_z);
                self.block_pos.store(BlockPos(new_block_pos));

                let chunk_pos = self.chunk_pos.load();
                if get_section_cord(floor_x) != chunk_pos.x
                    || get_section_cord(floor_z) != chunk_pos.z
                {
                    self.chunk_pos.store(Vector2::new(
                        get_section_cord(new_block_pos.x),
                        get_section_cord(new_block_pos.z),
                    ));
                }
            }
        }
    }

    /// Changes this entity's pitch and yaw to look at target
    pub async fn look_at(&self, target: Vector3<f64>) {
        let position = self.pos.load();
        let delta = target.sub(&position);
        let root = delta.x.hypot(delta.z);
        let pitch = wrap_degrees(-delta.y.atan2(root) as f32 * 180.0 / f32::consts::PI);
        let yaw = wrap_degrees((delta.z.atan2(delta.x) as f32 * 180.0 / f32::consts::PI) - 90.0);
        self.pitch.store(pitch);
        self.yaw.store(yaw);

        // send packet
        // TODO: do caching, only send packet when needed
        let yaw = (yaw * 256.0 / 360.0).rem_euclid(256.0);
        let pitch = (pitch * 256.0 / 360.0).rem_euclid(256.0);
        self.world
            .broadcast_packet_all(&CUpdateEntityRot::new(
                self.entity_id.into(),
                yaw as u8,
                pitch as u8,
                self.on_ground.load(std::sync::atomic::Ordering::Relaxed),
            ))
            .await;
        self.world
            .broadcast_packet_all(&CHeadRot::new(self.entity_id.into(), yaw as u8))
            .await;
    }

    pub async fn teleport(&self, position: Vector3<f64>, yaw: f32, pitch: f32) {
        self.world
            .broadcast_packet_all(&CTeleportEntity::new(
                self.entity_id.into(),
                position,
                Vector3::new(0.0, 0.0, 0.0),
                yaw,
                pitch,
                // TODO
                &[],
                self.on_ground.load(std::sync::atomic::Ordering::SeqCst),
            ))
            .await;
        self.set_pos(position);
        self.set_rotation(yaw, pitch);
    }

    /// Sets the Entity yaw & pitch Rotation
    pub fn set_rotation(&self, yaw: f32, pitch: f32) {
        // TODO
        self.yaw.store(yaw);
        self.pitch.store(pitch);
    }

    /// Removes the Entity from their current World
    pub async fn remove(&self) {
        self.world.remove_entity(self).await;
    }

    pub fn create_spawn_packet(&self, uuid: Uuid) -> CSpawnEntity {
        let entity_loc = self.pos.load();
        let entity_vel = self.velocity.load();
        CSpawnEntity::new(
            VarInt(self.entity_id),
            uuid,
            VarInt((self.entity_type) as i32),
            entity_loc.x,
            entity_loc.y,
            entity_loc.z,
            self.pitch.load(),
            self.yaw.load(),
            self.head_yaw.load(), // todo: head_yaw and yaw are swapped, find out why
            0.into(),
            entity_vel.x as f32,
            entity_vel.y as f32,
            entity_vel.z as f32,
        )
    }

    /// Applies knockback to the entity, following vanilla Minecraft's mechanics.
    ///
    /// This function calculates the entity's new velocity based on the specified knockback strength and direction.
    pub fn knockback(&self, strength: f64, x: f64, z: f64) {
        // This has some vanilla magic
        let mut x = x;
        let mut z = z;
        while x.mul_add(x, z * z) < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;
        let velocity = self.velocity.load();
        self.velocity.store(Vector3::new(
            velocity.x / 2.0 - var8.x,
            if self.on_ground.load(std::sync::atomic::Ordering::Relaxed) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - var8.z,
        ));
    }

    pub async fn set_sneaking(&self, sneaking: bool) {
        assert!(self.sneaking.load(std::sync::atomic::Ordering::Relaxed) != sneaking);
        self.sneaking
            .store(sneaking, std::sync::atomic::Ordering::Relaxed);
        self.set_flag(Flag::Sneaking, sneaking).await;
        if sneaking {
            self.set_pose(EntityPose::Crouching).await;
        } else {
            self.set_pose(EntityPose::Standing).await;
        }
    }

    pub async fn set_sprinting(&self, sprinting: bool) {
        assert!(self.sprinting.load(std::sync::atomic::Ordering::Relaxed) != sprinting);
        self.sprinting
            .store(sprinting, std::sync::atomic::Ordering::Relaxed);
        self.set_flag(Flag::Sprinting, sprinting).await;
    }

    pub fn check_fall_flying(&self) -> bool {
        !self.on_ground.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn set_fall_flying(&self, fall_flying: bool) {
        assert!(self.fall_flying.load(std::sync::atomic::Ordering::Relaxed) != fall_flying);
        self.fall_flying
            .store(fall_flying, std::sync::atomic::Ordering::Relaxed);
        self.set_flag(Flag::FallFlying, fall_flying).await;
    }

    async fn set_flag(&self, flag: Flag, value: bool) {
        let index = flag as u8;
        let mut b = 0i8;
        if value {
            b |= 1 << index;
        } else {
            b &= !(1 << index);
        }
        let packet = CSetEntityMetadata::new(self.entity_id.into(), Metadata::new(0, 0.into(), b));
        self.world.broadcast_packet_all(&packet).await;
    }

    /// Plays sound at this entity's position with the entity's sound category
    pub async fn play_sound(&self, sound: Sound) {
        self.world
            .play_sound(sound, SoundCategory::Neutral, &self.pos.load())
            .await;
    }

    pub async fn set_pose(&self, pose: EntityPose) {
        self.pose.store(pose);
        let pose = pose as i32;
        let packet = CSetEntityMetadata::<VarInt>::new(
            self.entity_id.into(),
            Metadata::new(6, 21.into(), pose.into()),
        );
        self.world.broadcast_packet_all(&packet).await;
    }
}

#[async_trait]
impl NBTStorage for Entity {
    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let position = self.pos.load();
        nbt.put(
            "Pos",
            NbtTag::List(vec![
                position.x.into(),
                position.y.into(),
                position.z.into(),
            ]),
        );
        let velocity = self.velocity.load();
        nbt.put(
            "Motion",
            NbtTag::List(vec![
                velocity.x.into(),
                velocity.y.into(),
                velocity.z.into(),
            ]),
        );
        nbt.put(
            "Rotation",
            NbtTag::List(vec![self.yaw.load().into(), self.pitch.load().into()]),
        );

        // todo more...
    }

    async fn read_nbt(&mut self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let position = nbt.get_list("Pos").unwrap();
        let x = position[0].extract_double().unwrap_or(0.0);
        let y = position[1].extract_double().unwrap_or(0.0);
        let z = position[2].extract_double().unwrap_or(0.0);
        self.pos.store(Vector3::new(x, y, z));
        let velocity = nbt.get_list("Motion").unwrap();
        let x = velocity[0].extract_double().unwrap_or(0.0);
        let y = velocity[1].extract_double().unwrap_or(0.0);
        let z = velocity[2].extract_double().unwrap_or(0.0);
        self.velocity.store(Vector3::new(x, y, z));
        let rotation = nbt.get_list("Rotation").unwrap();
        let yaw = rotation[0].extract_float().unwrap_or(0.0);
        let pitch = rotation[1].extract_float().unwrap_or(0.0);
        self.yaw.store(yaw);
        self.pitch.store(pitch);

        // todo more...
    }
}

#[async_trait]
pub trait NBTStorage: Send + Sync {
    async fn write_nbt(&self, nbt: &mut NbtCompound);

    async fn read_nbt(&mut self, nbt: &mut NbtCompound);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Represents various entity flags that are sent in entity metadata.
///
/// These flags are used by the client to modify the rendering of entities based on their current state.
///
/// **Purpose:**
///
/// This enum provides a more type-safe and readable way to represent entity flags compared to using raw integer values.
#[repr(u8)]
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
