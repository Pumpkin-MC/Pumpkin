use std::{
    sync::atomic::Ordering,
    sync::Arc,
};

use pumpkin_protocol::java::client::play::CEntityVelocity;
use log::{debug, info};

use super::{Entity, EntityBase, NBTStorage, living::LivingEntity};
use crate::server::Server;
use pumpkin_data::BlockDirection;
use pumpkin_util::math::{vector3::Vector3, position::BlockPos};
use pumpkin_util::math::boundingbox::BoundingBox;
use crate::world::World;
use pumpkin_util::math::vector3::Axis;
pub mod firework_rocket;
pub mod wind_charge;
pub mod egg;
pub mod snowball;

pub struct ThrownItemEntity {
    pub entity: Entity,
    pub owner_id: Option<i32>,
    pub collides_with_projectiles: bool,
}

impl ThrownItemEntity {
    pub fn new(entity: Entity, owner: &Entity) -> Self {
        let mut owner_pos = owner.pos.load();
        owner_pos.y = (owner_pos.y + f64::from(owner.entity_dimension.load().eye_height)) - 0.1;
        entity.pos.store(owner_pos);
        Self { entity, owner_id: Some(owner.entity_id), collides_with_projectiles: false }
    }

    pub fn set_velocity_from(
        &self,
        _shooter: &Entity,
        pitch: f32,
        yaw: f32,
        roll: f32,
        speed: f32,
        divergence: f32,
    ) {
        let yaw_rad = yaw.to_radians();
        let pitch_rad = pitch.to_radians();
        let roll_rad = (pitch + roll).to_radians();

        let x = -yaw_rad.sin() * pitch_rad.cos();
        let y = -roll_rad.sin();
        let z = yaw_rad.cos() * pitch_rad.cos();
        
        self.set_velocity(
            f64::from(x),
            f64::from(y),
            f64::from(z),
            f64::from(speed),
            f64::from(divergence),
        );
    }

    pub fn set_velocity(&self, x: f64, y: f64, z: f64, power: f64, uncertainty: f64) {
        fn next_triangular(mode: f64, deviation: f64) -> f64 {
            mode + deviation * (rand::random::<f64>() - rand::random::<f64>())
        }
        let velocity = Vector3::new(x, y, z)
            .normalize()
            .add_raw(
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
            )
            .multiply(power, power, power);
        
        self.entity.velocity.store(velocity);
        let len = velocity.horizontal_length();
        self.entity.set_rotation(
            velocity.x.atan2(velocity.z) as f32 * 57.295_776,
            velocity.y.atan2(len) as f32 * 57.295_776,
        );
    }
}

impl NBTStorage for ThrownItemEntity {}

impl ThrownItemEntity {
    pub async fn process_tick(&self, caller: Arc<dyn EntityBase>, _server: &Server) {
        let entity = self.get_entity();
        let world = entity.world.load();

        // Apply gravity and inertia
        let mut velocity = entity.velocity.load();
        velocity.y -= self.get_gravity();
        
        let inertia = if entity.touching_water.load(Ordering::Relaxed) { 0.8 } else { 0.99 };
        velocity = velocity.multiply(inertia, inertia, inertia);

        // Store velocity
        entity.velocity.store(velocity);
        
        let start_pos = entity.pos.load();
        let delta = velocity;
        
        // Update position
        let new_pos = start_pos.add(&delta);
        entity.set_pos(new_pos);
        
        // Send updated velocity to clients
        let packet = CEntityVelocity::new(entity.entity_id.into(), velocity);
        let _ = world.broadcast_packet_all(&packet).await;

        // Calculate search box for collisions
        let search_box = BoundingBox::new(
            Vector3::new(start_pos.x.min(new_pos.x), start_pos.y.min(new_pos.y), start_pos.z.min(new_pos.z)),
            Vector3::new(start_pos.x.max(new_pos.x), start_pos.y.max(new_pos.y), start_pos.z.max(new_pos.z))
        ).expand(0.3, 0.3, 0.3);

        let mut closest_t = 1.0f64;
        let mut hit = None;

        // Block collisions
        let (block_cols, block_positions) = world.get_block_collisions(search_box).await;
        for (idx, bb) in block_cols.iter().enumerate() {
            if let Some((t)) = calculate_ray_intersection(&start_pos, &delta, bb) {
                if t < closest_t {
                    closest_t = t;
                    // Map back to block pos
                    let mut curr = 0;
                    for (len, pos) in &block_positions {
                        curr += len;
                        if idx < curr {
                            let hit_pos = start_pos.add(&delta.multiply(t, t, t));
                            hit = Some(ProjectileHit::Block {
                                pos: *pos,
                                face: get_hit_face(hit_pos, *pos),
                                hit_pos,
                                normal: delta.normalize().multiply(-1.0, -1.0, -1.0),
                            });
                            break;
                        }
                    }
                }
            }
        }

        // Entity collisions
        let candidates = world.get_entities_at_box(&search_box).await;
        for cand in candidates {
            if self.should_skip_collision(entity, &cand) {
                continue;
            }

            let ebb = cand.get_entity().bounding_box.load().expand(0.3, 0.3, 0.3);
            if let Some(t) = calculate_ray_intersection(&start_pos, &delta, &ebb) {
                if t < closest_t {
                    closest_t = t;
                    let hit_pos = start_pos.add(&delta.multiply(t, t, t));
                    hit = Some(ProjectileHit::Entity {
                        entity: cand.clone(),
                        hit_pos,
                        normal: delta.normalize().multiply(-1.0, -1.0, -1.0),
                    });
                }
            }
        }

        // Handle hit or continue
        if let Some(h) = hit {
            // Just trigger hit effects and remove
            caller.on_hit(h).await;
            entity.remove().await;
        }
    }

