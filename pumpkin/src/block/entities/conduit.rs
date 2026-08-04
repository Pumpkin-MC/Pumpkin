use super::BlockEntity;
use crate::entity::living::LivingEntity;
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::MobCategory;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering};
use tokio::sync::Mutex;
use uuid::Uuid;

/// `ConduitBlockEntity.MIN_ACTIVE_SIZE`
const MIN_ACTIVE_SIZE: usize = 16;
/// `ConduitBlockEntity.MIN_KILL_SIZE`
const MIN_KILL_SIZE: usize = 42;
/// `ConduitBlockEntity.KILL_RANGE`
const KILL_RANGE: f64 = 8.0;
/// `ConduitBlockEntity.ROTATION_SPEED`
const ROTATION_SPEED: f32 = -0.0375;
/// `applyEffects`: `new MobEffectInstance(MobEffects.CONDUIT_POWER, 260, ...)`
const CONDUIT_POWER_DURATION: i32 = 260;

/// `ConduitBlockEntity.VALID_BLOCKS`
fn is_valid_frame_block(block: &Block) -> bool {
    block.id == Block::PRISMARINE.id
        || block.id == Block::PRISMARINE_BRICKS.id
        || block.id == Block::SEA_LANTERN.id
        || block.id == Block::DARK_PRISMARINE.id
}

/// The 42 offsets `updateShape` scans in the outer shell of the 5x5x5 cube around the
/// conduit. Pure geometry, independent of any world state, so it is unit-testable in
/// isolation.
fn frame_offsets() -> Vec<(i32, i32, i32)> {
    let mut offsets = Vec::with_capacity(MIN_KILL_SIZE);
    for ox in -2i32..=2 {
        for oy in -2i32..=2 {
            for oz in -2i32..=2 {
                let (ax, ay, az) = (ox.abs(), oy.abs(), oz.abs());
                let outside_inner_cube = ax > 1 || ay > 1 || az > 1;
                let on_an_axis_arm = (ox == 0 && (ay == 2 || az == 2))
                    || (oy == 0 && (ax == 2 || az == 2))
                    || (oz == 0 && (ax == 2 || ay == 2));
                if outside_inner_cube && on_an_axis_arm {
                    offsets.push((ox, oy, oz));
                }
            }
        }
    }
    offsets
}

/// `ConduitBlockEntity.updateShape`, split from world access so the offset geometry can
/// be tested without a live `World`. `is_water` mirrors `Level.isWaterAt` for the inner
/// 3x3x3 cube, `frame_block` mirrors `Level.getBlockState(pos).getBlock()` for the outer
/// shell.
fn scan_shape(
    origin: BlockPos,
    mut is_water: impl FnMut(BlockPos) -> bool,
    mut frame_block: impl FnMut(BlockPos) -> &'static Block,
) -> Vec<BlockPos> {
    for ox in -1..=1 {
        for oy in -1..=1 {
            for oz in -1..=1 {
                if !is_water(origin.offset(Vector3::new(ox, oy, oz))) {
                    return Vec::new();
                }
            }
        }
    }

    frame_offsets()
        .into_iter()
        .filter_map(|(ox, oy, oz)| {
            let pos = origin.offset(Vector3::new(ox, oy, oz));
            is_valid_frame_block(frame_block(pos)).then_some(pos)
        })
        .collect()
}

pub struct ConduitBlockEntity {
    pub position: BlockPos,
    tick_count: AtomicI32,
    active_rotation: AtomicCell<f32>,
    is_active: AtomicBool,
    is_hunting: AtomicBool,
    effect_blocks: Mutex<Vec<BlockPos>>,
    /// `EntityReference<LivingEntity> destroyTarget`, reduced to the persisted UUID.
    destroy_target: Mutex<Option<Uuid>>,
    next_ambient_sound_activation: AtomicI64,
}

