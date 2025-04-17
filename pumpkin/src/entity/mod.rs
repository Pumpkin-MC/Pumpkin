use crate::server::Server;
use async_trait::async_trait;
use bytes::BufMut;
use core::f32;
use crossbeam::atomic::AtomicCell;
use living::LivingEntity;
use player::Player;
use pumpkin_data::{
    block::{Facing, HorizontalFacing},
    damage::DamageType,
    entity::{EntityPose, EntityType},
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::{
    client::play::{
        CEntityVelocity, CHeadRot, CSetEntityMetadata, CSpawnEntity, CTeleportEntity,
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
use serde::Serialize;
use std::{
    f32::consts::PI,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
};
use tokio::sync::RwLock;

use crate::world::World;

pub mod ai;
pub mod effect;
pub mod experience_orb;
pub mod hunger;
pub mod item;
pub mod living;
pub mod mob;
pub mod player;
pub mod projectile;
pub mod tnt;

mod combat;

pub type EntityId = i32;

#[async_trait]
pub trait EntityBase: Send + Sync {
    /// Gets Called every tick
    async fn tick(&self, server: &Server) {
        if let Some(living) = self.get_living_entity() {
            living.tick(server).await;
        } else {
            self.get_entity().tick(server).await;
        }
    }

    /// Returns if damage was successful or not
    async fn damage(&self, amount: f32, damage_type: DamageType) -> bool {
        if let Some(living) = self.get_living_entity() {
            living.damage(amount, damage_type).await
        } else {
            self.get_entity().damage(amount, damage_type).await
        }
    }

    /// Called when a player collides with a entity
    async fn on_player_collision(&self, _player: Arc<Player>) {}
    fn get_entity(&self) -> &Entity;
    fn get_living_entity(&self) -> Option<&LivingEntity>;
}

static CURRENT_ID: AtomicI32 = AtomicI32::new(0);

/// Represents a non-living Entity (e.g. Item, Egg, Snowball...)
pub struct Entity {
    /// A unique identifier for the entity
    pub entity_id: EntityId,
    /// A persistent, unique identifier for the entity
    pub entity_uuid: uuid::Uuid,
    /// The type of entity (e.g., player, zombie, item)
    pub entity_type: EntityType,
    /// The world in which the entity exists.
    pub world: Arc<RwLock<Arc<World>>>,
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
    /// The entity's current velocity vector, aka knockback
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
    pub bounding_box_size: AtomicCell<EntityDimensions>,
    /// Whether this entity is invulnerable to all damage
    pub invulnerable: AtomicBool,
    /// List of damage types this entity is immune to
    pub damage_immunities: Vec<DamageType>,
}

impl Entity {
    pub fn new(
        entity_uuid: uuid::Uuid,
        world: Arc<World>,
        position: Vector3<f64>,
        entity_type: EntityType,
        invulnerable: bool,
    ) -> Self {
        let floor_x = position.x.floor() as i32;
        let floor_y = position.y.floor() as i32;
        let floor_z = position.z.floor() as i32;

        let bounding_box_size = EntityDimensions {
            width: entity_type.dimension[0],
            height: entity_type.dimension[1],
        };

        Self {
            entity_id: CURRENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            entity_uuid,
            entity_type,
            on_ground: AtomicBool::new(false),
            pos: AtomicCell::new(position),
            block_pos: AtomicCell::new(BlockPos(Vector3::new(floor_x, floor_y, floor_z))),
            chunk_pos: AtomicCell::new(Vector2::new(floor_x, floor_z)),
            sneaking: AtomicBool::new(false),
            world: Arc::new(RwLock::new(world)),
            sprinting: AtomicBool::new(false),
            fall_flying: AtomicBool::new(false),
            yaw: AtomicCell::new(0.0),
            head_yaw: AtomicCell::new(0.0),
            pitch: AtomicCell::new(0.0),
            velocity: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
            standing_eye_height: entity_type.eye_height,
            pose: AtomicCell::new(EntityPose::Standing),
            bounding_box: AtomicCell::new(BoundingBox::new_from_pos(
                position.x,
                position.y,
                position.z,
                &bounding_box_size,
            )),
            bounding_box_size: AtomicCell::new(bounding_box_size),
            invulnerable: AtomicBool::new(invulnerable),
            damage_immunities: Vec::new(),
        }
    }

    pub async fn set_velocity(&self, velocity: Vector3<f64>) {
        self.velocity.store(velocity);
        self.world
            .read()
            .await
            .broadcast_packet_all(&CEntityVelocity::new(self.entity_id.into(), velocity))
            .await;
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

    /// Returns entity rotation as vector
    pub fn rotation(&self) -> Vector3<f32> {
        // Convert degrees to radians if necessary
        let yaw_rad = self.yaw.load().to_radians();
        let pitch_rad = self.pitch.load().to_radians();

        Vector3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        )
        .normalize()
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

        // Broadcast the update packet.
        // TODO: Do caching to only send the packet when needed.
        let yaw = (yaw * 256.0 / 360.0).rem_euclid(256.0);
        let pitch = (pitch * 256.0 / 360.0).rem_euclid(256.0);
        self.world
            .read()
            .await
            .broadcast_packet_all(&CUpdateEntityRot::new(
                self.entity_id.into(),
                yaw as u8,
                pitch as u8,
                self.on_ground.load(std::sync::atomic::Ordering::Relaxed),
            ))
            .await;
        self.world
            .read()
            .await
            .broadcast_packet_all(&CHeadRot::new(self.entity_id.into(), yaw as u8))
            .await;
    }

    pub async fn teleport(&self, position: Vector3<f64>, yaw: f32, pitch: f32) {
        self.world
            .read()
            .await
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

    /// Sets the `Entity` yaw & pitch rotation
    pub fn set_rotation(&self, yaw: f32, pitch: f32) {
        // TODO
        self.yaw.store(yaw);
        self.pitch.store(pitch.clamp(-90.0, 90.0) % 360.0);
    }

    /// Removes the `Entity` from their current `World`
    pub async fn remove(&self) {
        self.world.read().await.remove_entity(self).await;
    }

    pub fn create_spawn_packet(&self) -> CSpawnEntity {
        let entity_loc = self.pos.load();
        let entity_vel = self.velocity.load();
        CSpawnEntity::new(
            VarInt(self.entity_id),
            self.entity_uuid,
            VarInt(i32::from(self.entity_type.id)),
            entity_loc,
            self.pitch.load(),
            self.yaw.load(),
            self.head_yaw.load(), // todo: head_yaw and yaw are swapped, find out why
            0.into(),
            entity_vel,
        )
    }
    pub fn width(&self) -> f32 {
        self.bounding_box_size.load().width
    }

    pub fn height(&self) -> f32 {
        self.bounding_box_size.load().height
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

    pub fn get_horizontal_facing(&self) -> HorizontalFacing {
        let adjusted_yaw = (self.yaw.load() % 360.0 + 360.0) % 360.0; // Normalize yaw to [0, 360)

        match adjusted_yaw {
            0.0..=45.0 | 315.0..=360.0 => HorizontalFacing::South,
            45.0..=135.0 => HorizontalFacing::West,
            135.0..=225.0 => HorizontalFacing::North,
            225.0..=315.0 => HorizontalFacing::East,
            _ => HorizontalFacing::South, // Default case, should not occur
        }
    }

    pub fn get_facing(&self) -> Facing {
        let pitch = self.pitch.load() * (PI / 180.0);
        let yaw = -self.yaw.load() * (PI / 180.0);

        let sin_pitch = pitch.sin();
        let cos_pitch = pitch.cos();
        let sin_yaw = yaw.sin();
        let cos_yaw = yaw.cos();

        let abs_sin_yaw = sin_yaw.abs();
        let abs_sin_pitch = sin_pitch.abs();
        let abs_cos_yaw = cos_yaw.abs();

        let o = abs_sin_yaw * cos_pitch.abs();

        if abs_sin_yaw > abs_cos_yaw {
            if abs_sin_pitch > o {
                if sin_pitch < 0.0 {
                    Facing::Up
                } else {
                    Facing::Down
                }
            } else if sin_yaw > 0.0 {
                Facing::East
            } else {
                Facing::West
            }
        } else if cos_yaw > 0.0 {
            Facing::South
        } else {
            Facing::North
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
        self.send_meta_data(&[Metadata::new(0, MetaDataType::Byte, b)])
            .await;
    }

    /// Plays sound at this entity's position with the entity's sound category
    pub async fn play_sound(&self, sound: Sound) {
        self.world
            .read()
            .await
            .play_sound(sound, SoundCategory::Neutral, &self.pos.load())
            .await;
    }

    pub async fn send_meta_data<T>(&self, meta: &[Metadata<T>])
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
        self.world
            .read()
            .await
            .broadcast_packet_all(&CSetEntityMetadata::new(self.entity_id.into(), buf.into()))
            .await;
    }

    pub async fn set_pose(&self, pose: EntityPose) {
        self.pose.store(pose);
        let pose = pose as i32;
        self.send_meta_data(&[Metadata::new(6, MetaDataType::EntityPose, VarInt(pose))])
            .await;
    }

    pub fn is_invulnerable_to(&self, damage_type: &DamageType) -> bool {
        self.invulnerable.load(std::sync::atomic::Ordering::Relaxed)
            || self.damage_immunities.contains(damage_type)
    }

    fn velocity_multiplier(_pos: Vector3<f64>) -> f32 {
        // let world = self.world.read().await;
        // TODO: handle when player is outside world
        // let block = world.get_block(&self.block_pos.load()).await;
        // block.velocity_multiplier
        0.0
        // if velo_multiplier == 1.0 {
        //     const VELOCITY_OFFSET: f64 = 0.500001; // Vanilla
        //     let pos_with_y_offset = BlockPos(Vector3::new(
        //         pos.x.floor() as i32,
        //         (pos.y - VELOCITY_OFFSET).floor() as i32,
        //         pos.z.floor() as i32,
        //     ));
        //     let block = world.get_block(&pos_with_y_offset).await.unwrap();
        //     block.velocity_multiplier
        // } else {
        // }
    }

    fn tick_move(&self) {
        let velo = self.velocity.load();
        let pos = self.pos.load();
        self.pos
            .store(Vector3::new(pos.x + velo.x, pos.y + velo.y, pos.z + velo.z));
        let multiplier = f64::from(Self::velocity_multiplier(pos));
        self.velocity
            .store(velo.multiply(multiplier, 1.0, multiplier));
    }
}

#[async_trait]
impl EntityBase for Entity {
    async fn damage(&self, _amount: f32, _damage_type: DamageType) -> bool {
        false
    }

    async fn tick(&self, _: &Server) {
        self.tick_move();
    }

    fn get_entity(&self) -> &Entity {
        self
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }
}

#[async_trait]
impl NBTStorage for Entity {
    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let position = self.pos.load();
        nbt.put(
            "Pos",
            NbtTag::List(
                vec![position.x.into(), position.y.into(), position.z.into()].into_boxed_slice(),
            ),
        );
        let velocity = self.velocity.load();
        nbt.put(
            "Motion",
            NbtTag::List(
                vec![velocity.x.into(), velocity.y.into(), velocity.z.into()].into_boxed_slice(),
            ),
        );
        nbt.put(
            "Rotation",
            NbtTag::List(vec![self.yaw.load().into(), self.pitch.load().into()].into_boxed_slice()),
        );
        nbt.put_bool("OnGround", self.on_ground.load(Ordering::Relaxed));

        // todo more...
    }

    async fn read_nbt(&mut self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let position = nbt.get_list("Pos").unwrap();
        let x = position[0].extract_double().unwrap_or(0.0);
        let y = position[1].extract_double().unwrap_or(0.0);
        let z = position[2].extract_double().unwrap_or(0.0);
        self.set_pos(Vector3::new(x, y, z));
        let velocity = nbt.get_list("Motion").unwrap();
        let x = velocity[0].extract_double().unwrap_or(0.0);
        let y = velocity[1].extract_double().unwrap_or(0.0);
        let z = velocity[2].extract_double().unwrap_or(0.0);
        self.velocity.store(Vector3::new(x, y, z));
        let rotation = nbt.get_list("Rotation").unwrap();
        let yaw = rotation[0].extract_float().unwrap_or(0.0);
        let pitch = rotation[1].extract_float().unwrap_or(0.0);
        self.set_rotation(yaw, pitch);
        self.head_yaw.store(yaw);
        self.on_ground
            .store(nbt.get_bool("OnGround").unwrap_or(false), Ordering::Relaxed);
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
