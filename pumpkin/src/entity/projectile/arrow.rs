use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

use crate::entity::projectile::ProjectileHit;
use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity},
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::CEntityVelocity;
use pumpkin_protocol::java::client::play::CSoundEffect;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

/// Represents the pickup rules for arrows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowPickup {
    Disallowed,
    Allowed,
    CreativeOnly,
}

impl ArrowPickup {
    #[must_use]
    pub const fn from_byte(byte: u8) -> Self {
        match byte {
            1 => Self::Allowed,
            2 => Self::CreativeOnly,
            _ => Self::Disallowed,
        }
    }

    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Disallowed => 0,
            Self::Allowed => 1,
            Self::CreativeOnly => 2,
        }
    }
}

pub struct ArrowEntity {
    pub entity: Entity,
    pub owner_id: Option<i32>,
    pub base_damage: f64,
    pub pickup: ArrowPickup,
    pub is_critical: AtomicBool,
    pub pierce_level: AtomicU8,
    pub in_ground: AtomicBool,
    pub in_ground_time: AtomicU32,
    pub life: AtomicU32,
    pub shake_time: AtomicU8,
    pub has_hit: AtomicBool,
    pub last_block_pos: Arc<std::sync::RwLock<Option<BlockPos>>>,
}

impl ArrowEntity {
    const ARROW_BASE_DAMAGE: f64 = 2.0;
    const WATER_INERTIA: f64 = 0.6;
    const AIR_INERTIA: f64 = 0.99;
    const GRAVITY: f64 = 0.05;
    const DESPAWN_TIME: u32 = 1200;

