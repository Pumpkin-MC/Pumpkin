use crate::{server::Server, world::portal::PortalManager};
use async_trait::async_trait;
use bytes::BufMut;
use core::f32;
use crossbeam::atomic::AtomicCell;
use living::LivingEntity;
use player::Player;
use pumpkin_data::block_properties::Integer0To15;
use pumpkin_data::{
    block_properties::{Facing, HorizontalFacing},
    damage::DamageType,
    entity::{EntityPose, EntityType},
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::{
    codec::var_int::VarInt,
    java::client::play::{
        CEntityPositionSync, CEntityVelocity, CHeadRot, CSetEntityMetadata, CSpawnEntity,
        CUpdateEntityRot, MetaDataType, Metadata,
    },
    ser::serializer::Serializer,
};
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    get_section_cord,
    position::BlockPos,
    vector2::Vector2,
    vector3::Vector3,
    wrap_degrees,
};
use serde::Serialize;
use std::sync::{
    Arc,
    atomic::{
        AtomicBool, AtomicI32, AtomicU32,
        Ordering::{self, Relaxed},
    },
};
use tokio::sync::{Mutex, RwLock};

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
pub mod r#type;

mod combat;

pub type EntityId = i32;

#[async_trait]
pub trait EntityBase: Send + Sync {
    /// Called every tick for this entity.
    ///
    /// The `caller` parameter is a reference to the entity that initiated the tick.
    /// This can be the same entity the method is being called on (`self`),
    /// but in some scenarios (e.g., interactions or events), it might be a different entity.
    ///
    /// The `server` parameter provides access to the game server instance.
    async fn tick(&self, caller: Arc<dyn EntityBase>, server: &Server) {
        if let Some(living) = self.get_living_entity() {
            living.tick(caller, server).await;
        } else {
            self.get_entity().tick(caller, server).await;
        }
    }

    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        if let Some(living) = self.get_living_entity() {
            living.write_nbt(nbt).await;
        } else {
            self.get_entity().write_nbt(nbt).await;
        }
    }

    async fn read_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        if let Some(living) = self.get_living_entity() {
            living.read_nbt(nbt).await;
        } else {
            self.get_entity().read_nbt(nbt).await;
        }
    }

    async fn init_data_tracker(&self) {}

    async fn teleport(
        self: Arc<Self>,
        position: Option<Vector3<f64>>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        world: Arc<World>,
    ) {
        self.get_entity()
            .teleport(position, yaw, pitch, world)
            .await;
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
    async fn on_player_collision(&self, _player: &Arc<Player>) {}
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
    pub fire_ticks: AtomicI32,
    pub has_visual_fire: AtomicBool,

    pub first_loaded_chunk_position: AtomicCell<Option<Vector3<i32>>>,

    pub portal_cooldown: AtomicU32,

    pub portal_manager: Mutex<Option<Mutex<PortalManager>>>,
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
            entity_id: CURRENT_ID.fetch_add(1, Relaxed),
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
            first_loaded_chunk_position: AtomicCell::new(None),
            bounding_box: AtomicCell::new(BoundingBox::new_from_pos(
                position.x,
                position.y,
                position.z,
                &bounding_box_size,
            )),
            bounding_box_size: AtomicCell::new(bounding_box_size),
            invulnerable: AtomicBool::new(invulnerable),
            damage_immunities: Vec::new(),
            fire_ticks: AtomicI32::new(-1),
            has_visual_fire: AtomicBool::new(false),
            portal_cooldown: AtomicU32::new(0),
            portal_manager: Mutex::new(None),
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
                new_position.x,
                new_position.y,
                new_position.z,
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
                self.on_ground.load(Relaxed),
            ))
            .await;
        self.world
            .read()
            .await
            .broadcast_packet_all(&CHeadRot::new(self.entity_id.into(), yaw as u8))
            .await;
    }

    fn default_portal_cooldown(&self) -> u32 {
        if self.entity_type == EntityType::PLAYER {
            10
        } else {
            300
        }
    }

    async fn tick_portal(&self, caller: &Arc<dyn EntityBase>) {
        if self.portal_cooldown.load(Ordering::Relaxed) > 0 {
            self.portal_cooldown.fetch_sub(1, Ordering::Relaxed);
        }
        let mut manager_guard = self.portal_manager.lock().await;
        // I know this is ugly, but a quick fix because i can't modify the thing while using it
        let mut should_remove = false;
        if let Some(pmanager_mutex) = manager_guard.as_ref() {
            let mut portal_manager = pmanager_mutex.lock().await;
            if portal_manager.tick() {
                // reset cooldown
                self.portal_cooldown
                    .store(self.default_portal_cooldown(), Ordering::Relaxed);
                let pos = self.pos.load();
                // TODO: this is bad
                let scale_factor_new = if portal_manager.portal_world.dimension_type
                    == VanillaDimensionType::TheNether
                {
                    8.0
                } else {
                    1.0
                };
                // TODO: this is bad
                let scale_factor_current =
                    if self.world.read().await.dimension_type == VanillaDimensionType::TheNether {
                        8.0
                    } else {
                        1.0
                    };
                let scale_factor = scale_factor_current / scale_factor_new;
                // TODO
                let pos = BlockPos::floored(pos.x * scale_factor, pos.y, pos.z * scale_factor);
                caller
                    .clone()
                    .teleport(
                        Some(pos.0.to_f64()),
                        None,
                        None,
                        portal_manager.portal_world.clone(),
                    )
                    .await;
            } else if portal_manager.ticks_in_portal == 0 {
                should_remove = true;
            }
        }
        if should_remove {
            *manager_guard = None;
        }
    }

    pub async fn try_use_portal(&self, portal_delay: u32, portal_world: Arc<World>, pos: BlockPos) {
        if self.portal_cooldown.load(Ordering::Relaxed) > 0 {
            self.portal_cooldown
                .store(self.default_portal_cooldown(), Ordering::Relaxed);
            return;
        }
        let mut manager = self.portal_manager.lock().await;
        if manager.is_none() {
            *manager = Some(Mutex::new(PortalManager::new(
                portal_delay,
                portal_world,
                pos,
            )));
        } else if let Some(manager) = manager.as_ref() {
            let mut manager = manager.lock().await;
            manager.pos = pos;
            manager.in_portal = true;
        }
    }

    /// Extinguishes this entity.
    pub fn extinguish(&self) {
        self.fire_ticks.store(0, Ordering::Relaxed);
    }

    pub fn set_on_fire_for(&self, seconds: f32) {
        self.set_on_fire_for_ticks((seconds * 20.0).floor() as u32);
    }

    pub fn set_on_fire_for_ticks(&self, ticks: u32) {
        if self.fire_ticks.load(Ordering::Relaxed) < ticks as i32 {
            self.fire_ticks.store(ticks as i32, Ordering::Relaxed);
        }
        // TODO: defrost
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
            if self.on_ground.load(Relaxed) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - var8.z,
        ));
    }

    pub async fn set_sneaking(&self, sneaking: bool) {
        assert!(self.sneaking.load(Relaxed) != sneaking);
        self.sneaking.store(sneaking, Relaxed);
        self.set_flag(Flag::Sneaking, sneaking).await;
        if sneaking {
            self.set_pose(EntityPose::Crouching).await;
        } else {
            self.set_pose(EntityPose::Standing).await;
        }
    }

    pub async fn set_on_fire(&self, on_fire: bool) {
        if self.has_visual_fire.load(Ordering::Relaxed) != on_fire {
            self.has_visual_fire.store(on_fire, Ordering::Relaxed);
            self.set_flag(Flag::OnFire, on_fire).await;
        }
    }

    pub fn get_horizontal_facing(&self) -> HorizontalFacing {
        let adjusted_yaw = self.yaw.load().rem_euclid(360.0); // Normalize yaw to [0, 360)

        match adjusted_yaw {
            0.0..=45.0 | 315.0..=360.0 => HorizontalFacing::South,
            45.0..=135.0 => HorizontalFacing::West,
            135.0..=225.0 => HorizontalFacing::North,
            225.0..=315.0 => HorizontalFacing::East,
            _ => HorizontalFacing::South, // Default case, should not occur
        }
    }

    pub fn get_rotation_16(&self) -> Integer0To15 {
        let adjusted_yaw = self.yaw.load().rem_euclid(360.0);

        let index = (adjusted_yaw / 22.5).round() as u8 % 16;

        match index {
            0 => Integer0To15::L0,
            1 => Integer0To15::L1,
            2 => Integer0To15::L2,
            3 => Integer0To15::L3,
            4 => Integer0To15::L4,
            5 => Integer0To15::L5,
            6 => Integer0To15::L6,
            7 => Integer0To15::L7,
            8 => Integer0To15::L8,
            9 => Integer0To15::L9,
            10 => Integer0To15::L10,
            11 => Integer0To15::L11,
            12 => Integer0To15::L12,
            13 => Integer0To15::L13,
            14 => Integer0To15::L14,
            _ => Integer0To15::L15,
        }
    }

    pub fn get_facing(&self) -> Facing {
        let pitch = self.pitch.load().to_radians();
        let yaw = -self.yaw.load().to_radians();

        let (sin_p, cos_p) = pitch.sin_cos();
        let (sin_y, cos_y) = yaw.sin_cos();

        let x = sin_y * cos_p;
        let y = -sin_p;
        let z = cos_y * cos_p;

        let ax = x.abs();
        let ay = y.abs();
        let az = z.abs();

        if ax > ay && ax > az {
            if x > 0.0 { Facing::East } else { Facing::West }
        } else if ay > ax && ay > az {
            if y > 0.0 { Facing::Up } else { Facing::Down }
        } else if z > 0.0 {
            Facing::South
        } else {
            Facing::North
        }
    }

    pub fn get_entity_facing_order(&self) -> [Facing; 6] {
        let pitch = self.pitch.load().to_radians();
        let yaw = -self.yaw.load().to_radians();

        let sin_p = pitch.sin();
        let cos_p = pitch.cos();
        let sin_y = yaw.sin();
        let cos_y = yaw.cos();

        let east_west = if sin_y > 0.0 {
            Facing::East
        } else {
            Facing::West
        };
        let up_down = if sin_p < 0.0 {
            Facing::Up
        } else {
            Facing::Down
        };
        let south_north = if cos_y > 0.0 {
            Facing::South
        } else {
            Facing::North
        };

        let x_axis = sin_y.abs();
        let y_axis = sin_p.abs();
        let z_axis = cos_y.abs();
        let x_weight = x_axis * cos_p;
        let z_weight = z_axis * cos_p;

        let (first, second, third) = if x_axis > z_axis {
            if y_axis > x_weight {
                (up_down, east_west, south_north)
            } else if z_weight > y_axis {
                (east_west, south_north, up_down)
            } else {
                (east_west, up_down, south_north)
            }
        } else if y_axis > z_weight {
            (up_down, south_north, east_west)
        } else if x_weight > y_axis {
            (south_north, east_west, up_down)
        } else {
            (south_north, up_down, east_west)
        };

        [
            first,
            second,
            third,
            third.opposite(),
            second.opposite(),
            first.opposite(),
        ]
    }

    pub async fn set_sprinting(&self, sprinting: bool) {
        assert!(self.sprinting.load(Relaxed) != sprinting);
        self.sprinting.store(sprinting, Relaxed);
        self.set_flag(Flag::Sprinting, sprinting).await;
    }

    pub fn check_fall_flying(&self) -> bool {
        !self.on_ground.load(Relaxed)
    }

    pub async fn set_fall_flying(&self, fall_flying: bool) {
        assert!(self.fall_flying.load(Relaxed) != fall_flying);
        self.fall_flying.store(fall_flying, Relaxed);
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
        self.invulnerable.load(Relaxed) || self.damage_immunities.contains(damage_type)
    }

    fn velocity_multiplier(_pos: Vector3<f64>) -> f32 {
        // let world = self.world.read().await;
        // TODO: handle when player is outside world
        // let block = world.get_block(&self.block_pos.load()).await;
        // block.velocity_multiplier
        1.0
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

    pub async fn check_block_collision(entity: &dyn EntityBase, server: &Server) {
        let aabb = entity.get_entity().bounding_box.load();
        let blockpos = BlockPos::new(
            (aabb.min.x + 0.001).floor() as i32,
            (aabb.min.y + 0.001).floor() as i32,
            (aabb.min.z + 0.001).floor() as i32,
        );
        let blockpos1 = BlockPos::new(
            (aabb.max.x - 0.001).floor() as i32,
            (aabb.max.y - 0.001).floor() as i32,
            (aabb.max.z - 0.001).floor() as i32,
        );
        let world = entity.get_entity().world.read().await;

        for x in blockpos.0.x..=blockpos1.0.x {
            for y in blockpos.0.y..=blockpos1.0.y {
                for z in blockpos.0.z..=blockpos1.0.z {
                    let pos = BlockPos::new(x, y, z);
                    let (block, state) = world.get_block_and_block_state(&pos).await;
                    let block_outlines = state.get_block_outline_shapes();

                    if let Some(outlines) = block_outlines {
                        if outlines.is_empty() {
                            world
                                .block_registry
                                .on_entity_collision(block, &world, entity, &pos, state, server)
                                .await;
                            let fluid = world.get_fluid(&pos).await;
                            world
                                .block_registry
                                .on_entity_collision_fluid(&fluid, entity)
                                .await;
                            continue;
                        }
                        for outline in outlines {
                            let outline_aabb = outline.at_pos(pos);
                            if outline_aabb.intersects(&aabb) {
                                world
                                    .block_registry
                                    .on_entity_collision(block, &world, entity, &pos, state, server)
                                    .await;
                                let fluid = world.get_fluid(&pos).await;
                                world
                                    .block_registry
                                    .on_entity_collision_fluid(&fluid, entity)
                                    .await;
                                break;
                            }
                        }
                    } else {
                        world
                            .block_registry
                            .on_entity_collision(block, &world, entity, &pos, state, server)
                            .await;
                        let fluid = world.get_fluid(&pos).await;
                        world
                            .block_registry
                            .on_entity_collision_fluid(&fluid, entity)
                            .await;
                    }
                }
            }
        }
    }

    async fn teleport(
        &self,
        position: Option<Vector3<f64>>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        _world: Arc<World>,
    ) {
        // TODO: handle world change
        self.world
            .read()
            .await
            .broadcast_packet_all(&CEntityPositionSync::new(
                self.entity_id.into(),
                position.unwrap_or(Vector3::new(0.0, 0.0, 0.0)),
                Vector3::new(0.0, 0.0, 0.0),
                yaw.unwrap_or(0.0),
                pitch.unwrap_or(0.0),
                self.on_ground.load(Ordering::SeqCst),
            ))
            .await;
    }
}

