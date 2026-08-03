use crossbeam::atomic::AtomicCell;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use tokio::sync::RwLock;

use crate::block::blocks::redstone::target_block::TargetBlock;
use crate::entity::projectile::{ProjectileHit, on_target_block_hit};
use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
        player::Player,
    },
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::PotionDurationScaleImpl;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
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
    pub item_stack: RwLock<ItemStack>,
    pub base_damage: AtomicCell<f64>,
    pub pickup: ArrowPickup,
    pub is_critical: AtomicBool,
    pub pierce_level: AtomicU8,
    pub punch_level: AtomicU8,
    pub is_flame: AtomicBool,
    pub in_ground: AtomicBool,
    pub in_ground_time: AtomicU32,
    pub life: AtomicU32,
    pub shake_time: AtomicU8,
    pub has_hit: AtomicBool,
    pub last_block_pos: Arc<std::sync::RwLock<Option<BlockPos>>>,
    pierced_entity_ids: Mutex<HashSet<i32>>,
}

impl ArrowEntity {
    const ARROW_BASE_DAMAGE: f64 = 2.0;
    const WATER_INERTIA: f64 = 0.6;
    const AIR_INERTIA: f64 = 0.99;
    const GRAVITY: f64 = 0.05;
    const DESPAWN_TIME: u32 = 1200;

    pub fn new(entity: Entity, owner_id: Option<i32>) -> Self {
        let item_stack = ItemStack::new(1, Self::default_item(entity.entity_type));
        Self::new_with_item(entity, owner_id, &item_stack, ArrowPickup::Disallowed)
    }

    pub fn new_with_item(
        entity: Entity,
        owner_id: Option<i32>,
        item_stack: &ItemStack,
        pickup: ArrowPickup,
    ) -> Self {
        Self {
            entity,
            owner_id,
            item_stack: RwLock::new(item_stack.copy_with_count(1)),
            base_damage: AtomicCell::new(Self::ARROW_BASE_DAMAGE),
            pickup,
            is_critical: AtomicBool::new(false),
            pierce_level: AtomicU8::new(0),
            punch_level: AtomicU8::new(0),
            is_flame: AtomicBool::new(false),
            in_ground: AtomicBool::new(false),
            in_ground_time: AtomicU32::new(0),
            life: AtomicU32::new(0),
            shake_time: AtomicU8::new(0),
            has_hit: AtomicBool::new(false),
            last_block_pos: Arc::new(std::sync::RwLock::new(None)),
            pierced_entity_ids: Mutex::new(HashSet::new()),
        }
    }

