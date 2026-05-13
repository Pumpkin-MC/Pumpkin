use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockDirection;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CEntityPositionSync, Metadata};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use rand::RngExt;

use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::ai::goal::look_around::RandomLookAroundGoal;
use crate::entity::ai::goal::look_at_entity::LookAtEntityGoal;
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::projectile::shulker_bullet::ShulkerBulletEntity;
use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture};

const DEFAULT_ATTACH_FACE: BlockDirection = BlockDirection::Down;
const NO_COLOR: u8 = 16;
const PEEK_PER_TICK: f32 = 0.05;
const MAX_TELEPORT_DISTANCE: i32 = 8;

/// Axis type for bullet creation (the bullet cannot start moving along this axis)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

/// Local extension trait so we can add helpers to the external `BlockDirection` type.
trait BlockDirectionExt {
    fn axis_of(self) -> Axis;
}

impl BlockDirectionExt for BlockDirection {
    fn axis_of(self) -> Axis {
        match self {
            Self::East | Self::West => Axis::X,
            Self::Up | Self::Down => Axis::Y,
            Self::North | Self::South => Axis::Z,
        }
    }
}

pub struct ShulkerEntity {
    pub mob_entity: MobEntity,
    attach_face: AtomicU8,
    peek_amount: AtomicU8,
    color: AtomicU8,
    /// Visual interpolation
    current_peek_amount: AtomicCell<f32>,
    prev_peek_amount: AtomicCell<f32>,
}

