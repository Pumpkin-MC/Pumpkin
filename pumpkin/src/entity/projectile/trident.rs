use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use tokio::sync::Mutex;

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity, player::Player,
    },
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::{CEntityVelocity, CSoundEffect};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::ProjectileHit;
use super::arrow::ArrowPickup;

pub struct TridentEntity {
    pub entity: Entity,
    pub owner_id: Option<i32>,
    pub item_stack: Arc<Mutex<ItemStack>>,
    pub pickup: ArrowPickup,
    pub in_ground: AtomicBool,
    pub in_ground_time: AtomicU32,
    pub life: AtomicU32,
    pub shake_time: AtomicU8,
    pub has_hit: AtomicBool,
    pub last_block_pos: Arc<std::sync::RwLock<Option<BlockPos>>>,
}

impl TridentEntity {
    const BASE_DAMAGE: f64 = 8.0;
    const AIR_INERTIA: f64 = 0.99;
    const WATER_INERTIA: f64 = 0.9;
    const GRAVITY: f64 = 0.05;
    const DESPAWN_TIME: u32 = 1200;

    pub fn new(entity: Entity, owner_id: Option<i32>) -> Self {
        Self {
            entity,
            owner_id,
            item_stack: Arc::new(Mutex::new(ItemStack::new(1, &Item::TRIDENT))),
            pickup: ArrowPickup::Disallowed,
            in_ground: AtomicBool::new(false),
            in_ground_time: AtomicU32::new(0),
            life: AtomicU32::new(0),
            shake_time: AtomicU8::new(0),
            has_hit: AtomicBool::new(false),
            last_block_pos: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    pub fn new_shot(
        entity: Entity,
        shooter: &Entity,
        item_stack: ItemStack,
        pickup: ArrowPickup,
    ) -> Self {
        let mut owner_pos = shooter.pos.load();
        owner_pos.y = owner_pos.y + f64::from(shooter.entity_dimension.load().eye_height) - 0.1;
        entity.pos.store(owner_pos);
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));

        Self {
            entity,
            owner_id: Some(shooter.entity_id),
            item_stack: Arc::new(Mutex::new(item_stack)),
            pickup,
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

        // Skip other projectiles and item entities
        if other_ent.entity_type == &pumpkin_data::entity::EntityType::ARROW
            || other_ent.entity_type == &pumpkin_data::entity::EntityType::TRIDENT
            || other_ent.entity_type == &pumpkin_data::entity::EntityType::ITEM
            || other_ent.entity_type == &pumpkin_data::entity::EntityType::FALLING_BLOCK
        {
            return true;
        }

        false
    }
}

impl NBTStorage for TridentEntity {}

impl EntityBase for TridentEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
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
                let _in_ground_time = self.in_ground_time.fetch_add(1, Ordering::Relaxed);
                let life = self.life.fetch_add(1, Ordering::Relaxed);

                // Despawn after enough time
                if life >= Self::DESPAWN_TIME {
                    entity.remove().await;
                }
                return;
            }

            // Trident is flying
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

            // Move trident
            let new_pos = start_pos.add(&velocity);
            entity.set_pos(new_pos);

            // Broadcast velocity update
            let packet = CEntityVelocity::new(entity.entity_id.into(), velocity);
            let chunk_pos = entity.chunk_pos.load();
            world.broadcast_to_chunk(chunk_pos, &packet);

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
            let (block_cols, block_positions) = world
                .get_block_collisions(search_box, self.get_entity())
                .await;
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
            if let Some(h) = hit
                && !self.has_hit.swap(true, Ordering::SeqCst)
            {
                caller.on_hit(h).await;
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let world = entity.world.load();

            match hit {
                ProjectileHit::Block { pos, hit_pos, .. } => {
                    self.in_ground.store(true, Ordering::Relaxed);
                    self.shake_time.store(7, Ordering::Relaxed);
                    *self.last_block_pos.write().unwrap() = Some(pos);

                    // Stop the trident
                    entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
                    entity.set_pos(hit_pos);

                    // Play sound
                    let sound_packet = CSoundEffect::new(
                        IdOr::Id(Sound::ItemTridentHitGround as u16),
                        SoundCategory::Neutral,
                        &hit_pos,
                        1.0,
                        1.0,
                        0.0,
                    );
                    let chunk_pos = entity.chunk_pos.load();
                    world.broadcast_to_chunk(chunk_pos, &sound_packet);
                }
                ProjectileHit::Entity {
                    entity: target,
                    hit_pos,
                    ..
                } => {
                    let mut damage = Self::BASE_DAMAGE;

                    // Apply Impaling enchantment extra damage
                    if let Some(enchantments) = self
                        .item_stack
                        .lock()
                        .await
                        .get_data_component::<pumpkin_data::data_component_impl::EnchantmentsImpl>(
                    ) {
                        for (enchantment, level) in enchantments.enchantment.iter() {
                            if **enchantment == pumpkin_data::Enchantment::IMPALING {
                                let in_water =
                                    target.get_entity().touching_water.load(Ordering::Relaxed);
                                if in_water {
                                    damage += 1.25 * f64::from(*level);
                                }
                            }
                        }
                    }

                    target
                        .damage(&*target, damage as f32, DamageType::TRIDENT)
                        .await;

                    // Play hit sound
                    let sound_packet = CSoundEffect::new(
                        IdOr::Id(Sound::ItemTridentHit as u16),
                        SoundCategory::Neutral,
                        &hit_pos,
                        1.0,
                        1.0,
                        0.0,
                    );
                    world.broadcast_packet_all(&sound_packet);

                    // Standard bounce/fall-back behavior
                    entity.velocity.store(Vector3::new(0.0, -0.1, 0.0));
                    self.has_hit.store(false, Ordering::Relaxed); // Let it hit the ground
                }
            }
        })
    }

    fn on_player_collision<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            // Can only pick up when on the ground
            if !self.in_ground.load(Ordering::Relaxed) {
                return;
            }

            if player.living_entity.health.load() <= 0.0 {
                return;
            }

            match self.pickup {
                ArrowPickup::Disallowed => return,
                ArrowPickup::CreativeOnly if !player.is_creative() => return,
                _ => {}
            }

            let mut stack = self.item_stack.lock().await.clone();
            if player.is_creative() || player.inventory.insert_stack_anywhere(&mut stack).await {
                player.living_entity.pickup(&self.entity, 1);
                self.get_entity().remove().await;
            }
        })
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
fn get_hit_face(hit_pos: Vector3<f64>, block_pos: BlockPos) -> pumpkin_data::BlockDirection {
    let local = hit_pos.sub(&block_pos.0.to_f64());
    let eps = 1.0e-4;

    if local.x <= eps {
        pumpkin_data::BlockDirection::West
    } else if local.x >= 1.0 - eps {
        pumpkin_data::BlockDirection::East
    } else if local.y <= eps {
        pumpkin_data::BlockDirection::Down
    } else if local.y >= 1.0 - eps {
        pumpkin_data::BlockDirection::Up
    } else if local.z <= eps {
        pumpkin_data::BlockDirection::North
    } else {
        pumpkin_data::BlockDirection::South
    }
}