    pub fn new_shot(
        entity: Entity,
        shooter: &Entity,
        item_stack: &ItemStack,
        pickup: ArrowPickup,
    ) -> Self {
        let mut owner_pos = shooter.pos.load();
        owner_pos.y = owner_pos.y + f64::from(shooter.entity_dimension.load().eye_height) - 0.1;
        entity.pos.store(owner_pos);
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));

        Self {
            entity,
            owner_id: Some(shooter.entity_id),
            item_stack: RwLock::new(item_stack.copy_with_count(1)),
            base_damage: AtomicCell::new(Self::ARROW_BASE_DAMAGE),
            pickup,
            is_critical: AtomicBool::new(false),
            pierce_level: AtomicU8::new(0),
            punch_level: AtomicU8::new(0),
            is_flame: AtomicBool::new(false),
            in_ground: AtomicBool::new(false),
            in_ground_time: AtomicU32::new(0),
            life: AtomicU32::new(0),
            shake_time: AtomicU8::new(0),
            has_hit: AtomicBool::new(false),
            last_block_pos: Arc::new(std::sync::RwLock::new(None)),
            pierced_entity_ids: Mutex::new(HashSet::new()),
        }
    }

    #[must_use]
    pub const fn entity_type_for_item(item: &'static Item) -> &'static EntityType {
        if item.id == Item::SPECTRAL_ARROW.id {
            &EntityType::SPECTRAL_ARROW
        } else {
            &EntityType::ARROW
        }
    }

    #[must_use]
    pub const fn default_item(entity_type: &'static EntityType) -> &'static Item {
        if entity_type.id == EntityType::SPECTRAL_ARROW.id {
            &Item::SPECTRAL_ARROW
        } else {
            &Item::ARROW
        }
    }

    fn write_item_stack_nbt(item_stack: &ItemStack, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let mut item = pumpkin_nbt::compound::NbtCompound::new();
        item_stack.copy_with_count(1).write_item_stack(&mut item);
        nbt.put_compound("item", item);
    }

    fn read_item_stack_nbt(nbt: &pumpkin_nbt::compound::NbtCompound) -> Option<ItemStack> {
        nbt.get_compound("item")
            .and_then(ItemStack::read_item_stack)
            .map(|item_stack| item_stack.copy_with_count(1))
    }

    fn pickup_item_stack(item_stack: &ItemStack) -> ItemStack {
        item_stack.copy_with_count(1)
    }

    const fn spectral_glowing_effect() -> pumpkin_data::potion::Effect {
        pumpkin_data::potion::Effect {
            effect_type: &pumpkin_data::effect::StatusEffect::GLOWING,
            duration: 200,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        }
    }

    const fn should_apply_post_hurt_effects(damage_succeeded: bool) -> bool {
        damage_succeeded
    }

    /// `Arrow#doPostHurtEffects` scales durations via `MobEffectInstance#withScaledDuration`:
    /// floor (not round), min 1 tick, infinite (-1) and zero durations untouched
    /// (`#mapDuration`). Distinct from the splash-potion scale-down in `PotionContents`.
    fn scale_arrow_effect_duration(duration: i32, scale: f32) -> i32 {
        if duration == -1 || duration == 0 {
            return duration;
        }
        ((duration as f32 * scale).floor() as i32).max(1)
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

    pub fn set_base_damage(&self, damage: f64) {
        self.base_damage.store(damage);
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

impl NBTStorage for ArrowEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.write_nbt(nbt).await;
            let item_stack = self.item_stack.read().await;
            Self::write_item_stack_nbt(&item_stack, nbt);
        })
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.read_nbt_non_mut(nbt).await;
            if let Some(item_stack) = Self::read_item_stack_nbt(nbt) {
                *self.item_stack.write().await = item_stack;
            }
        })
    }
}

impl EntityBase for ArrowEntity {
    #[allow(clippy::too_many_lines)]
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