    fn should_skip_collision(&self, self_ent: &Entity, other: &Arc<dyn EntityBase>) -> bool {
        let other_ent = other.get_entity();
        if other_ent.entity_id == self_ent.entity_id { return true; }
        
        // Skip owner for initial frames
        if Some(other_ent.entity_id) == self.owner_id && self_ent.age.load(Ordering::Relaxed) < 5 {
            return true;
        }

        // Projectile vs projectile logic
        if !self.collides_with_projectiles && other.is_projectile() {
            return true;
        }

        false
    }

    fn get_entity(&self) -> &Entity { &self.entity }
    fn get_living_entity(&self) -> Option<&LivingEntity> { None }
    fn as_nbt_storage(&self) -> &dyn NBTStorage { self }
    fn get_gravity(&self) -> f64 { 0.03 }
}

/// Ray intersection algorithm for AABBs, returning a t value
fn calculate_ray_intersection(
    start: &Vector3<f64>, 
    dir: &Vector3<f64>, 
    bb: &BoundingBox
) -> Option<f64> {
    let mut t_min = 0.0f64;
    let mut t_max = 1.0f64;

    let b_min = [bb.min.x, bb.min.y, bb.min.z];
    let b_max = [bb.max.x, bb.max.y, bb.max.z];
    let s = [start.x, start.y, start.z];
    let d = [dir.x, dir.y, dir.z];

    for i in 0..3 {
        if d[i].abs() < 1e-9 {
            if s[i] < b_min[i] || s[i] > b_max[i] { 
                return None; 
            }
        } else {
            let t1 = (b_min[i] - s[i]) / d[i];
            let t2 = (b_max[i] - s[i]) / d[i];
            t_min = t_min.max(t1.min(t2));
            t_max = t_max.min(t1.max(t2));
        }
    }
    
    if t_max >= t_min && t_min >= 0.0 && t_min <= 1.0 { 
        Some(t_min) 
    } else { 
        None 
    }
}

fn get_hit_face(hit_pos: Vector3<f64>, block_pos: BlockPos) -> BlockDirection {
    let local = hit_pos.sub(&block_pos.0.to_f64());
    let eps = 1.0e-4;
    
    if local.x <= eps { BlockDirection::West }
    else if local.x >= 1.0 - eps { BlockDirection::East }
    else if local.y <= eps { BlockDirection::Down }
    else if local.y >= 1.0 - eps { BlockDirection::Up }
    else if local.z <= eps { BlockDirection::North }
    else { BlockDirection::South }
}

pub enum ProjectileHit {
    Block {
        pos: BlockPos,
        face: BlockDirection,
        hit_pos: Vector3<f64>,
        normal: Vector3<f64>,
    },
    Entity {
        entity: Arc<dyn EntityBase>,
        hit_pos: Vector3<f64>,
        normal: Vector3<f64>,
    },
}

impl ProjectileHit {
    /// Returns the exact impact coordinates regardless of what was hit.
    pub fn hit_pos(&self) -> Vector3<f64> {
        match self {
            Self::Block { hit_pos, .. } | Self::Entity { hit_pos, .. } => *hit_pos,
        }
    }

    /// Returns the surface normal of the impact.
    pub fn normal(&self) -> Vector3<f64> {
        match self {
            Self::Block { normal, .. } | Self::Entity { normal, .. } => *normal,
        }
    }

    /// Safely returns the face hit if it was a block, otherwise None.
    pub fn face(&self) -> Option<BlockDirection> {
        match self {
            Self::Block { face, .. } => Some(*face),
            Self::Entity { .. } => None,
        }
    }
}