impl ShulkerEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let shulker = Self {
            mob_entity,
            attach_face: AtomicU8::new(DEFAULT_ATTACH_FACE as u8),
            peek_amount: AtomicU8::new(0),
            color: AtomicU8::new(NO_COLOR),
            current_peek_amount: AtomicCell::new(0.0),
            prev_peek_amount: AtomicCell::new(0.0),
        };
        let mob_arc = Arc::new(shulker);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            goal_selector.add_goal(
                1,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(4, Box::new(ShulkerAttackGoal::new(mob_arc.clone())));
            goal_selector.add_goal(7, Box::new(ShulkerPeekGoal::new(mob_arc.clone())));
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub fn get_attach_face(&self) -> BlockDirection {
        BlockDirection::try_from(i32::from(self.attach_face.load(Ordering::Relaxed)))
            .unwrap_or(DEFAULT_ATTACH_FACE)
    }

    async fn set_attach_face(&self, face: BlockDirection) {
        self.attach_face.store(face as u8, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity
            .send_meta_data(&[Metadata::new(
                TrackedData::ATTACH_FACE_ID,
                MetaDataType::DIRECTION,
                VarInt(face as i32),
            )])
            .await;
    }

    pub fn get_raw_peek(&self) -> u8 {
        self.peek_amount.load(Ordering::Relaxed)
    }

    pub async fn set_raw_peek(&self, amount: u8) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        if amount == 0 {
            // Closed from open state
            world
                .play_sound_fine(
                    Sound::EntityShulkerClose,
                    SoundCategory::Hostile,
                    &pos,
                    1.0,
                    1.0,
                )
                .await;
        } else if self.peek_amount.load(Ordering::Relaxed) == 0 {
            // Opened from closed state
            world
                .play_sound_fine(
                    Sound::EntityShulkerOpen,
                    SoundCategory::Hostile,
                    &pos,
                    1.0,
                    1.0,
                )
                .await;
        }

        self.peek_amount.store(amount, Ordering::Relaxed);

        entity
            .send_meta_data(&[Metadata::new(
                TrackedData::PEEK_ID,
                MetaDataType::BYTE,
                amount,
            )])
            .await;
    }

    pub fn is_closed(&self) -> bool {
        self.peek_amount.load(Ordering::Relaxed) == 0
    }

    /// Advances `current_peek_amount` toward target by `PEEK_PER_TICK`.
    /// Returns true if the value changed.
    fn update_peek_amount(&self) -> bool {
        let current = self.current_peek_amount.load();
        self.prev_peek_amount.store(current);
        let target = f32::from(self.get_raw_peek()) * 0.01;

        if (current - target).abs() < 1e-6 {
            return false;
        }

        let new = if current > target {
            (current - PEEK_PER_TICK).max(target)
        } else {
            (current + PEEK_PER_TICK).min(target)
        };

        self.current_peek_amount.store(new);
        true
    }

    /// Returns `Some(direction)` if the neighbour block in that direction from
    /// `pos` is solid (the shulker can attach there).
    async fn find_attachable_face(&self, pos: &BlockPos) -> Option<BlockDirection> {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        for dir in BlockDirection::all() {
            let neighbour = pos.offset_direction(dir);
            let state = world.get_block_state(&neighbour).await;
            if state.is_solid() {
                return Some(dir);
            }
        }
        None
    }

    /// Check whether the shulker can stay where it is.
    /// The block at `pos` must be free (air) and the block in the `face` direction from `pos` must be solid.
    async fn can_stay_at(&self, pos: &BlockPos, face: BlockDirection) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();

        // Shulker's own block must not be occupied
        let own_state = world.get_block_state(pos).await;
        if !own_state.is_air() {
            return false;
        }

        // The block the shulker is attached to must still be solid
        let neighbour = pos.offset_direction(face);
        let state = world.get_block_state(&neighbour).await;
        state.is_solid()
    }

    /// Try to find a new attachment point, cascading to a random teleport.
    async fn find_new_attachment(&self) {
        let pos = self.mob_entity.living_entity.entity.block_pos.load();
        if let Some(dir) = self.find_attachable_face(&pos).await {
            self.set_attach_face(dir).await;
        } else {
            self.teleport_somewhere().await;
        }
    }

    /// Attempt to teleport to a random nearby location where the shulker can attach.
    /// Returns `true` on success.
    pub async fn teleport_somewhere(&self) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let base_pos = entity.block_pos.load();
        let world = entity.world.load();

        // Collect all random offsets up-front so ThreadRng (which is !Send) is
        // dropped before any .await boundary.
        let candidates: Vec<(i32, i32, i32)> = {
            let mut rng = rand::rng();
            (0..20)
                .map(|_| {
                    (
                        rng.random_range(-MAX_TELEPORT_DISTANCE..=MAX_TELEPORT_DISTANCE),
                        rng.random_range(-MAX_TELEPORT_DISTANCE..=MAX_TELEPORT_DISTANCE),
                        rng.random_range(-MAX_TELEPORT_DISTANCE..=MAX_TELEPORT_DISTANCE),
                    )
                })
                .collect()
        };

        for (dx, dy, dz) in candidates {
            let candidate = BlockPos::new(base_pos.0.x + dx, base_pos.0.y + dy, base_pos.0.z + dz);

            // Target block must be air and there must be an attachable adjacent solid face.
            let candidate_state = world.get_block_state(&candidate).await;
            if !candidate_state.is_air() {
                continue;
            }

            if let Some(dir) = self.find_attachable_face(&candidate).await {
                let new_pos = Vector3::new(
                    f64::from(candidate.0.x) + 0.5,
                    f64::from(candidate.0.y),
                    f64::from(candidate.0.z) + 0.5,
                );

                self.set_attach_face(dir).await;
                entity.set_pos(new_pos);

                world
                    .broadcast_packet_all(&CEntityPositionSync::new(
                        entity.entity_id.into(),
                        new_pos,
                        Vector3::new(0.0, 0.0, 0.0),
                        entity.yaw.load(),
                        entity.pitch.load(),
                        entity.on_ground.load(Ordering::Relaxed),
                    ))
                    .await;

                entity.last_sent_pos.store(new_pos);

                world
                    .play_sound_fine(
                        Sound::EntityShulkerTeleport,
                        SoundCategory::Hostile,
                        &new_pos,
                        1.0,
                        1.0,
                    )
                    .await;

                // Close the shulker and drop the current target after teleport.
                self.set_raw_peek(0).await;
                self.mob_entity.target.lock().await.take();

                return true;
            }
        }
        false
    }

    pub async fn on_shulker_damage(&self, _damage_type: DamageType) {
        let living = &self.mob_entity.living_entity;
        let health = living.health.load();
        let max = living.get_max_health();

        // Teleport at half-health (random 1-in-4 chance)
        if health < max * 0.5 && rand::rng().random_range(0..4) == 0 {
            self.teleport_somewhere().await;
        }

        // pre_damage for arrow blocking below.
    }
}

impl NBTStorage for ShulkerEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_byte("AttachFace", self.attach_face.load(Ordering::Relaxed) as i8);
            nbt.put_byte("PeekAmount", self.peek_amount.load(Ordering::Relaxed) as i8);
            nbt.put_byte("Color", self.color.load(Ordering::Relaxed) as i8);
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            if let Some(face) = nbt.get_byte("AttachFace") {
                self.attach_face.store(face as u8, Ordering::Relaxed);
            }
            if let Some(peek) = nbt.get_byte("PeekAmount") {
                self.peek_amount.store(peek as u8, Ordering::Relaxed);
            }
            if let Some(color) = nbt.get_byte("Color") {
                self.color.store(color as u8, Ordering::Relaxed);
            }
        })
    }
}

