use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

use crate::entity::projectile::ProjectileHit;
use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity, player::Player,
    },
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::CEntityVelocity;
use pumpkin_protocol::java::client::play::CSoundEffect;
use pumpkin_protocol::java::client::play::CTakeItemEntity;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TridentPickup {
    Disallowed,
    Allowed,
    CreativeOnly,
}

/// Enchantment levels captured at throw-time so the projectile doesn't need
/// to revisit the carried stack's data components.
#[derive(Debug, Clone, Copy, Default)]
pub struct TridentEnchants {
    pub loyalty: u8,
    pub channeling: bool,
    pub impaling: u8,
}

pub struct TridentEntity {
    pub entity: Entity,
    pub owner_id: Option<i32>,
    pub pickup: TridentPickup,
    pub enchants: TridentEnchants,
    /// The trident item stack carried by this projectile.
    /// On pickup the player receives this exact stack (preserving durability/NBT).
    pub item_stack: Mutex<ItemStack>,
    pub in_ground: AtomicBool,
    pub in_ground_time: AtomicU32,
    pub life: AtomicU32,
    pub shake_time: AtomicU8,
    pub dealt_damage: AtomicBool,
    /// True once Loyalty has begun returning the trident to its owner.
    pub returning: AtomicBool,
    /// Set when the owner has disappeared (disconnect, death) while the
    /// trident still expected to return. Once set, all Loyalty behavior is
    /// disabled and the trident behaves as a plain projectile that will drop
    /// to the ground.
    pub owner_lost: AtomicBool,
    /// Ticks since `returning` became true; used to time the return sound.
    pub return_ticks: AtomicU32,
    pub last_block_pos: Arc<std::sync::RwLock<Option<BlockPos>>>,
}

impl TridentEntity {
    const BASE_DAMAGE: f32 = 8.0;
    /// Bonus damage applied to channeling-triggered lightning hits.
    const CHANNELING_DAMAGE_BONUS: f32 = 5.0;
    const GRAVITY: f64 = 0.05;
    const AIR_INERTIA: f64 = 0.99;
    // Vanilla: trident does not slow down in water/lava.
    const WATER_INERTIA: f64 = 0.99;
    const DESPAWN_TIME: u32 = 1200;

    pub fn new_shot(
        entity: Entity,
        shooter: &Entity,
        pickup: TridentPickup,
        stack: ItemStack,
        enchants: TridentEnchants,
    ) -> Self {
        let mut owner_pos = shooter.pos.load();
        owner_pos.y = owner_pos.y + f64::from(shooter.entity_dimension.load().eye_height) - 0.1;
        entity.pos.store(owner_pos);
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0));

        Self {
            entity,
            owner_id: Some(shooter.entity_id),
            pickup,
            enchants,
            item_stack: Mutex::new(stack),
            in_ground: AtomicBool::new(false),
            in_ground_time: AtomicU32::new(0),
            life: AtomicU32::new(0),
            shake_time: AtomicU8::new(0),
            dealt_damage: AtomicBool::new(false),
            returning: AtomicBool::new(false),
            owner_lost: AtomicBool::new(false),
            return_ticks: AtomicU32::new(0),
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

    fn owner_player(&self) -> Option<Arc<Player>> {
        let id = self.owner_id?;
        let world = self.entity.world.load();
        world.get_player_by_id(id)
    }

    /// Try to summon a lightning bolt at the given position if the conditions
    /// (Channeling enchant + thunderstorm + exposed sky) are met.
    async fn try_channel_lightning(&self, hit_pos: Vector3<f64>) -> bool {
        if !self.enchants.channeling {
            return false;
        }
        let world = self.entity.world.load();
        if !world.is_thundering().await {
            return false;
        }
        let block_pos = hit_pos.to_block_pos();
        if world.get_sky_light_level(&block_pos) < 15 {
            return false;
        }

        let bolt = Entity::new(world.clone(), hit_pos, &EntityType::LIGHTNING_BOLT);
        world.spawn_entity(Arc::new(bolt)).await;

        let sound_packet = CSoundEffect::new(
            IdOr::Id(Sound::ItemTridentThunder as u16),
            SoundCategory::Weather,
            &hit_pos,
            5.0,
            1.0,
            0.0,
        );
        world.broadcast_packet_all(&sound_packet);
        true
    }

    fn begin_return(&self) {
        if self.enchants.loyalty == 0 || self.owner_lost.load(Ordering::Relaxed) {
            return;
        }
        if self.returning.swap(true, Ordering::SeqCst) {
            return;
        }
        self.in_ground.store(false, Ordering::Relaxed);
        self.shake_time.store(0, Ordering::Relaxed);
        self.return_ticks.store(0, Ordering::Relaxed);
    }

    /// Called when the owner is gone mid-return. Cancels Loyalty and drops
    /// the trident: gravity takes over and it will land as a pickupable
    /// (by anyone in creative; otherwise no one, but it can despawn).
    fn lose_owner(&self) {
        self.owner_lost.store(true, Ordering::Relaxed);
        self.returning.store(false, Ordering::Relaxed);
        self.in_ground.store(false, Ordering::Relaxed);
    }
}

