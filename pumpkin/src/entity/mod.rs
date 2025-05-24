use crate::{server::Server, world::portal::PortalManager};
use async_trait::async_trait;
use bytes::BufMut;
use core::f32;
use crossbeam::atomic::AtomicCell;
use living::LivingEntity;
use player::Player;
use pumpkin_data::{
    Block,
    BlockDirection,
    block_properties::{BlockProperties, CampfireLikeProperties, COLLISION_SHAPES, Facing, HorizontalFacing},
    damage::DamageType,
    entity::{EntityPose, EntityType},
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::{
    client::play::{
        CEntityPositionSync, CEntityVelocity, CHeadRot, CSetEntityMetadata, CSpawnEntity,
        CUpdateEntityPos, CUpdateEntityPosRot, CUpdateEntityRot, MetaDataType, Metadata,
    },
    codec::var_int::VarInt,
    ser::serializer::Serializer,
};
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    get_section_cord,
    position::BlockPos,
    vector2::Vector2,
    vector3::Axis,
    vector3::Vector3,
    wrap_degrees,
};
use pumpkin_world::item::ItemStack;
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
    
    /*
    There are two types of movement
    One from velocity, the other from one-time moves,
    like pushing out of blocks or knockback.
    The latter uses entity.tick_move() (adjust for collisions) 
    or entity.move_pos()
    */
    async fn handle_physics(
        &self,
        gravity: f64,
        server: &Server,
    ) {
        let entity = self.get_entity();
        let living = self.get_living_entity();
        
        if entity.pos.load().y < -100.0 {
            if let Some(live) = living {
                live.kill().await; 
            } else {
                entity.remove().await;
            }
            entity.velocity.store(Vector3::default());
            return;
        }
        
        let world = entity.world.read().await;
        
        // Apply gravity
        let mut velo = entity.velocity.load();
        velo.y -= gravity;
        entity.velocity.store(velo);
        
        // in_water will be set by fluid collision
        entity.in_water.store(false, Ordering::Relaxed);
        let (in_wall, suffocating) = Entity::check_block_collision(entity, server).await;
        // TODO in block collisions:
        /*
        lava:
            cut living.fall_distance in half
            velo.y *= 0.8
            velo *= 0.5
        water:
            set living.fall_distance to 0
            velo *= 0.8
            if buoyant: velo.y += 0.2
        
        cobweb:
            velo.x = velo.x * 0.25;
            velo.z = velo.z * 0.25;
            
            velo.y = velo.y * 0.05;
        */
        
        
        let mut velo = entity.velocity.load();
        
        if suffocating {
            if let Some(live) = living {
	        live.damage(1.0, DamageType::IN_WALL).await;
            }
        }
        
        if in_wall {
	    // TODO: Push out of blocks
	    
	    /*
	    // Note: 0.1m/tick speed to get out of blocks = 2m/s
	    if let Some(move_out) = entity.push_out_of_blocks(&world).await? {
		entity.move_pos(move_out);
	    }
	    */
	    
	    // Apply suffocating friction
	    velo = velo * 0.8;
        }
        
        // Block underneath
        let (block, state) = world.get_block_and_block_state(&entity.block_pos.load()).await;
        
        if let Some(live) = living {
            if block == Block::CAMPFIRE || block == Block::SOUL_CAMPFIRE {
                if CampfireLikeProperties::from_state_id(state.id, &block).r#signal_fire {
                    let _ = live.damage(1.0, DamageType::CAMPFIRE).await;
                }
            }
            
            if block == Block::MAGMA_BLOCK {
                let _ = live.damage(1.0, DamageType::HOT_FLOOR).await;
            }
        }
        
        let mut friction = 0.98;
        if entity.on_ground.load(Relaxed) {
                friction = 0.91 * block.slipperiness as f64;
        }
        velo.x = velo.x * friction;
        velo.z = velo.z * friction;
        
        // TODO: entity.tick_move(liquid flow)
        
        //velo = velo.multiply(entity.velocity_multiplier());
        
        entity.velocity.store(velo);
    }
    
    // Move and send
    async fn tick_move(&self, mut motion: Vector3<f64>) {
        let entity = self.get_entity();
        let living = self.get_living_entity();
        
        for axis in Axis::all() {
        	if motion.get_axis(axis).abs() < 0.03 {
        		motion.set_axis(axis, 0.0);
        	}
        }
        
        let final_move = entity.adjust_movement_for_collisions(motion).await;
        
        let on_ground = final_move.y == 0.0;
        entity.on_ground.store(on_ground, Relaxed);
        
        entity.move_pos(final_move);
        entity.velocity.store(final_move);
        
        if let Some(live) = living {
            live.update_fall_distance(
                final_move.y,
                entity.on_ground.load(Relaxed),
                false,
            ).await;
        }
        
        //entity.look_at(entity.velocity.load()).await;
        entity.send_pos_rot().await;
        entity.send_velocity().await;
        
        //entity.debug_loc().await;
        
    }
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
    /// The last position of the entity known to the client
    pub last_pos: AtomicCell<Vector3<f64>>,
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
    /// Indicates whether the entity is in water
    pub in_water: AtomicBool,
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
            in_water: AtomicBool::new(false),
            pos: AtomicCell::new(position),
            last_pos: AtomicCell::new(position),
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
            fire_ticks: AtomicI32::new(-1),
            has_visual_fire: AtomicBool::new(false),
            portal_cooldown: AtomicU32::new(0),
            portal_manager: Mutex::new(None),
        }
    }

    pub async fn set_velocity(&self, velocity: Vector3<f64>) {
        self.velocity.store(velocity);
        self.send_velocity().await;
    }
    
    pub async fn send_velocity(&self) {
        let velocity = self.velocity.load();
        self.world
            .read()
            .await
            .broadcast_packet_all(&CEntityVelocity::new(self.entity_id.into(), velocity))
            .await;
    }
    
    pub fn move_pos(&self, delta: Vector3<f64>) {
        self.set_pos(self.pos.load() + delta);
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

        self.send_rot().await;
    }
    
    pub async fn send_rot(&self) {
        let yaw = self.yaw.load();
        let pitch = self.pitch.load();
        // Broadcast the update packet.
        // TODO: Do caching to only send the packet when needed.
        let yaw = (yaw * 256.0 / 360.0).rem_euclid(256.0) as u8;
        let pitch = (pitch * 256.0 / 360.0).rem_euclid(256.0);
        self.world
            .read()
            .await
            .broadcast_packet_all(&CUpdateEntityRot::new(
                self.entity_id.into(),
                yaw,
                pitch as u8,
                self.on_ground.load(Relaxed),
            ))
            .await;
        self.send_head_rot(yaw).await;
    }
    
    // Returns last "last_pos"
    pub fn update_last_pos(&self) -> Vector3<f64> {
        let pos = self.pos.load();
        let old = self.last_pos.load();
        self.last_pos.store(pos);
        old
    }
    
    pub async fn send_pos(&self) {
        let old = self.update_last_pos();
        let new = self.pos.load();
        
        let converted = Vector3::new(
            new.x.mul_add(4096.0, - (old.x * 4096.0)) as i16,
            new.y.mul_add(4096.0, - (old.y * 4096.0)) as i16,
            new.z.mul_add(4096.0, - (old.z * 4096.0)) as i16
        );
            
        self.world
            .read()
            .await
            .broadcast_packet_all(&CUpdateEntityPos::new(
                self.entity_id.into(),
                Vector3::new(
                    converted.x as i16,
                    converted.y as i16,
                    converted.z as i16
                ),
                self.on_ground.load(Relaxed),
            ))
            .await;
    }
    
    pub async fn send_pos_rot(&self) {
        let old = self.update_last_pos();
        let new = self.pos.load();
    
        let converted = Vector3::new(
            new.x.mul_add(4096.0, - (old.x * 4096.0)) as i16,
            new.y.mul_add(4096.0, - (old.y * 4096.0)) as i16,
            new.z.mul_add(4096.0, - (old.z * 4096.0)) as i16
        );
        
        let yaw = self.yaw.load();
        let pitch = self.pitch.load();
        // Broadcast the update packet.
        // TODO: Do caching to only send the packet when needed.
        let yaw = (yaw * 256.0 / 360.0).rem_euclid(256.0) as u8;
        let pitch = (pitch * 256.0 / 360.0).rem_euclid(256.0);
        
        self.world
            .read()
            .await
            .broadcast_packet_all(&CUpdateEntityPosRot::new(
                self.entity_id.into(),
                Vector3::new(
                    converted.x as i16,
                    converted.y as i16,
                    converted.z as i16
                ),
                yaw,
                pitch as u8,
                self.on_ground.load(Relaxed),
            ))
            .await;
            
        self.send_head_rot(yaw).await;
    }
    
    pub async fn send_head_rot(&self, head_yaw: u8) {
        self.world
            .read()
            .await
            .broadcast_packet_all(&CHeadRot::new(self.entity_id.into(), head_yaw))
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
                caller
                    .clone()
                    .teleport(None, None, None, portal_manager.portal_world.clone())
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
    
    pub fn get_drops(&self) -> Option<Vec<ItemStack>> {
        // TODO
        // self.entity_type.get_loot()
        None
    }

    /// Removes the `Entity` from their current `World`
    pub async fn remove(&self) {
        
        let world = self.world.read().await;
        
        if let Some(loot) = self.get_drops() {
            for stack in loot {
                world.drop_stack(&self.block_pos.load(), stack).await;
            }
        }
        
        self.world.read().await.remove_entity(self).await;
        // TODO: Drops
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
    
    /*
    pub async fn set_fire(&self, fire: bool) {
        if self.on_fire.load(Relaxed) == fire {
            return;
        }
        self.on_fire.store(fire, Relaxed);
        self.set_flag(Flag::OnFire, fire).await;
    }
    */

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

    // Returns whether entity is in a solid block, and whether it's suffocating
    pub async fn check_block_collision(entity_base: &dyn EntityBase, server: &Server) -> (bool, bool) {
        let entity = entity_base.get_entity();
        let bounding_box = entity.bounding_box.load();
        
        let mut eye_level_box = bounding_box.clone();
        let eye_height = entity.standing_eye_height as f64;
        eye_level_box.min.y = eye_height;
        eye_level_box.max.y = eye_height;
        
        let mut in_wall = false;
        let mut suffocating = false;
        
        let aabb = bounding_box.expand(-0.001, -0.001, -0.001);
        let blockpos = aabb.min_block_pos();
        let blockpos1 = aabb.max_block_pos();
        let world = entity.world.read().await;

        for x in blockpos.0.x..=blockpos1.0.x {
            for y in blockpos.0.y..=blockpos1.0.y {
                for z in blockpos.0.z..=blockpos1.0.z {
                    let pos = BlockPos::new(x, y, z);
                    let (block, state) = world.get_block_and_block_state(&pos).await;
                    
                    let mut collided = false;
                    if state.is_full_cube() {
                        collided = true;
                        
                        if !suffocating && state.is_solid() {
                            let collision_shape = COLLISION_SHAPES[state.collision_shapes[0] as usize]
                                .to_box()
                                .add_pos(pos);
                            suffocating = collision_shape.intersects(&eye_level_box);
                        }
                        
                    } else if !state.is_air() && !state.collision_shapes.is_empty() {
                        'shapes: for shape in state.collision_shapes {
                            let collision_shape = COLLISION_SHAPES[*shape as usize]
                                .to_box()
                                .add_pos(pos);
                            if collision_shape.intersects(&bounding_box) {
                                collided = true;
                                
                                if !suffocating && state.is_solid() {
                                    suffocating = collision_shape.intersects(&eye_level_box);
                                }
                            }
                        }
                    }
                    
                    if collided {
                        if state.is_solid() {
                            in_wall = true;
                        }
                        world
                            .block_registry
                            .on_entity_collision(block, &world, entity_base, pos, state, server)
                            .await;
                    }
                    
                    if let Ok(fluid) = world.get_fluid(&pos).await {
                        // TODO: Check fluid level
                        world
                            .block_registry
                            .on_entity_collision_fluid(&fluid, entity_base)
                            .await;
                    }
                }
            }
        }
        (in_wall, suffocating)
    }

    async fn teleport(
        &self,
        position: Option<Vector3<f64>>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        _world: Arc<World>,
    ) {
        dbg!("aa");
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
    
    pub async fn debug_loc(&self) {
        self
            .world
            .read()
            .await
            .spawn_particle(
                self.pos.load(),
                Vector3::default(),
                0.2,
                7,
                pumpkin_data::particle::Particle::DustPlume,
            )
            .await;
    }

    async fn adjust_movement_for_collisions(
        &self, 
        movement: Vector3<f64>
    ) -> Vector3<f64> {
        
        if movement.length_squared() == 0.0 {
            return movement;
        }
        
        let bounding_box = self.bounding_box.load();
        
        let (collisions, _) = self
            .world.read().await
            .get_block_collisions(bounding_box.stretch(movement)).await;
        if collisions.len() == 0 {
            return movement;
        };
        
        let mut adjusted_movement = movement;
        for axis in Axis::all() {
            if movement.get_axis(axis) == 0.0 {
                continue;
            }
            
            let mut max_time = 1.0;
            for inert_box in &collisions {
                if let Some(collision_time) = bounding_box.calculate_collision_time(
                    inert_box,
                    adjusted_movement,
                    axis,
                    max_time
                ) {
                    
                    max_time = collision_time;
                }
            }
            
            if max_time < 1.0 {
                let changed_component = adjusted_movement.get_axis(axis) * max_time;
                adjusted_movement.set_axis(axis, changed_component);
            }
        }
        
        adjusted_movement
    }

    // Faulty on testing. Idk why
    // Only sometimes works
    /*
    async fn push_out_of_blocks(&self, world: &World) -> Result<Option<Vector3<f64>>, GetBlockError> {
        let block_pos = self.block_pos.load();
        
        let mut movement = None;
        
        // Block pos can be in air, yet I collide with blocks;
        // in that case, move to center of block
        let current_block = world.get_block_state_result(&block_pos).await?;
        if !current_block.is_full_cube() {
            let block_pos_into = block_pos.0.to_f64();
            let center = Vector3::new(
                block_pos_into.x + 0.5,
                block_pos_into.y,
                block_pos_into.z + 0.5
            );
            let delta = center.sub(&self.pos.load());
            //let scaled = delta.normalize() * 0.1;
            //movement = Some(scaled);
            movement = Some(delta);
            
            if current_block.is_air() {
                return Ok(movement);
            }
        }
        
        for direction in BlockDirection::horizontal() {
            let offset = direction.to_offset();
            let offset_pos = block_pos.offset(offset);
            
            let block = world.get_block_state_result(&offset_pos).await?;
                
                
            if let Some(_) = movement {
                if block.is_air() {
                    movement = Some(offset.to_f64() * 0.5);
                    break;
                }
            } else {
                if !block.is_full_cube() {
                    movement = Some(offset.to_f64() * 0.5);
                }
            }
        }
        Ok(movement)
    }
    */

    /// Applies knockback to the entity, following vanilla Minecraft's mechanics.
    ///
    /// This function calculates the entity's new velocity based on the specified knockback strength and direction.
    pub fn calculate_knockback(&self, strength: f64, x: f64, z: f64) -> Vector3<f64> {
        // This has some vanilla magic
        let mut x = x;
        let mut z = z;
        while x.mul_add(x, z * z) < 1.0E-5 {
            x = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
            z = (rand::random::<f64>() - rand::random::<f64>()) * 0.01;
        }

        let var8 = Vector3::new(x, 0.0, z).normalize() * strength;
        let velocity = self.velocity.load();
        return Vector3::new(
            velocity.x / 2.0 - var8.x,
            if self.on_ground.load(Relaxed) {
                (velocity.y / 2.0 + strength).min(0.4)
            } else {
                velocity.y
            },
            velocity.z / 2.0 - var8.z,
        );
    }
    
    pub fn jump(&self, strength: f64) {
        if self.on_ground.load(Relaxed) {
            let mut velocity = self.velocity.load();
            velocity.y += strength;
            self.velocity.store(velocity);
        }
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
        nbt.put_short("Fire", self.fire_ticks.load(Relaxed) as i16);
        nbt.put_bool("OnGround", self.on_ground.load(Relaxed));
        nbt.put_bool("Invulnerable", self.invulnerable.load(Relaxed));
        nbt.put_int("PortalCooldown", self.portal_cooldown.load(Relaxed) as i32);
        if self.has_visual_fire.load(Relaxed) {
            nbt.put_bool("HasVisualFire", true);
        }

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
    async fn write_nbt(&self, nbt: &mut NbtCompound);

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
