use core::f32;
use std::sync::{atomic::AtomicBool, Arc};

use crossbeam::atomic::AtomicCell;
use crate::server::Server;
use crossbeam::atomic::AtomicCell;
use itertools::Itertools;
use num_derive::FromPrimitive;
use num_traits::real::Real;
use pumpkin_core::math::{
    boundingbox::{BoundingBox, BoundingBoxSize},
    get_section_cord,
    position::WorldPosition,
    vector2::Vector2,
    vector3::Vector3,
    wrap_degrees,
};
use pumpkin_entity::{entity_type::EntityType, pose::EntityPose, EntityId};
use pumpkin_protocol::client::play::{CEntityVelocity, CSpawnEntity, CUpdateEntityPos};
use pumpkin_protocol::{
    client::play::{CHeadRot, CSetEntityMetadata, CTeleportEntity, CUpdateEntityRot, Metadata},
    codec::var_int::VarInt,
};
use pumpkin_world::coordinates::ChunkRelativeBlockCoordinates;
use rand::Rng;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};
use uuid::Uuid;

use crate::world::World;

pub mod ai;
pub mod mob;

pub mod item;
pub mod living;
pub mod player;

/// Represents a not living Entity (e.g. Item, Egg, Snowball...)
pub struct Entity {
    /// A unique identifier for the entity
    pub entity_id: EntityId,
    /// A persistant, unique identifier for the entity
    pub entity_uuid: uuid::Uuid,
    pub uuid: Uuid,
    /// The type of entity (e.g., player, zombie, item)
    pub entity_type: EntityType,
    /// The world in which the entity exists.
    pub world: Arc<World>,
    /// The entity's current position in the world
    pub pos: AtomicCell<Vector3<f64>>,
    /// The entity's position rounded to the nearest block coordinates
    pub block_pos: AtomicCell<WorldPosition>,
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
    /// Indicates whether the entity is inside of ground, and needs to be pushed out.
    pub in_ground: AtomicBool,
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
        uuid: Uuid,
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
            uuid,
            entity_type,
            in_ground: AtomicBool::new(false),
            on_ground: AtomicBool::new(false),
            pos: AtomicCell::new(position),
            block_pos: AtomicCell::new(WorldPosition(Vector3::new(floor_x, floor_y, floor_z))),
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
                self.block_pos.store(WorldPosition(new_block_pos));

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

    pub async fn advance_position(&self) {
        let velocity = self.velocity.load();
        let mut pos = self.pos.load().add(&velocity);
        dbg!(pos.y);
        self.collision_check(true).await;
        let collision_y_pos = self.pos.load().y;
        if (pos.y.ceil() - collision_y_pos.ceil()) > 1. {
            pos.y = collision_y_pos;
        }
        self.set_pos(pos.x, pos.y, pos.z);
    }