impl NBTStorage for TridentEntity {}

impl EntityBase for TridentEntity {
    #[allow(clippy::too_many_lines)]
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let world = entity.world.load();

            // --- Returning phase (Loyalty) ---
            if self.returning.load(Ordering::Relaxed) {
                let owner = self.owner_player();
                let owner_alive = owner
                    .as_ref()
                    .is_some_and(|p| p.living_entity.health.load() > 0.0);
                if !owner_alive {
                    // Owner disconnected or died — abort the return and let
                    // gravity drop the trident to the ground from here.
                    self.lose_owner();
                    // Fall through into the flying phase on the next tick.
                    return;
                }
                let owner = owner.unwrap();

                let pos = entity.pos.load();
                let owner_entity = &owner.living_entity.entity;
                let mut target = owner_entity.pos.load();
                target.y += owner_entity.get_eye_height() * 0.5;
                let to_owner = target.sub(&pos);
                let dist = to_owner.length();

                // Touched the owner — try to give the stack back.
                if dist < 1.0 {
                    let stack = self.item_stack.lock().await.clone();
                    let mut to_insert = stack;
                    let inserted = owner.is_creative()
                        || owner.inventory.insert_stack_anywhere(&mut to_insert).await;
                    if inserted {
                        owner
                            .client
                            .enqueue_packet(&CTakeItemEntity::new(
                                entity.entity_id.into(),
                                owner.entity_id().into(),
                                1u8.into(),
                            ))
                            .await;
                    }
                    entity.remove().await;
                    return;
                }

                // Vanilla LoyaltyReturn:
                //   d = 0.05 * level
                //   velocity = velocity * 0.95 + to_owner.normalize() * d
                // No gravity, no fluid drag — the 0.95 IS the damping.
                let level = f64::from(self.enchants.loyalty);
                let d = 0.05 * level;
                let v = entity.velocity.load();
                let pull = to_owner.normalize().multiply(d, d, d);
                let new_v = Vector3::new(
                    v.x * 0.95 + pull.x,
                    v.y * 0.95 + pull.y,
                    v.z * 0.95 + pull.z,
                );
                entity.velocity.store(new_v);
                entity.set_pos(pos.add(&new_v));

                let len = new_v.horizontal_length();
                entity.set_rotation(
                    new_v.x.atan2(new_v.z) as f32 * 57.295_776,
                    new_v.y.atan2(len) as f32 * 57.295_776,
                );

                let packet = CEntityVelocity::new(entity.entity_id.into(), new_v);
                world.broadcast_packet_all(&packet);

                let ticks = self.return_ticks.fetch_add(1, Ordering::Relaxed);
                if ticks.is_multiple_of(10) {
                    let sound_packet = CSoundEffect::new(
                        IdOr::Id(Sound::ItemTridentReturn as u16),
                        SoundCategory::Neutral,
                        &pos,
                        10.0,
                        1.0,
                        0.0,
                    );
                    world.broadcast_packet_all(&sound_packet);
                }
                return;
            }

            // --- Grounded phase ---
            let shake = self.shake_time.load(Ordering::Relaxed);
            if shake > 0 {
                self.shake_time.store(shake - 1, Ordering::Relaxed);
            }
            if self.in_ground.load(Ordering::Relaxed) {
                let _ = self.in_ground_time.fetch_add(1, Ordering::Relaxed);
                let life = self.life.fetch_add(1, Ordering::Relaxed);
                if life >= Self::DESPAWN_TIME {
                    entity.remove().await;
                }
                return;
            }

            // --- Flying phase ---
            let start_pos = entity.pos.load();
            let mut velocity = entity.velocity.load();
            velocity.y -= Self::GRAVITY;

            let inertia = if entity.touching_water.load(Ordering::Relaxed) {
                Self::WATER_INERTIA
            } else {
                Self::AIR_INERTIA
            };
            velocity = velocity.multiply(inertia, inertia, inertia);
            entity.velocity.store(velocity);

            let len = velocity.horizontal_length();
            entity.set_rotation(
                velocity.x.atan2(velocity.z) as f32 * 57.295_776,
                velocity.y.atan2(len) as f32 * 57.295_776,
            );

            let new_pos = start_pos.add(&velocity);
            entity.set_pos(new_pos);