impl BlockEntity for ConduitBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let destroy_target = nbt.get_int_array("Target").and_then(uuid_from_int_array);
        Self {
            position,
            tick_count: AtomicI32::new(0),
            active_rotation: AtomicCell::new(0.0),
            is_active: AtomicBool::new(false),
            is_hunting: AtomicBool::new(false),
            effect_blocks: Mutex::new(Vec::new()),
            destroy_target: Mutex::new(destroy_target),
            next_ambient_sound_activation: AtomicI64::new(0),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let target = *self.destroy_target.lock().await;
            if let Some(uuid) = target {
                nbt.put("Target", uuid_to_int_array(uuid));
            }
        })
    }

    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move { self.server_tick(world).await })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(target) = self.destroy_target.try_lock()
            && let Some(uuid) = *target
        {
            nbt.put("Target", uuid_to_int_array(uuid));
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ConduitBlockEntity {
    pub const ID: &'static str = "minecraft:conduit";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            tick_count: AtomicI32::new(0),
            active_rotation: AtomicCell::new(0.0),
            is_active: AtomicBool::new(false),
            is_hunting: AtomicBool::new(false),
            effect_blocks: Mutex::new(Vec::new()),
            destroy_target: Mutex::new(None),
            next_ambient_sound_activation: AtomicI64::new(0),
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }

    pub fn is_hunting(&self) -> bool {
        self.is_hunting.load(Ordering::Relaxed)
    }

    /// `getActiveRotation`: the rendered rotation for a given partial tick.
    pub fn get_active_rotation(&self, partial_tick: f32) -> f32 {
        (self.active_rotation.load() + partial_tick) * ROTATION_SPEED
    }

    /// `ConduitBlockEntity.serverTick`
    async fn server_tick(&self, world: &Arc<World>) {
        self.tick_count.fetch_add(1, Ordering::Relaxed);
        let game_time = world.get_world_age().await;

        if game_time % 40 == 0 {
            let shape = scan_shape(
                self.position,
                |pos| world.get_fluid(&pos) == &Fluid::FLOWING_WATER,
                |pos| world.get_block(&pos),
            );
            let active = shape.len() >= MIN_ACTIVE_SIZE;
            let hunting = shape.len() >= MIN_KILL_SIZE;

            if active != self.is_active.load(Ordering::Relaxed) {
                let sound = if active {
                    Sound::BlockConduitActivate
                } else {
                    Sound::BlockConduitDeactivate
                };
                world.play_block_sound(sound, SoundCategory::Blocks, self.position);
            }
            self.is_active.store(active, Ordering::Relaxed);
            self.is_hunting.store(hunting, Ordering::Relaxed);
            *self.effect_blocks.lock().await = shape;

            if active {
                self.apply_effects(world).await;
                self.update_and_attack_target(world, hunting).await;
            }
        }

        if self.is_active() {
            if game_time % 80 == 0 {
                world.play_block_sound(
                    Sound::BlockConduitAmbient,
                    SoundCategory::Blocks,
                    self.position,
                );
            }

            if game_time > self.next_ambient_sound_activation.load(Ordering::Relaxed) {
                let delay = 60 + rand::rng().random_range(0..40);
                self.next_ambient_sound_activation
                    .store(game_time + delay, Ordering::Relaxed);
                world.play_block_sound(
                    Sound::BlockConduitAmbientShort,
                    SoundCategory::Blocks,
                    self.position,
                );
            }
        }

        if self.is_active() {
            self.active_rotation
                .store(self.active_rotation.load() + 1.0);
        }
    }

    /// `ConduitBlockEntity.applyEffects`
    async fn apply_effects(&self, world: &Arc<World>) {
        let active_size = self.effect_blocks.lock().await.len() as i32;
        let effect_range = f64::from(active_size / 7 * 16);
        if effect_range <= 0.0 {
            return;
        }

        let bb = BoundingBox::from_block(&self.position)
            .expand(effect_range, effect_range, effect_range)
            .expand_towards(0.0, f64::from(world.dimension.height), 0.0);

        for player in world.get_players_at_box(&bb) {
            let player_pos = player.living_entity.entity.block_pos.load();
            let close_enough =
                self.position.squared_distance(&player_pos) < (effect_range * effect_range) as i32;
            if close_enough && is_wet(&player.living_entity, world).await {
                player
                    .add_effect(Effect {
                        effect_type: &StatusEffect::CONDUIT_POWER,
                        duration: CONDUIT_POWER_DURATION,
                        amplifier: 0,
                        ambient: true,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    })
                    .await;
            }
        }
    }

    /// `ConduitBlockEntity.updateAndAttackTarget`
    async fn update_and_attack_target(&self, world: &Arc<World>, hunting: bool) {
        let new_target = self.update_destroy_target(world, hunting).await;

        if let Some(uuid) = new_target
            && let Some(target) = world.get_entity_by_uuid(uuid)
            && target.get_entity().is_alive()
            && target
                .get_living_entity()
                .is_some_and(|living| living.health.load() > 0.0)
        {
            let pos = target.get_entity().pos.load();
            world.play_sound(Sound::BlockConduitAttackTarget, SoundCategory::Blocks, &pos);
            target.damage(&*target, 4.0, DamageType::MAGIC).await;
        }

        *self.destroy_target.lock().await = new_target;
    }

    /// `ConduitBlockEntity.updateDestroyTarget`
    async fn update_destroy_target(&self, world: &Arc<World>, hunting: bool) -> Option<Uuid> {
        if !hunting {
            return None;
        }

        let current = *self.destroy_target.lock().await;
        let Some(uuid) = current else {
            return self.select_new_target(world).await;
        };

        let still_valid = world.get_entity_by_uuid(uuid).is_some_and(|target| {
            target.get_entity().is_alive()
                && target
                    .get_living_entity()
                    .is_some_and(|living| living.health.load() > 0.0)
                && self
                    .position
                    .squared_distance(&target.get_entity().block_pos.load())
                    < (KILL_RANGE * KILL_RANGE) as i32
        });

        still_valid.then_some(uuid)
    }

    /// `ConduitBlockEntity.selectNewTarget`. Vanilla filters candidates by the `Enemy`
    /// marker interface; Pumpkin has no equivalent trait, so `MobCategory::MONSTER` is
    /// used as the closest available proxy (it covers effectively all `Enemy`
    /// implementors that can appear at all, at the cost of not matching a couple of
    /// edge-case mobs whose category differs from their `Enemy` status upstream).
    async fn select_new_target(&self, world: &Arc<World>) -> Option<Uuid> {
        let bb = BoundingBox::from_block(&self.position).expand(KILL_RANGE, KILL_RANGE, KILL_RANGE);

        let mut candidates = Vec::new();
        for entity in world.get_entities_at_box(&bb) {
            let Some(living) = entity.get_living_entity() else {
                continue;
            };
            if entity.get_entity().entity_type.category.id != MobCategory::MONSTER.id {
                continue;
            }
            if is_wet(living, world).await {
                candidates.push(entity.get_entity().entity_uuid);
            }
        }

        if candidates.is_empty() {
            return None;
        }
        let idx = rand::rng().random_range(0..candidates.len());
        Some(candidates[idx])
    }
}