            // Spawn critical particle trail while arrow is flying and critical
            if self.is_critical.load(Ordering::Relaxed) {
                world.spawn_particle(
                    entity.pos.load(),
                    Vector3::new(0.0f32, 0.0f32, 0.0f32),
                    0.0,
                    1,
                    Particle::Crit,
                );
            }

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
            if let Some(h) = hit {
                let is_piercing_entity = self.pierce_level.load(Ordering::Relaxed) > 0
                    && matches!(&h, ProjectileHit::Entity { .. });
                if !is_piercing_entity && self.has_hit.swap(true, Ordering::SeqCst) {
                    return;
                }

                // Arrow has its own hit path (doesn't go through
                // ThrownItemEntity::process_tick), so PROJECTILE_LAND needs its own
                // emission mirroring the one in projectile::mod.
                let land_pos = crate::entity::projectile::projectile_land_pos(&h);
                caller.on_hit(h).await;
                crate::entity::projectile::emit_projectile_land(&world, caller, land_pos).await;
            }
        })
    }

    #[expect(
        clippy::too_many_lines,
        reason = "projectile hit handling keeps vanilla branches together"
    )]
    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let world = entity.world.load();

            match hit {
                ProjectileHit::Block {
                    pos, face, hit_pos, ..
                } => {
                    // Arrow hit a block - stick into it
                    self.in_ground.store(true, Ordering::Relaxed);
                    self.shake_time.store(7, Ordering::Relaxed);
                    *self.last_block_pos.write().unwrap() = Some(pos);

                    let block = world.get_block(&pos);
                    if block == &pumpkin_data::Block::TARGET {
                        on_target_block_hit(
                            &world,
                            &pos,
                            face,
                            hit_pos,
                            self.owner_id,
                            TargetBlock::PERSISTENT_PROJECTILE_DELAY,
                        )
                        .await;
                    }

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
                    let chunk_pos = entity.chunk_pos.load();
                    world.broadcast_to_chunk(chunk_pos, &sound_packet);

                    // Reset critical flag
                    self.is_critical.store(false, Ordering::Relaxed);
                }
                ProjectileHit::Entity {
                    entity: target,
                    hit_pos,
                    ..
                } => {
                    let pierce = self.pierce_level.load(Ordering::Relaxed);
                    if pierce > 0 {
                        let target_id = target.get_entity().entity_id;
                        if !self.register_piercing_hit(target_id, pierce) {
                            entity.remove().await;
                            return;
                        }
                    }

                    // Calculate damage
                    let velocity = entity.velocity.load();
                    let power = velocity.length();
                    let mut damage = calculate_arrow_damage(power, self.base_damage.load());

                    // Apply critical hit bonus
                    if self.is_critical.load(Ordering::Relaxed) {
                        let bonus = (rand::random::<u32>() % (damage / 2 + 2) as u32) as i32;
                        damage = damage.saturating_add(bonus);
                    }
                    if self.is_flame.load(Ordering::Relaxed) {
                        target.get_entity().set_on_fire_for_ticks(100);
                    }

                    let damage_succeeded =
                        target.damage(self, damage as f32, DamageType::ARROW).await;

                    if let Some(living) = target.get_living_entity() {
                        let punch = self.punch_level.load(Ordering::Relaxed);
                        if punch > 0
                            && let Some(owner_id) = self.owner_id
                            && let Some(owner_entity) = world.get_entity_by_id(owner_id)
                        {
                            crate::entity::combat::handle_knockback(
                                owner_entity.get_entity(),
                                target.as_ref(),
                                f64::from(punch) * 0.6,
                            );
                        }

                        // Play hit sound
                        let sound_packet = CSoundEffect::new(
                            IdOr::Id(Sound::EntityArrowHit as u16),
                            SoundCategory::Neutral,
                            &hit_pos,
                            1.0,
                            1.0,
                            0.0,
                        );
                        world.broadcast_packet_all(&sound_packet);

                        if Self::should_apply_post_hurt_effects(damage_succeeded) {
                            let item_stack = self.item_stack.read().await.clone();
                            let scale = item_stack
                                .get_data_component::<PotionDurationScaleImpl>()
                                .map_or(1.0, |component| component.scale);

                            for (
                                effect_type,
                                duration,
                                amplifier,
                                ambient,
                                show_particles,
                                show_icon,
                            ) in crate::item::potion::PotionContents::read_potion_effects(
                                &item_stack,
                            ) {
                                living
                                    .add_effect(pumpkin_data::potion::Effect {
                                        effect_type,
                                        duration: Self::scale_arrow_effect_duration(
                                            duration, scale,
                                        ),
                                        amplifier,
                                        ambient,
                                        show_particles,
                                        show_icon,
                                        blend: false,
                                    })
                                    .await;
                            }

                            if entity.entity_type.id == EntityType::SPECTRAL_ARROW.id {
                                living.add_effect(Self::spectral_glowing_effect()).await;
                            }
                        }
                    }

                    if pierce == 0 {
                        // No piercing - remove arrow
                        entity.remove().await;
                    }
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

    fn on_player_collision<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            // Only allow picking up grounded arrows
            if !self.in_ground.load(Ordering::Relaxed) {
                return;
            }

            if player.living_entity.health.load() <= 0.0 {
                return;
            }

            // Check pickup rules
            match self.pickup {
                ArrowPickup::Disallowed => return,
                ArrowPickup::CreativeOnly if !player.is_creative() => return,
                _ => {}
            }

            // Try to insert an arrow into the player's inventory
            let item_stack = self.item_stack.read().await;
            let mut stack = Self::pickup_item_stack(&item_stack);
            if player.is_creative() || player.inventory.insert_stack_anywhere(&mut stack).await {
                player.living_entity.pickup(&self.entity, 1);

                // Remove arrow entity after pickup
                self.get_entity().remove().await;
            }
        })
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ArrowEntity {
    fn register_piercing_hit(&self, target_id: i32, pierce: u8) -> bool {
        let Ok(mut pierced_entity_ids) = self.pierced_entity_ids.lock() else {
            tracing::error!("arrow pierced-entity set is poisoned");
            return false;
        };
        if pierced_entity_ids.len() >= piercing_hit_limit(pierce) {
            return false;
        }
        pierced_entity_ids.insert(target_id)
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

        // Skip other arrows, item entities, and falling block entities
        if (other_ent.entity_type == &pumpkin_data::entity::EntityType::ARROW
            || other_ent.entity_type == &pumpkin_data::entity::EntityType::SPECTRAL_ARROW)
            || other_ent.entity_type == &pumpkin_data::entity::EntityType::ITEM
            || other_ent.entity_type == &pumpkin_data::entity::EntityType::FALLING_BLOCK
        {
            return true;
        }

        if self.pierce_level.load(Ordering::Relaxed) > 0
            && self
                .pierced_entity_ids
                .lock()
                .is_ok_and(|pierced_entity_ids| pierced_entity_ids.contains(&other_ent.entity_id))
        {
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

const fn calculate_arrow_damage(power: f64, base_damage: f64) -> i32 {
    (power * base_damage).ceil() as i32
}

const fn piercing_hit_limit(pierce_level: u8) -> usize {
    pierce_level as usize + 1
}

#[cfg(test)]
mod tests {
    use super::{ArrowEntity, calculate_arrow_damage, piercing_hit_limit};
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{
        DataComponentImpl, PotionContentsImpl, PotionDurationScaleImpl,
    };
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    fn tipped_payload(count: u8) -> ItemStack {
        let mut tipped = ItemStack::new(32, &Item::TIPPED_ARROW);
        tipped.patch.push((
            DataComponent::PotionContents,
            Some(
                PotionContentsImpl {
                    potion_id: Some(5),
                    custom_color: Some(0x123456),
                    custom_effects: Vec::new(),
                    custom_name: Some("payload".to_string()),
                }
                .to_dyn(),
            ),
        ));
        tipped.patch.push((
            DataComponent::PotionDurationScale,
            Some(PotionDurationScaleImpl { scale: 0.5 }.to_dyn()),
        ));
        tipped.copy_with_count(count)
    }

    #[test]
    fn projectile_payload_keeps_components_at_one_count() {
        let tipped = tipped_payload(32);

        let payload = tipped.copy_with_count(1);

        assert_eq!(payload.item_count, 1);
        assert!(payload.are_items_and_components_equal(&tipped));
        assert_eq!(
            ArrowEntity::entity_type_for_item(payload.item),
            &EntityType::ARROW
        );
        assert_eq!(
            ArrowEntity::entity_type_for_item(&Item::SPECTRAL_ARROW),
            &EntityType::SPECTRAL_ARROW
        );
    }

    #[test]
    fn arrow_nbt_payload_round_trips() {
        let payload = tipped_payload(1);
        let mut nbt = pumpkin_nbt::compound::NbtCompound::new();

        ArrowEntity::write_item_stack_nbt(&payload, &mut nbt);
        let restored = ArrowEntity::read_item_stack_nbt(&nbt).expect("arrow payload should decode");

        assert!(restored.are_equal(&payload));
        assert_eq!(restored.item_count, 1);
    }

    #[test]
    fn grounded_pickup_stack_keeps_exact_arrow_payload() {
        let payload = tipped_payload(32);
        let pickup = ArrowEntity::pickup_item_stack(&payload);

        assert_eq!(pickup.item_count, 1);
        assert!(pickup.are_items_and_components_equal(&payload));
    }

    #[test]
    fn spectral_arrow_applies_vanilla_glowing_effect() {
        let effect = ArrowEntity::spectral_glowing_effect();

        assert_eq!(
            effect.effect_type,
            &pumpkin_data::effect::StatusEffect::GLOWING
        );
        assert_eq!(effect.duration, 200);
        assert_eq!(effect.amplifier, 0);
        assert!(effect.show_particles);
        assert!(effect.show_icon);
    }

    #[test]
    fn post_hurt_effects_require_successful_arrow_damage() {
        assert!(!ArrowEntity::should_apply_post_hurt_effects(false));
        assert!(ArrowEntity::should_apply_post_hurt_effects(true));
    }

    #[test]
    fn arrow_damage_uses_configured_base_damage() {
        assert_eq!(calculate_arrow_damage(1.5, 2.0), 3);
        assert_eq!(calculate_arrow_damage(1.5, 4.0), 6);
    }

    #[test]
    fn piercing_hits_allow_one_more_target_than_the_level() {
        assert_eq!(piercing_hit_limit(1), 2);
        assert_eq!(piercing_hit_limit(4), 5);
    }

    #[test]
    fn arrow_effect_duration_scale_floors_and_clamps_to_one_tick() {
        assert_eq!(ArrowEntity::scale_arrow_effect_duration(900, 0.125), 112);
        assert_eq!(ArrowEntity::scale_arrow_effect_duration(160, 0.125), 20);
        assert_eq!(ArrowEntity::scale_arrow_effect_duration(1, 0.125), 1);
        assert_eq!(ArrowEntity::scale_arrow_effect_duration(-1, 0.125), -1);
        assert_eq!(ArrowEntity::scale_arrow_effect_duration(0, 0.125), 0);
    }
}