            let packet = CEntityVelocity::new(entity.entity_id.into(), velocity);
            world.broadcast_packet_all(&packet);

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
                .get_block_collisions(search_box, self.get_entity())
                .await;
            for (idx, bb) in block_cols.iter().enumerate() {
                if let Some(t) = calculate_ray_intersection(&start_pos, &velocity, bb)
                    && t < closest_t
                {
                    closest_t = t;
                    let mut curr = 0;
                    for (len_blk, pos) in &block_positions {
                        curr += len_blk;
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

            // After dealing damage the trident cannot hit any more entities.
            let candidates = if self.dealt_damage.load(Ordering::Relaxed) {
                Vec::new()
            } else {
                world.get_entities_at_box(&search_box)
            };
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

            if let Some(h) = hit {
                caller.on_hit(h).await;
            }
        })
    }

    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let world = entity.world.load();

            match hit {
                ProjectileHit::Block { pos, hit_pos, .. } => {
                    if self.enchants.loyalty > 0 {
                        // Don't stick — initiate Loyalty return.
                        self.begin_return();
                        return;
                    }
                    self.in_ground.store(true, Ordering::Relaxed);
                    self.shake_time.store(7, Ordering::Relaxed);
                    *self.last_block_pos.write().unwrap() = Some(pos);

                    entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
                    entity.set_pos(hit_pos);

                    let sound_packet = CSoundEffect::new(
                        IdOr::Id(Sound::ItemTridentHitGround as u16),
                        SoundCategory::Neutral,
                        &hit_pos,
                        1.0,
                        1.0,
                        0.0,
                    );
                    world.broadcast_packet_all(&sound_packet);
                }
                ProjectileHit::Entity {
                    entity: target,
                    hit_pos,
                    ..
                } => {
                    if self.dealt_damage.swap(true, Ordering::SeqCst) {
                        return;
                    }

                    // Impaling: +2.5 dmg/level vs entities sensitive to impaling.
                    let mut damage = Self::BASE_DAMAGE;
                    if self.enchants.impaling > 0 {
                        let target_type = target.get_entity().entity_type;
                        if target_type.has_tag(&tag::EntityType::MINECRAFT_SENSITIVE_TO_IMPALING) {
                            damage += f32::from(self.enchants.impaling) * 2.5;
                        }
                    }

                    let pre_velocity = entity.velocity.load();
                    target.damage(&*target, damage, DamageType::TRIDENT).await;

                    if target.get_living_entity().is_some() {
                        let target_entity = target.get_entity();
                        target_entity.apply_knockback(0.6, pre_velocity.x, pre_velocity.z);
                    }

                    // Channeling: lightning at the hit position. The bolt
                    // itself deals additional damage when it strikes.
                    let channeled = self.try_channel_lightning(hit_pos).await;
                    if channeled && target.get_living_entity().is_some() {
                        target
                            .damage(
                                &*target,
                                Self::CHANNELING_DAMAGE_BONUS,
                                DamageType::LIGHTNING_BOLT,
                            )
                            .await;
                    }

                    let sound_packet = CSoundEffect::new(
                        IdOr::Id(Sound::ItemTridentHit as u16),
                        SoundCategory::Neutral,
                        &hit_pos,
                        1.0,
                        1.0,
                        0.0,
                    );
                    world.broadcast_packet_all(&sound_packet);

                    if self.enchants.loyalty > 0 {
                        // Return immediately without bouncing.
                        self.begin_return();
                    } else {
                        // Vanilla bounce: weakly reflect so the trident falls down.
                        entity
                            .velocity
                            .store(pre_velocity.multiply(-0.01, -0.1, -0.01));
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
            // Returning tridents are picked up by their owner on contact (handled in tick).
            if self.returning.load(Ordering::Relaxed) {
                return;
            }
            if !self.in_ground.load(Ordering::Relaxed) {
                return;
            }
            if player.living_entity.health.load() <= 0.0 {
                return;
            }
            match self.pickup {
                TridentPickup::Disallowed => return,
                TridentPickup::CreativeOnly if !player.is_creative() => return,
                _ => {}
            }

            // Owner-only pickup, unless creative OR the owner has been lost
            // (disconnected / died mid-return) — in which case any player may
            // pick the trident up off the ground.
            let owner_lost = self.owner_lost.load(Ordering::Relaxed);
            let owner_match = self.owner_id.is_none_or(|id| id == player.entity_id());
            if !owner_match && !player.is_creative() && !owner_lost {
                return;
            }

            let mut stack = self.item_stack.lock().await.clone();
            if player.is_creative() || player.inventory.insert_stack_anywhere(&mut stack).await {
                player
                    .client
                    .enqueue_packet(&CTakeItemEntity::new(
                        self.entity.entity_id.into(),
                        player.entity_id().into(),
                        1u8.into(),
                    ))
                    .await;
                self.get_entity().remove().await;
            }
        })
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TridentEntity {
    fn should_skip_collision(&self, self_ent: &Entity, other: &Arc<dyn EntityBase>) -> bool {
        let other_ent = other.get_entity();

        if other_ent.entity_id == self_ent.entity_id {
            return true;
        }
        if Some(other_ent.entity_id) == self.owner_id && self_ent.age.load(Ordering::Relaxed) < 5 {
            return true;
        }
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