/// `LivingEntity.isInWaterOrRain`, approximated with the primitives this codebase
/// already exposes: `LivingEntity::is_in_water` (block-at-feet check) plus the
/// server's rain-at-position check.
async fn is_wet(living: &LivingEntity, world: &World) -> bool {
    if living.is_in_water() {
        return true;
    }
    if !world.is_raining().await {
        return false;
    }
    world.is_raining_at(&living.entity.block_pos.load()).await
}

/// `UUIDUtil.CODEC`: four big-endian ints, most-significant first.
fn uuid_from_int_array(values: &[i32]) -> Option<Uuid> {
    let &[a, b, c, d] = values else { return None };
    Some(Uuid::from_u128(
        ((a as u32 as u128) << 96)
            | ((b as u32 as u128) << 64)
            | ((c as u32 as u128) << 32)
            | (d as u32 as u128),
    ))
}

fn uuid_to_int_array(u: Uuid) -> NbtTag {
    let v = u.as_u128();
    NbtTag::IntArray(vec![
        (v >> 96) as i32,
        ((v >> 64) & 0xFFFF_FFFF) as i32,
        ((v >> 32) & 0xFFFF_FFFF) as i32,
        (v & 0xFFFF_FFFF) as i32,
    ])
}

#[cfg(test)]
mod test {
    use super::{Block, frame_offsets, is_valid_frame_block, scan_shape};
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector3::Vector3;
    use std::collections::HashSet;