#[async_trait]
impl EntityBase for Entity {
    async fn damage(&self, _amount: f32, _damage_type: DamageType) -> bool {
        false
    }

    async fn tick(&self, caller: Arc<dyn EntityBase>, _server: &Server) {
        self.tick_portal(&caller).await;
        let fire_ticks = self.fire_ticks.load(Ordering::Relaxed);
        if fire_ticks > 0 {
            if self.entity_type.fire_immune {
                self.fire_ticks.store(fire_ticks - 4, Ordering::Relaxed);
                if self.fire_ticks.load(Ordering::Relaxed) < 0 {
                    self.extinguish();
                }
            } else {
                if fire_ticks % 20 == 0 {
                    caller.damage(1.0, DamageType::ON_FIRE).await;
                }

                self.fire_ticks.store(fire_ticks - 1, Ordering::Relaxed);
            }
        }
        self.set_on_fire(self.fire_ticks.load(Ordering::Relaxed) > 0)
            .await;
        // TODO: Tick
    }

    async fn teleport(
        self: Arc<Self>,
        position: Option<Vector3<f64>>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        world: Arc<World>,
    ) {
        // TODO: handle world change
        self.teleport(position, yaw, pitch, world).await;
    }

    fn get_entity(&self) -> &Entity {
        self
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let position = self.pos.load();
        nbt.put_string(
            "id",
            format!("minecraft:{}", self.entity_type.resource_name),
        );
        let uuid = self.entity_uuid.as_u128();
        nbt.put(
            "UUID",
            NbtTag::IntArray(vec![
                (uuid >> 96) as i32,
                ((uuid >> 64) & 0xFFFF_FFFF) as i32,
                ((uuid >> 32) & 0xFFFF_FFFF) as i32,
                (uuid & 0xFFFF_FFFF) as i32,
            ]),
        );
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
        nbt.put_short("Fire", self.fire_ticks.load(Relaxed) as i16);
        nbt.put_bool("OnGround", self.on_ground.load(Relaxed));
        nbt.put_bool("Invulnerable", self.invulnerable.load(Relaxed));
        nbt.put_int("PortalCooldown", self.portal_cooldown.load(Relaxed) as i32);
        if self.has_visual_fire.load(Relaxed) {
            nbt.put_bool("HasVisualFire", true);
        }

        // todo more...
    }

