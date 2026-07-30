use super::{Entity, EntityBase, NBTStorage, living::LivingEntity};
use crate::server::Server;
use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityType;
use pumpkin_protocol::java::client::play::CEntityVelocity;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use std::{
    sync::Arc,
    sync::atomic::{AtomicBool, Ordering},
};
pub mod arrow;
pub mod dragon_fireball;
pub mod egg;
pub mod ender_pearl;
pub mod eye_of_ender;
pub mod fireball;
pub mod firework_rocket;
pub mod fishing_bobber;
pub mod lingering_potion;
pub mod shulker_bullet;
pub mod small_fireball;
pub mod snowball;
pub mod splash_potion;
pub mod trident;
pub mod wind_charge;

#[must_use]
pub fn is_projectile(entity_type: &EntityType) -> bool {
    *entity_type == EntityType::ARROW
        || *entity_type == EntityType::TRIDENT
        || *entity_type == EntityType::EGG
        || *entity_type == EntityType::SNOWBALL
        || *entity_type == EntityType::FIREWORK_ROCKET
        || *entity_type == EntityType::WIND_CHARGE
        || *entity_type == EntityType::SPLASH_POTION
        || *entity_type == EntityType::LINGERING_POTION
        || *entity_type == EntityType::ENDER_PEARL
        || *entity_type == EntityType::SHULKER_BULLET
        || *entity_type == EntityType::FIREBALL
        || *entity_type == EntityType::SMALL_FIREBALL
        || *entity_type == EntityType::DRAGON_FIREBALL
        || *entity_type == EntityType::FISHING_BOBBER
}

pub struct ThrownItemEntity {
    pub entity: Entity,
    pub owner_id: Option<i32>,
    pub collides_with_projectiles: bool,
    pub has_hit: AtomicBool,
    pub gravity: f64,
}