    #[test]
    fn frame_offsets_match_vanilla_scan_count() {
        // `MIN_KILL_SIZE` (42) is exactly the number of positions vanilla's
        // `updateShape` loop can ever add to `effectBlocks`.
        let offsets = frame_offsets();
        assert_eq!(offsets.len(), 42);

        let unique: HashSet<_> = offsets.iter().copied().collect();
        assert_eq!(unique.len(), offsets.len(), "offsets must not repeat");
    }

    #[test]
    fn frame_offsets_stay_outside_inner_cube_and_within_shell() {
        for &(x, y, z) in &frame_offsets() {
            assert!(x.abs() > 1 || y.abs() > 1 || z.abs() > 1);
            assert!(x.abs() <= 2 && y.abs() <= 2 && z.abs() <= 2);
        }
    }

    #[test]
    fn frame_offsets_include_known_arm_positions() {
        let offsets: HashSet<_> = frame_offsets().into_iter().collect();
        // One of the six axis-aligned "plus" arms.
        assert!(offsets.contains(&(2, 0, 0)));
        assert!(offsets.contains(&(-2, 0, 0)));
        assert!(offsets.contains(&(0, 2, 0)));
        assert!(offsets.contains(&(0, 0, 2)));
        // Off-axis shell positions such as (2, 1, 0) do qualify too (ax=2, oz==0).
        assert!(offsets.contains(&(2, 1, 0)));
        // But a true corner like (2, 2, 2) never satisfies the "one axis is zero" arm test.
        assert!(!offsets.contains(&(2, 2, 2)));
    }

    #[test]
    fn is_valid_frame_block_matches_vanilla_list() {
        assert!(is_valid_frame_block(&Block::PRISMARINE));
        assert!(is_valid_frame_block(&Block::PRISMARINE_BRICKS));
        assert!(is_valid_frame_block(&Block::SEA_LANTERN));
        assert!(is_valid_frame_block(&Block::DARK_PRISMARINE));
        assert!(!is_valid_frame_block(&Block::STONE));
        assert!(!is_valid_frame_block(&Block::WATER));
    }

    #[test]
    fn scan_shape_requires_full_water_core() {
        let origin = BlockPos(Vector3::new(0, 64, 0));
        // All frame blocks present, but the inner 3x3x3 core is not fully water.
        let shape = scan_shape(
            origin,
            |pos| pos != origin.offset(Vector3::new(1, 0, 0)),
            |_| &Block::PRISMARINE,
        );
        assert!(shape.is_empty());
    }

    #[test]
    fn scan_shape_counts_only_valid_frame_blocks() {
        let origin = BlockPos(Vector3::new(0, 64, 0));
        let shape = scan_shape(origin, |_| true, |_| &Block::PRISMARINE);
        assert_eq!(shape.len(), 42);

        let shape = scan_shape(origin, |_| true, |_| &Block::STONE);
        assert!(shape.is_empty());
    }

    #[test]
    fn scan_shape_positions_are_relative_to_origin() {
        let origin = BlockPos(Vector3::new(10, 64, -20));
        let shape = scan_shape(origin, |_| true, |_| &Block::SEA_LANTERN);
        assert!(shape.contains(&origin.offset(Vector3::new(2, 0, 0))));
        assert!(!shape.contains(&origin));
    }
}