impl Mob for ShulkerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;

            if !entity.is_alive() {
                return;
            }

            entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));

            // Advance peek interpolation
            self.update_peek_amount();

            // Ensure the current attachment face still has a solid block behind it.
            let pos = entity.block_pos.load();
            let face = self.get_attach_face();
            if !self.can_stay_at(&pos, face).await {
                self.find_new_attachment().await;
            }
        })
    }

    fn on_damage<'a>(
        &'a self,
        damage_type: DamageType,
        _source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.on_shulker_damage(damage_type).await;
        })
    }

    /// When closed, block arrows entirely.
    fn pre_damage<'a>(
        &'a self,
        damage_type: DamageType,
        _source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if self.is_closed() && damage_type == DamageType::ARROW {
                return false;
            }
            true
        })
    }

    /// Apply armor modifier (20 armor) reduction when closed.
    fn modify_incoming_damage(&self, amount: f32, damage_type: DamageType) -> f32 {
        if self.is_closed() && !crate::entity::living::bypasses_armor_durability(&damage_type) {
            const ARMOR: f32 = 20.0;
            let f1 = (ARMOR - amount * 0.5f32).clamp(ARMOR * 0.2f32, 20.0f32);
            (amount * (1.0f32 - f1 / 25.0f32)).max(0.0)
        } else {
            amount
        }
    }
}

struct ShulkerAttackGoal {
    shulker: Arc<ShulkerEntity>,
    attack_cooldown: AtomicI32,
}

impl ShulkerAttackGoal {
    const fn new(shulker: Arc<ShulkerEntity>) -> Self {
        Self {
            shulker,
            attack_cooldown: AtomicI32::new(20),
        }
    }
}

impl Goal for ShulkerAttackGoal {
    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }

    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target
                .as_ref()
                .is_some_and(|t| t.get_living_entity().is_some_and(|l| l.entity.is_alive()))
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let target = mob.get_mob_entity().target.lock().await;
            target
                .as_ref()
                .is_some_and(|t| t.get_living_entity().is_some_and(|l| l.entity.is_alive()))
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.attack_cooldown.store(20, Ordering::Relaxed);
            self.shulker.set_raw_peek(100).await;
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.shulker.set_raw_peek(0).await;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let mob_entity = mob.get_mob_entity();
            let target_arc = {
                let guard = mob_entity.target.lock().await;
                guard.clone()
            };

            let Some(target) = target_arc else {
                return;
            };

            if !target.get_entity().is_alive() {
                return;
            }

            let entity = &mob_entity.living_entity.entity;
            let shulker_pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            let dist_sq = shulker_pos.squared_distance_to_vec(&target_pos);

            // De-target if too far (>20 blocks)
            if dist_sq > 400.0 {
                mob_entity.target.lock().await.take();
                return;
            }

            let cooldown = self.attack_cooldown.fetch_sub(1, Ordering::Relaxed) - 1;
            if cooldown <= 0 {
                // Reset cooldown
                let new_cd = 20 + mob.get_random().random_range(0..5) * 10;
                self.attack_cooldown.store(new_cd, Ordering::Relaxed);

                // Spawn bullet
                let world = entity.world.load();
                let target_pos = target.get_entity().pos.load();
                let bullet = ShulkerBulletEntity::new(
                    entity,
                    target.get_entity().entity_id,
                    target_pos,
                    self.shulker.get_attach_face().axis_of(),
                );
                let bullet_arc = Arc::new(bullet);
                world.spawn_entity(bullet_arc).await;

                // Shoot sound (random pitch)
                let pitch = 1.0
                    + (mob.get_random().random::<f32>() - mob.get_random().random::<f32>()) * 0.2;
                world
                    .play_sound_fine(
                        Sound::EntityShulkerShoot,
                        SoundCategory::Hostile,
                        &shulker_pos,
                        2.0,
                        pitch,
                    )
                    .await;
            }
        })
    }
}

struct ShulkerPeekGoal {
    shulker: Arc<ShulkerEntity>,
    peek_time: AtomicI32,
}

impl ShulkerPeekGoal {
    const fn new(shulker: Arc<ShulkerEntity>) -> Self {
        Self {
            shulker,
            peek_time: AtomicI32::new(0),
        }
    }
}

impl Goal for ShulkerPeekGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let has_target = mob.get_mob_entity().target.lock().await.is_some();
            if has_target {
                return false;
            }
            if mob.get_random().random_range(0..40) != 0 {
                return false;
            }
            let pos = mob.get_mob_entity().living_entity.entity.block_pos.load();
            let face = self.shulker.get_attach_face();
            self.shulker.can_stay_at(&pos, face).await
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let has_target = mob.get_mob_entity().target.lock().await.is_some();
            !has_target && self.peek_time.load(Ordering::Relaxed) > 0
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let duration = 20 * (1 + mob.get_random().random_range(0..3));
            self.peek_time.store(duration, Ordering::Relaxed);
            self.shulker.set_raw_peek(30).await;
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let has_target = self.shulker.mob_entity.target.lock().await.is_some();
            if !has_target {
                self.shulker.set_raw_peek(0).await;
            }
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.peek_time.fetch_sub(1, Ordering::Relaxed);
        })
    }
}

trait BlockPosExt {
    fn offset_direction(&self, dir: BlockDirection) -> Self;
}

impl BlockPosExt for BlockPos {
    fn offset_direction(&self, dir: BlockDirection) -> Self {
        let offset = dir.to_offset();
        Self::new(
            self.0.x + offset.x,
            self.0.y + offset.y,
            self.0.z + offset.z,
        )
    }
}