impl ThrownItemEntity {
    pub fn new(entity: Entity, owner: &Entity, gravity: f64) -> Self {
        let mut owner_pos = owner.pos.load();
        owner_pos.y += owner.get_eye_height() - 0.1;
        entity.pos.store(owner_pos);
        Self {
            entity,
            owner_id: Some(owner.entity_id),
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity,
        }
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
            deviation.mul_add(rand::random::<f64>() - rand::random::<f64>(), mode)
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

/// Movement model for projectiles that extend `AbstractHurtingProjectile` in vanilla
/// (dragon fireball, ghast fireball, blaze small fireball, wither skull).
///
/// Instead of gravity + drag, these use an acceleration-power model:
/// `velocity = (velocity + normalize(velocity) * accelerationPower) * inertia`
pub struct HurtingProjectileEntity {
    pub entity: Entity,
    pub owner_id: Option<i32>,
    pub acceleration_power: f64,
    pub has_hit: AtomicBool,
    /// Matches Java's `Projectile.leftOwner`: becomes true once the projectile's
    /// AABB (expanded by delta movement + inflated by 1.0) no longer intersects
    /// the owner's bounding box. Until then, the projectile cannot collide with
    /// the owner.
    left_owner: AtomicBool,
}

impl HurtingProjectileEntity {
    pub const DEFAULT_ACCELERATION_POWER: f64 = 0.1;

    pub fn new(entity: Entity, owner: &Entity, acceleration_power: f64) -> Self {
        let mut owner_pos = owner.pos.load();
        owner_pos.y += owner.get_eye_height() - 0.1;
        entity.pos.store(owner_pos);
        Self {
            entity,
            owner_id: Some(owner.entity_id),
            acceleration_power,
            has_hit: AtomicBool::new(false),
            left_owner: AtomicBool::new(false),
        }
    }
}

impl NBTStorage for HurtingProjectileEntity {}

impl HurtingProjectileEntity {
    /// Process a tick using `AbstractHurtingProjectile` movement physics.
    #[expect(clippy::too_many_lines)]
    pub async fn process_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>, _server: &'a Server) {
        let entity = &self.entity;
        let world = entity.world.load();

        // Match Java's Projectile.tick() → checkLeftOwner():
        // Once the projectile's AABB (expanded by velocity, inflated by 1.0)
        // no longer intersects the owner's bounding box, set left_owner = true.
        if !self.left_owner.load(Ordering::Relaxed) {
            if let Some(owner_id) = self.owner_id {
                if let Some(owner) = world.get_entity_by_id(owner_id) {
                    let velocity = entity.velocity.load();
                    let projectile_bb = entity
                        .bounding_box
                        .load()
                        .expand_towards(velocity.x, velocity.y, velocity.z)
                        .expand(1.0, 1.0, 1.0);
                    let owner_bb = owner.get_entity().bounding_box.load();
                    if !projectile_bb.intersects(&owner_bb) {
                        self.left_owner.store(true, Ordering::Relaxed);
                    }
                } else {
                    self.left_owner.store(true, Ordering::Relaxed);
                }
            } else {
                self.left_owner.store(true, Ordering::Relaxed);
            }
        }

        entity.update_last_pos();

        let mut velocity = entity.velocity.load();

        // `AbstractHurtingProjectile.applyInertia()`
        let inertia = if entity.touching_water.load(Ordering::Relaxed) {
            0.8
        } else {
            0.95
        };
        let len = velocity.length();
        if len > 1e-9 {
            let nx = velocity.x / len;
            let ny = velocity.y / len;
            let nz = velocity.z / len;
            let ap = self.acceleration_power;
            velocity = Vector3::new(
                (velocity.x + nx * ap) * inertia,
                (velocity.y + ny * ap) * inertia,
                (velocity.z + nz * ap) * inertia,
            );
        } else {
            velocity = velocity.multiply(inertia, inertia, inertia);
        }
        entity.velocity.store(velocity);

        let start_pos = entity.pos.load();
        let delta = velocity;

        let new_pos = start_pos.add(&delta);
        entity.set_pos(new_pos);

        let packet = CEntityVelocity::new(entity.entity_id.into(), velocity);
        let chunk_pos = entity.chunk_pos.load();
        world.broadcast_to_chunk(chunk_pos, &packet);

        let search_box = BoundingBox::new(
            Vector3::new(
                start_pos.x.min(new_pos.x),
                start_pos.y.min(new_pos.y),
                start_pos.z.min(new_pos.z),
            ),
            Vector3::new(
                start_pos.x.max(new_pos.x),
                start_pos.y.max(new_pos.y),
                start_pos.z.max(new_pos.z),
            ),
        )
        .expand(0.3, 0.3, 0.3);

        let mut closest_t = 1.0f64;
        let mut hit = None;

        let (block_cols, block_positions) = world
            .get_block_collisions(search_box, caller.as_ref())
            .await;
        for (idx, bb) in block_cols.iter().enumerate() {
            if let Some(t) = calculate_ray_intersection(&start_pos, &delta, bb)
                && t < closest_t
            {
                closest_t = t;
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

        let candidates = world.get_entities_at_box(&search_box);
        for cand in candidates {
            if self.should_skip_collision(entity, &cand) {
                continue;
            }

            let ebb = cand.get_entity().bounding_box.load().expand(0.3, 0.3, 0.3);
            if let Some(t) = calculate_ray_intersection(&start_pos, &delta, &ebb)
                && t < closest_t
            {
                closest_t = t;
                let hit_pos = start_pos.add(&delta.multiply(t, t, t));
                hit = Some(ProjectileHit::Entity {
                    entity: cand.clone(),
                    hit_pos,
                    normal: delta.normalize().multiply(-1.0, -1.0, -1.0),
                });
            }
        }

        if let Some(h) = hit {
            if self.has_hit.swap(true, Ordering::SeqCst) {
                return;
            }
            caller.on_hit(h).await;
            entity.remove().await;
        }
    }

    fn should_skip_collision(&self, self_ent: &Entity, other: &Arc<dyn EntityBase>) -> bool {
        let other_ent = other.get_entity();
        if other_ent.entity_id == self_ent.entity_id {
            return true;
        }

        if let Some(owner_id) = self.owner_id {
            // Match Java's Projectile.canHitEntity: skip owner until
            // the projectile has left the owner's collision range.
            if other_ent.entity_id == owner_id && !self.left_owner.load(Ordering::Relaxed) {
                return true;
            }

            if *other_ent.entity_type == EntityType::ENDER_DRAGON {
                let oid = other_ent.entity_id;
                if oid > owner_id && oid <= owner_id + 8 {
                    return true;
                }
            }
        }

        if *other_ent.entity_type == EntityType::AREA_EFFECT_CLOUD {
            return true;
        }

        if is_projectile(other_ent.entity_type) {
            return true;
        }

        false
    }
}

impl ThrownItemEntity {
    /// Process a tick for projectile movement and collisions
    pub async fn process_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>, _server: &'a Server) {
        let entity = self.get_entity();
        let world = entity.world.load();

        entity.update_last_pos();

        // Apply gravity and inertia
        let mut velocity = entity.velocity.load();
        velocity.y -= self.get_gravity();

        let inertia = if entity.touching_water.load(Ordering::Relaxed) {
            0.8
        } else {
            0.99
        };
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
        let chunk_pos = entity.chunk_pos.load();
        world.broadcast_to_chunk(chunk_pos, &packet);

        // Calculate search box for collisions
        let search_box = BoundingBox::new(
            Vector3::new(
                start_pos.x.min(new_pos.x),
                start_pos.y.min(new_pos.y),
                start_pos.z.min(new_pos.z),
            ),
            Vector3::new(
                start_pos.x.max(new_pos.x),
                start_pos.y.max(new_pos.y),
                start_pos.z.max(new_pos.z),
            ),
        )
        .expand(0.3, 0.3, 0.3);

        let mut closest_t = 1.0f64;
        let mut hit = None;

        // Block collisions
        let (block_cols, block_positions) = world
            .get_block_collisions(search_box, caller.as_ref())
            .await;
        for (idx, bb) in block_cols.iter().enumerate() {
            if let Some(t) = calculate_ray_intersection(&start_pos, &delta, bb)
                && t < closest_t
            {
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

        // Entity collisions
        let candidates = world.get_entities_at_box(&search_box);
        for cand in candidates {
            if self.should_skip_collision(entity, &cand) {
                continue;
            }

            let ebb = cand.get_entity().bounding_box.load().expand(0.3, 0.3, 0.3);
            if let Some(t) = calculate_ray_intersection(&start_pos, &delta, &ebb)
                && t < closest_t
            {
                closest_t = t;
                let hit_pos = start_pos.add(&delta.multiply(t, t, t));
                hit = Some(ProjectileHit::Entity {
                    entity: cand.clone(),
                    hit_pos,
                    normal: delta.normalize().multiply(-1.0, -1.0, -1.0),
                });
            }
        }

        // Handle hit or continue
        if let Some(h) = hit {
            // Ensure hit is only processed once per projectile
            if self.has_hit.swap(true, Ordering::SeqCst) {
                return;
            }

            // Just trigger hit effects and remove
            caller.on_hit(h).await;
            entity.remove().await;
        }
    }

    /// Returns if collision should be skipped (e.g. owner or projectile vs projectile)
    fn should_skip_collision(&self, self_ent: &Entity, other: &Arc<dyn EntityBase>) -> bool {
        let other_ent = other.get_entity();
        if other_ent.entity_id == self_ent.entity_id {
            return true;
        }

        if let Some(owner_id) = self.owner_id {
            // Skip owner for initial frames
            if other_ent.entity_id == owner_id && self_ent.age.load(Ordering::Relaxed) < 5 {
                return true;
            }

            // Skip dragon parts belonging to the owner.  Dragon parts are
            // assigned consecutive entity IDs `owner_id + 1 ..= owner_id + 8`
            // by `Entity::reserve_ids(8)` + `from_uuid_with_id(base_id + i)`.
            if *other_ent.entity_type == EntityType::ENDER_DRAGON {
                let oid = other_ent.entity_id;
                if oid > owner_id && oid <= owner_id + 8 {
                    return true;
                }
            }
        }

        // Projectiles should pass through lingering clouds
        if *other_ent.entity_type == EntityType::AREA_EFFECT_CLOUD {
            return true;
        }

        // Projectile vs projectile logic
        if !self.collides_with_projectiles && is_projectile(other_ent.entity_type) {
            return true;
        }

        false
    }

    const fn get_entity(&self) -> &Entity {
        &self.entity
    }

    #[allow(dead_code, clippy::unused_self)]
    const fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    #[allow(dead_code, clippy::unused_self)]
    const fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }
    const fn get_gravity(&self) -> f64 {
        self.gravity
    }
}

/// Ray intersection algorithm for AABBs, returning a t value
fn calculate_ray_intersection(
    start: &Vector3<f64>,
    dir: &Vector3<f64>,
    bb: &BoundingBox,
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

    (0.0..=1.0).contains(&t_min).then_some(t_min)
}

/// Get the face of the block that was hit
fn get_hit_face(hit_pos: Vector3<f64>, block_pos: BlockPos) -> BlockDirection {
    let local = hit_pos.sub(&block_pos.0.to_f64());
    let eps = 1.0e-4;

    if local.x <= eps {
        BlockDirection::West
    } else if local.x >= 1.0 - eps {
        BlockDirection::East
    } else if local.y <= eps {
        BlockDirection::Down
    } else if local.y >= 1.0 - eps {
        BlockDirection::Up
    } else if local.z <= eps {
        BlockDirection::North
    } else {
        BlockDirection::South
    }
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
    #[must_use]
    pub const fn hit_pos(&self) -> Vector3<f64> {
        match self {
            Self::Block { hit_pos, .. } | Self::Entity { hit_pos, .. } => *hit_pos,
        }
    }

    /// Returns the surface normal of the impact.
    #[must_use]
    pub const fn normal(&self) -> Vector3<f64> {
        match self {
            Self::Block { normal, .. } | Self::Entity { normal, .. } => *normal,
        }
    }

    /// Safely returns the face hit if it was a block, otherwise None.
    #[must_use]
    pub const fn face(&self) -> Option<BlockDirection> {
        match self {
            Self::Block { face, .. } => Some(*face),
            Self::Entity { .. } => None,
        }
    }
}