    pub fn new(entity: Entity, owner_id: Option<i32>) -> Self {
        Self {
            entity,
            owner_id,
            base_damage: Self::ARROW_BASE_DAMAGE,
            pickup: ArrowPickup::Disallowed,
            is_critical: AtomicBool::new(false),
            pierce_level: AtomicU8::new(0),
            in_ground: AtomicBool::new(false),
            in_ground_time: AtomicU32::new(0),
            life: AtomicU32::new(0),
            shake_time: AtomicU8::new(0),
            has_hit: AtomicBool::new(false),
            last_block_pos: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    pub async fn new_shot(entity: Entity, shooter: &Entity, pickup: ArrowPickup) -> Self {
        let mut owner_pos = shooter.pos.load();
        owner_pos.y = owner_pos.y + f64::from(shooter.entity_dimension.load().eye_height) - 0.1;
        entity.pos.store(owner_pos);
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0)).await;

        Self {
            entity,
            owner_id: Some(shooter.entity_id),
            base_damage: Self::ARROW_BASE_DAMAGE,
            pickup,
            is_critical: AtomicBool::new(false),
            pierce_level: AtomicU8::new(0),
            in_ground: AtomicBool::new(false),
            in_ground_time: AtomicU32::new(0),
            life: AtomicU32::new(0),
            shake_time: AtomicU8::new(0),
            has_hit: AtomicBool::new(false),
            last_block_pos: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    pub fn set_velocity_from_rotation(
        &self,
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

    pub fn set_critical(&self, critical: bool) {
        self.is_critical.store(critical, Ordering::Relaxed);
    }

    pub fn set_pierce_level(&self, level: u8) {
        self.pierce_level.store(level, Ordering::Relaxed);
    }

    pub const fn set_base_damage(&self, _damage: f64) {
        // TODO: implement this
    }

    #[allow(dead_code)]
    fn apply_inertia(&self, inertia: f64) {
        let velocity = self.entity.velocity.load();
        self.entity
            .velocity
            .store(velocity.multiply(inertia, inertia, inertia));
    }

    #[allow(dead_code)]
    fn apply_gravity(&self) {
        let mut velocity = self.entity.velocity.load();
        velocity.y -= Self::GRAVITY;
        self.entity.velocity.store(velocity);
    }
}

impl NBTStorage for ArrowEntity {}

impl EntityBase for ArrowEntity {
    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let world = entity.world.load();

            // Handle shake time
            let shake = self.shake_time.load(Ordering::Relaxed);
            if shake > 0 {
                self.shake_time.store(shake - 1, Ordering::Relaxed);
            }

            if self.in_ground.load(Ordering::Relaxed) {
                // Increment in-ground time and life
                let _in_ground_time = self.in_ground_time.fetch_add(1, Ordering::Relaxed);
                let life = self.life.fetch_add(1, Ordering::Relaxed);

                // Despawn after enough time
                if life >= Self::DESPAWN_TIME {
                    entity.remove().await;
                }
                return;
            }

            // Arrow is flying
            let start_pos = entity.pos.load();
            let mut velocity = entity.velocity.load();

            // Apply gravity
            velocity.y -= Self::GRAVITY;

            // Apply inertia (air resistance or water drag)
            let inertia = if entity.touching_water.load(Ordering::Relaxed) {
                Self::WATER_INERTIA
            } else {
                Self::AIR_INERTIA
            };
            velocity = velocity.multiply(inertia, inertia, inertia);

            entity.velocity.store(velocity);

            // Update rotation based on velocity
            let len = velocity.horizontal_length();
            entity.set_rotation(
                velocity.x.atan2(velocity.z) as f32 * 57.295_776,
                velocity.y.atan2(len) as f32 * 57.295_776,
            );

            // Move arrow
            let new_pos = start_pos.add(&velocity);
            entity.set_pos(new_pos);

            // Broadcast velocity update
            let packet = CEntityVelocity::new(entity.entity_id.into(), velocity);
            world.broadcast_packet_all(&packet).await;

            // Check for collisions using raycasting
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
            let (block_cols, block_positions) = world.get_block_collisions(search_box).await;
            for (idx, bb) in block_cols.iter().enumerate() {
                if let Some(t) = calculate_ray_intersection(&start_pos, &velocity, bb)
                    && t < closest_t
                {
                    closest_t = t;

                    // Map back to block pos
                    let mut curr = 0;
                    for (len, pos) in &block_positions {
                        curr += len;
                        if idx < curr {
                            let hit_pos = start_pos.add(&velocity.multiply(t, t, t));
                            hit = Some(ProjectileHit::Block {
                                pos: *pos,
                                face: get_hit_face(hit_pos, *pos),
                                hit_pos,
                                normal: velocity.normalize().multiply(-1.0, -1.0, -1.0),
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
                if let Some(t) = calculate_ray_intersection(&start_pos, &velocity, &ebb)
                    && t < closest_t
                {
                    closest_t = t;
                    let hit_pos = start_pos.add(&velocity.multiply(t, t, t));
                    hit = Some(ProjectileHit::Entity {
                        entity: cand.clone(),
                        hit_pos,
                        normal: velocity.normalize().multiply(-1.0, -1.0, -1.0),
                    });
                }
            }

            // Handle hit
            if let Some(h) = hit {
                // Ensure hit is only processed once
                if self.has_hit.swap(true, Ordering::SeqCst) {
                    return;
                }

                caller.on_hit(h).await;
            }
        })
    }

    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let world = entity.world.load();

            match hit {
                ProjectileHit::Block {
                    pos,
                    face: _,
                    hit_pos,
                    ..
                } => {
                    // Arrow hit a block - stick into it
                    self.in_ground.store(true, Ordering::Relaxed);
                    self.shake_time.store(7, Ordering::Relaxed);
                    *self.last_block_pos.write().unwrap() = Some(pos);

                    // Stop the arrow
                    entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
                    entity.set_pos(hit_pos);

                    // Play sound
                    let sound_packet = CSoundEffect::new(
                        IdOr::Id(Sound::EntityArrowHit as u16),
                        SoundCategory::Neutral,
                        &hit_pos,
                        1.0,
                        1.0,
                        0.0,
                    );
                    world.broadcast_packet_all(&sound_packet).await;

                    // Reset critical flag
                    self.is_critical.store(false, Ordering::Relaxed);
                }
                ProjectileHit::Entity {
                    entity: target,
                    hit_pos,
                    ..
                } => {
                    // Calculate damage
                    let velocity = entity.velocity.load();
                    let power = velocity.length();
                    let mut damage = (power * self.base_damage).ceil() as i32;

                    // Apply critical hit bonus
                    if self.is_critical.load(Ordering::Relaxed) {
                        let bonus = (rand::random::<u32>() % (damage / 2 + 2) as u32) as i32;
                        damage = damage.saturating_add(bonus);
                    }

                    if let Some(living) = target.get_living_entity() {
                        // Apply damage to target
                        living
                            .damage(living, damage as f32, DamageType::ARROW)
                            .await;

                        // Play hit sound
                        let sound_packet = CSoundEffect::new(
                            IdOr::Id(Sound::EntityArrowHit as u16),
                            SoundCategory::Neutral,
                            &hit_pos,
                            1.0,
                            1.0,
                            0.0,
                        );
                        world.broadcast_packet_all(&sound_packet).await;
                    }

                    // Check pierce level
                    let pierce = self.pierce_level.load(Ordering::Relaxed);
                    if pierce == 0 {
                        // No piercing - remove arrow
                        entity.remove().await;
                    }
                    // If piercing > 0, arrow continues (TODO: would need to track pierced entities)
                }
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    #[allow(dead_code, clippy::unused_self)]
    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    #[allow(dead_code, clippy::unused_self)]
    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }
}

impl ArrowEntity {
    fn should_skip_collision(&self, self_ent: &Entity, other: &Arc<dyn EntityBase>) -> bool {
        let other_ent = other.get_entity();

        // Don't collide with self
        if other_ent.entity_id == self_ent.entity_id {
            return true;
        }

        // Skip owner for initial frames (5 ticks)
        if Some(other_ent.entity_id) == self.owner_id && self_ent.age.load(Ordering::Relaxed) < 5 {
            return true;
        }

        false
    }
}

/// Ray intersection algorithm for AABBs
fn calculate_ray_intersection(
    start: &Vector3<f64>,
    dir: &Vector3<f64>,
    bb: &pumpkin_util::math::boundingbox::BoundingBox,
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
fn get_hit_face(hit_pos: Vector3<f64>, block_pos: BlockPos) -> pumpkin_data::BlockDirection {
    use pumpkin_data::BlockDirection;

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