    async fn collision_check(&self, snap: bool) {
        // TODO: Collision check with other entities.

        let pos = self.pos.load();
        let future_positions = self.add_velocity_block_by_block();

        let mut chunks = vec![];
        for future_position in &future_positions {
            // TODO Change rounding based on velocity direction
            let x_section = get_section_cord(future_position.x.round() as i32);
            let z_section = get_section_cord(future_position.z.round() as i32);
            let chunk_pos = Vector2::new(x_section, z_section);
            if !chunks.contains(&chunk_pos) {
                chunks.push(chunk_pos);
            }
        }

        if chunks.is_empty() {
            return;
        }
        let (_, mut chunks) = self.world.receive_chunks(&chunks);

        let mut passed_y_value = pos.y;
        while let Some(chunk) = chunks.recv().await {
            let chunk = chunk.read().await;
            for future_position in &future_positions {
                let (section_x, section_z) = (
                    get_section_cord(future_position.x.round() as i32),
                    get_section_cord(future_position.z.round() as i32),
                );
                // TODO: Add check for other blocks that affect collision, like water
                if chunk.position.x != section_x || chunk.position.z != section_z {
                    continue;
                }

                let block_id =
                    chunk
                        .blocks
                        .get_block(ChunkRelativeBlockCoordinates::from(Vector3 {
                            x: future_position.x.round(),
                            z: future_position.z.round(),
                            y: future_position.y.floor(),
                        }));
                // Air check

                if block_id == 0 {
                    passed_y_value = passed_y_value.min(future_position.y);
                    self.on_ground.store(false, Ordering::Relaxed);
                } else if pos.y > future_position.y || !self.on_ground.load(Ordering::Relaxed) {
                    passed_y_value = dbg!(passed_y_value.floor());
                    let mut new_pos = pos;
                    new_pos.y = pos.y.floor();
                    self.on_ground.store(true, Ordering::Relaxed);
                    if snap {
                        dbg!(passed_y_value);
                        self.set_pos(new_pos.x, passed_y_value, new_pos.z);
                    }
                }
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn add_velocity_block_by_block(&self) -> Vec<Vector3<f64>> {
        let velocity = self.velocity.load();
        let pos = self.pos.load();
        let blocks = |velocity: f64, out: &mut Vec<f64>, pos: f64| {
            if velocity > 0. {
                if velocity > 1. {
                    for i in (1..=(velocity.ceil() as i32)).rev() {
                        out.push(f64::from(i));
                    }
                } else {
                    out.push(1.);
                }
            } else if velocity < 0. {
                if velocity < -1. {
                    for i in ((velocity.floor() as i32)..0).rev() {
                        out.push(f64::from(i));
                    }
                } else {
                    out.push(-1.);
                }
            }
            out.iter_mut().for_each(|velocity| *velocity += pos.round());
        };

        let mut x = vec![];
        let mut y = vec![];
        let mut z = vec![];
        blocks(velocity.x, &mut x, pos.x);
        blocks(velocity.y, &mut y, pos.y);
        blocks(velocity.z, &mut z, pos.z);

        let fix_length = |length: usize, other: &mut Vec<f64>| {
            if other.len() < length {
                let last = other.last().unwrap_or(&0.);
                let first = other.first().unwrap_or(&0.);
                let increment = (last - first) / length as f64;
                *other = (0..length)
                    .map(|i| first + increment * i as f64)
                    .collect_vec();
            }
        };
        let (x_len, y_len, z_len) = (x.len(), y.len(), z.len());
        if x_len >= y_len && x_len >= z_len {
            fix_length(x_len, &mut y);
            fix_length(x_len, &mut z);
        } else if y_len >= x_len && y_len >= z_len {
            fix_length(y_len, &mut x);
            fix_length(y_len, &mut z);
        } else if z_len >= x_len && z_len >= y_len {
            fix_length(z_len, &mut x);
            fix_length(z_len, &mut y);
        }

        x.into_iter()
            .zip(y)
            .zip(z)
            .map(|((x, y), z)| Vector3 { x, y, z })
            .collect_vec()
    }

    pub async fn send_position(&self, old_position: Vector3<f64>, server: &Arc<Server>) {
        let pos = self.pos.load();
        let (dx, dy, dz) = (
            pos.x.mul_add(4096., -(old_position.x * 4096.)),
            pos.y.mul_add(4096., -(old_position.y * 4096.)),
            pos.z.mul_add(4096., -(old_position.z * 4096.)),
        );
        server
            .broadcast_packet_all(&CUpdateEntityPos::new(
                self.entity_id.into(),
                dx as i16,
                dy as i16,
                dz as i16,
                self.on_ground.load(Ordering::Relaxed),
            ))
            .await;
    }

    pub async fn send_velocity(&self, server: &Arc<Server>) {
        self.velocity.load();
        let entity_id = self.entity_id.into();
        let packet = CEntityVelocity::new(&entity_id, self.velocity.load());
        server.broadcast_packet_all(&packet).await;
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

    pub async fn set_pose(&self, pose: EntityPose) {
        self.pose.store(pose);
        let pose = pose as i32;
        let packet = CSetEntityMetadata::<VarInt>::new(
            self.entity_id.into(),
            Metadata::new(6, 21.into(), pose.into()),
        );
        self.world.broadcast_packet_all(&packet).await;
    }

    // This gets run once per "tick" (tokio task sleeping to imitate tick)
    pub fn apply_gravity(&self) {
        let mut velocity = self.velocity.load();
        velocity.y -= self.entity_type.gravity();
        self.velocity.store(velocity);
    }

    pub fn get_spawn_entity_packet(&self, data: Option<i32>) -> CSpawnEntity {
        let pos = self.pos.load();
        let velocity = self.velocity.load();
        CSpawnEntity::new(
            self.entity_id.into(),
            self.uuid,
            (self.entity_type as i32).into(),
            pos.x,
            pos.y,
            pos.z,
            self.pitch.load(),
            self.yaw.load(),
            self.head_yaw.load(),
            data.unwrap_or(0).into(),
            velocity.x,
            velocity.y,
            velocity.z,
        )
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

#[must_use]
pub fn random_float() -> f64 {
    rand::thread_rng().gen_range(0.0..=1.0)
}