    async fn read_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        let position = nbt.get_list("Pos").unwrap();
        let x = position[0].extract_double().unwrap_or(0.0);
        let y = position[1].extract_double().unwrap_or(0.0);
        let z = position[2].extract_double().unwrap_or(0.0);
        let pos = Vector3::new(x, y, z);
        self.set_pos(pos);
        self.first_loaded_chunk_position.store(Some(pos.to_i32()));
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
        self.fire_ticks
            .store(i32::from(nbt.get_short("Fire").unwrap_or(0)), Relaxed);
        self.on_ground
            .store(nbt.get_bool("OnGround").unwrap_or(false), Relaxed);
        self.invulnerable
            .store(nbt.get_bool("Invulnerable").unwrap_or(false), Relaxed);
        self.portal_cooldown
            .store(nbt.get_int("PortalCooldown").unwrap_or(0) as u32, Relaxed);
        self.has_visual_fire
            .store(nbt.get_bool("HasVisualFire").unwrap_or(false), Relaxed);
        // todo more...
    }
}

#[async_trait]
pub trait NBTStorage: Send + Sync + Sized {
    async fn write_nbt(&self, _nbt: &mut NbtCompound) {}

    async fn read_nbt(&mut self, _nbt: &mut NbtCompound) {}

    async fn read_nbt_non_mut(&self, _nbt: &mut NbtCompound) {}

    /// Creates an instance of the type from NBT data. If the NBT data is invalid or cannot be parsed, it returns `None`.
    async fn create_from_nbt(_nbt: &mut NbtCompound) -> Option<Self> {
        None
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
