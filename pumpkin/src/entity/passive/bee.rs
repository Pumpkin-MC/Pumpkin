use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, AtomicU8, Ordering::Relaxed},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::block_properties::{
    BlockProperties, DoubleBlockHalf, TallSeagrassLikeProperties,
};
use pumpkin_data::block_state::BlockState;
use pumpkin_data::damage::DamageType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, effect::StatusEffect, potion::Effect};
use pumpkin_data::{entity::EntityType, meta_data_type::MetaDataType, tracked_data::TrackedData};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        Controls, Goal, GoalFuture, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    ai::pathfinder::NavigatorGoal,
    mob::{Mob, MobEntity},
};

/// `Bee.FLAG_ROLL`, `Bee.FLAG_HAS_STUNG`, `Bee.FLAG_HAS_NECTAR`.
const FLAG_ROLL: u8 = 2;
const FLAG_HAS_STUNG: u8 = 4;
const FLAG_HAS_NECTAR: u8 = 8;

/// `Bee.STING_DEATH_COUNTDOWN`.
const STING_DEATH_COUNTDOWN: i32 = 1200;
/// `Bee.TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME`.
const TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME: i32 = 3600;
/// `Bee.COOLDOWN_BEFORE_LOCATING_NEW_FLOWER`.
const COOLDOWN_BEFORE_LOCATING_NEW_FLOWER: i32 = 200;
/// `Bee.MIN_FIND_FLOWER_RETRY_COOLDOWN` / `Bee.MAX_FIND_FLOWER_RETRY_COOLDOWN`.
const MIN_FIND_FLOWER_RETRY_COOLDOWN: i32 = 20;
const MAX_FIND_FLOWER_RETRY_COOLDOWN: i32 = 60;

/// `Bee.doHurtTarget`: `POISON_SECONDS_NORMAL` / `POISON_SECONDS_HARD`.
const fn poison_duration(difficulty: Difficulty) -> Option<i32> {
    match difficulty {
        Difficulty::Normal => Some(10 * 20),
        Difficulty::Hard => Some(18 * 20),
        Difficulty::Peaceful | Difficulty::Easy => None,
    }
}

/// `Bee.customServerAiStep`: `random.nextInt(Mth.clamp(1200 - timeSinceSting, 1, 1200)) == 0`.
const fn sting_death_roll_bound(time_since_sting: i32) -> i32 {
    let remaining = STING_DEATH_COUNTDOWN - time_since_sting;
    if remaining < 1 {
        1
    } else if remaining > STING_DEATH_COUNTDOWN {
        STING_DEATH_COUNTDOWN
    } else {
        remaining
    }
}

/// `Bee.isTiredOfLookingForNectar`.
const fn is_tired_of_looking_for_nectar(ticks_without_nectar: i32) -> bool {
    ticks_without_nectar > TICKS_WITHOUT_NECTAR_BEFORE_GOING_HOME
}

/// Represents a Bee, a neutral flying mob that can pollinate crops and sting attackers.
///
/// Wiki: <https://minecraft.wiki/w/Bee>
pub struct BeeEntity {
    pub mob_entity: MobEntity,
    /// `Bee.DATA_FLAGS_ID`.
    flags: AtomicU8,
    /// `Bee.hivePos`.
    pub hive_pos: AtomicCell<Option<BlockPos>>,
    /// `Bee.savedFlowerPos`.
    pub flower_pos: AtomicCell<Option<BlockPos>>,
    /// `Bee.ticksWithoutNectarSinceExitingHive`.
    ticks_without_nectar: AtomicI32,
    /// `Bee.stayOutOfHiveCountdown`.
    stay_out_of_hive_countdown: AtomicI32,
    /// `Bee.numCropsGrownSincePollination`.
    crops_grown_since_pollination: AtomicI32,
    /// `Bee.timeSinceSting`.
    time_since_sting: AtomicI32,
    /// `Bee.underWaterTicks`.
    under_water_ticks: AtomicI32,
    /// `Bee.remainingCooldownBeforeLocatingNewFlower`.
    flower_cooldown: AtomicI32,
}

impl BeeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let bee = Self {
            mob_entity,
            flags: AtomicU8::new(0),
            hive_pos: AtomicCell::new(None),
            flower_pos: AtomicCell::new(None),
            ticks_without_nectar: AtomicI32::new(0),
            stay_out_of_hive_countdown: AtomicI32::new(0),
            crops_grown_since_pollination: AtomicI32::new(0),
            time_since_sting: AtomicI32::new(0),
            under_water_ticks: AtomicI32::new(0),
            flower_cooldown: AtomicI32::new(
                rand::rng()
                    .random_range(MIN_FIND_FLOWER_RETRY_COOLDOWN..=MAX_FIND_FLOWER_RETRY_COOLDOWN),
            ),
        };
        let mob_arc = Arc::new(bee);
        let bee_weak = Arc::downgrade(&mob_arc);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(BeeAttackGoal::new(bee_weak.clone())));
            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, Box::new(BeePollinateGoal::new(bee_weak)));
            goal_selector.add_goal(3, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                4,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(5, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            // `Bee.BeeHurtByOtherGoal extends HurtByTargetGoal`. Its `setAlertOthers` alerting of
            // nearby bees and `BeeBecomeAngryTargetGoal` both need the per-player persistent
            // anger state Pumpkin does not track yet, so only the revenge part is ported.
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
        };

        mob_arc
    }

    /// `Bee.setFlag`: read-modify-write on the shared flag byte; vanilla's synched data only
    /// broadcasts when the value actually changes.
    fn set_flag(&self, flag: u8, value: bool) {
        let previous = if value {
            self.flags.fetch_or(flag, Relaxed)
        } else {
            self.flags.fetch_and(!flag, Relaxed)
        };
        let new = if value {
            previous | flag
        } else {
            previous & !flag
        };
        if previous == new {
            return;
        }
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::BEE_FLAGS,
                MetaDataType::BYTE,
                new as i8,
            )],
            None,
        );
    }

    fn get_flag(&self, flag: u8) -> bool {
        self.flags.load(Relaxed) & flag != 0
    }

    #[must_use]
    pub fn has_nectar(&self) -> bool {
        self.get_flag(FLAG_HAS_NECTAR)
    }

    /// `Bee.setHasNectar`.
    pub fn set_has_nectar(&self, has_nectar: bool) {
        if has_nectar {
            self.ticks_without_nectar.store(0, Relaxed);
        }
        self.set_flag(FLAG_HAS_NECTAR, has_nectar);
    }

    #[must_use]
    pub fn has_stung(&self) -> bool {
        self.get_flag(FLAG_HAS_STUNG)
    }

    fn set_has_stung(&self, has_stung: bool) {
        self.set_flag(FLAG_HAS_STUNG, has_stung);
    }

    #[must_use]
    pub fn is_rolling(&self) -> bool {
        self.get_flag(FLAG_ROLL)
    }

    /// `Bee.dropOffNectar`, called by the hive when a bee is released.
    pub fn drop_off_nectar(&self) {
        self.set_has_nectar(false);
        self.crops_grown_since_pollination.store(0, Relaxed);
    }

    /// `Bee.isTiredOfLookingForNectar`.
    #[must_use]
    pub fn is_tired_of_looking_for_nectar(&self) -> bool {
        is_tired_of_looking_for_nectar(self.ticks_without_nectar.load(Relaxed))
    }

    /// `Bee.setStayOutOfHiveCountdown`.
    pub fn set_stay_out_of_hive_countdown(&self, ticks: i32) {
        self.stay_out_of_hive_countdown.store(ticks, Relaxed);
    }

    /// `Bee.dropFlower`.
    fn drop_flower(&self) {
        self.flower_pos.store(None);
        self.flower_cooldown.store(
            rand::rng()
                .random_range(MIN_FIND_FLOWER_RETRY_COOLDOWN..=MAX_FIND_FLOWER_RETRY_COOLDOWN),
            Relaxed,
        );
    }

    /// `Bee.resetTicksWithoutNectarSinceExitingHive`.
    fn reset_ticks_without_nectar(&self) {
        self.ticks_without_nectar.store(0, Relaxed);
    }
}

/// `Bee.attractsBees`.
fn attracts_bees(block: &Block, state: &BlockState) -> bool {
    if !block.has_tag(&tag::Block::MINECRAFT_BEE_ATTRACTIVE) {
        return false;
    }
    if state.is_waterlogged() {
        return false;
    }
    if block.id == Block::SUNFLOWER.id {
        return TallSeagrassLikeProperties::from_state_id(state.id, block).half
            == DoubleBlockHalf::Upper;
    }
    true
}

/// `Bee.BeeAttackGoal`: a `MeleeAttackGoal` that stops once the bee has stung.
///
/// Vanilla additionally requires `isAngry()`; Pumpkin tracks no persistent anger, so the bee's
/// target (set only by `RevengeGoal`) stands in for it.
pub struct BeeAttackGoal {
    bee: Weak<BeeEntity>,
    melee: MeleeAttackGoal,
}

impl BeeAttackGoal {
    #[must_use]
    pub fn new(bee: Weak<BeeEntity>) -> Self {
        Self {
            bee,
            melee: MeleeAttackGoal::new(1.4, true),
        }
    }

    fn can_sting(&self) -> bool {
        self.bee.upgrade().is_some_and(|bee| !bee.has_stung())
    }
}

impl Goal for BeeAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.can_sting() && self.melee.can_start(mob).await })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.can_sting() && self.melee.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.melee.start(mob)
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.melee.stop(mob)
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        self.melee.tick(mob)
    }

    fn should_run_every_tick(&self) -> bool {
        self.melee.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.melee.controls()
    }
}

/// `Bee.BeePollinateGoal`.
///
/// Ported with two documented reductions, both forced by engine gaps rather than choice:
///
/// - Vanilla filters candidate flowers through `navigation.createPath(pos, 1).canReach()` and
///   remembers unreachable ones in `unreachableFlowerCache` for 600 ticks. Pumpkin's `Navigator`
///   exposes no path-reachability query, so the reachability filter and its cache are dropped;
///   the retry cooldowns (`MIN`/`MAX_FIND_FLOWER_RETRY_COOLDOWN` on failure,
///   `COOLDOWN_BEFORE_LOCATING_NEW_FLOWER` on stop) are kept and are what bound the scan cost.
/// - Vanilla hovers inside the flower with `MoveControl.setWantedPosition` at
///   `HOVER_HEIGHT_WITHIN_FLOWER`, re-jittering by `HOVER_POS_OFFSET` on a 1-in-25 roll.
///   Pumpkin's `MoveControl` only produces yaw plus forward input and cannot hold a Y target,
///   so the hover jitter is dropped; the bee stops navigating on arrival and pollinates in
///   place. The pollination timings (`MIN_POLLINATION_TICKS`, `MAX_POLLINATING_TICKS`, the
///   1-in-5 continue roll and the pollinate sound throttle) are unchanged.
pub struct BeePollinateGoal {
    bee: Weak<BeeEntity>,
    goal_control: Controls,
    successful_pollinating_ticks: i32,
    last_sound_played_tick: i32,
    pollinating: bool,
    pollinating_ticks: i32,
}

/// `BeePollinateGoal.MIN_POLLINATION_TICKS`.
const MIN_POLLINATION_TICKS: i32 = 400;
/// `BeePollinateGoal.MAX_POLLINATING_TICKS`.
const MAX_POLLINATING_TICKS: i32 = 600;
/// `BeePollinateGoal.FLOWER_SEARCH_RADIUS`.
const FLOWER_SEARCH_RADIUS: i32 = 5;
/// `BeePollinateGoal.HOVER_HEIGHT_WITHIN_FLOWER`.
const HOVER_HEIGHT_WITHIN_FLOWER: f64 = 0.6;

impl BeePollinateGoal {
    #[must_use]
    pub const fn new(bee: Weak<BeeEntity>) -> Self {
        Self {
            bee,
            goal_control: Controls::MOVE,
            successful_pollinating_ticks: 0,
            last_sound_played_tick: 0,
            pollinating: false,
            pollinating_ticks: 0,
        }
    }

    const fn has_pollinated_long_enough(&self) -> bool {
        self.successful_pollinating_ticks > MIN_POLLINATION_TICKS
    }

    /// `BeePollinateGoal.findNearbyFlower`, iterating `BlockPos.withinManhattan(pos, 5, 5, 5)`
    /// in vanilla's increasing-Manhattan-distance order so the closest flower wins.
    fn find_nearby_flower(mob: &dyn Mob) -> Option<BlockPos> {
        let origin = mob.get_entity().block_pos.load();
        let world = mob.get_entity().world.load();
        let max_depth = FLOWER_SEARCH_RADIUS * 3;

        for depth in 0..=max_depth {
            let max_x = FLOWER_SEARCH_RADIUS.min(depth);
            for x in -max_x..=max_x {
                let max_y = FLOWER_SEARCH_RADIUS.min(depth - x.abs());
                for y in -max_y..=max_y {
                    let z = depth - x.abs() - y.abs();
                    if z > FLOWER_SEARCH_RADIUS {
                        continue;
                    }
                    for z in [z, -z] {
                        let pos = BlockPos::new(origin.0.x + x, origin.0.y + y, origin.0.z + z);
                        let (block, state) = world.get_block_and_state(&pos);
                        if attracts_bees(block, state) {
                            return Some(pos);
                        }
                        if z == 0 {
                            break;
                        }
                    }
                }
            }
        }

        None
    }

    /// `Vec3.atBottomCenterOf(savedFlowerPos).add(0.0, HOVER_HEIGHT_WITHIN_FLOWER, 0.0)`.
    fn flower_target(flower_pos: BlockPos) -> Vector3<f64> {
        Vector3::new(
            f64::from(flower_pos.0.x) + 0.5,
            f64::from(flower_pos.0.y) + HOVER_HEIGHT_WITHIN_FLOWER,
            f64::from(flower_pos.0.z) + 0.5,
        )
    }

    async fn is_raining(mob: &dyn Mob) -> bool {
        mob.get_entity().world.load().weather.lock().await.raining
    }
}

impl Goal for BeePollinateGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(bee) = self.bee.upgrade() else {
                return false;
            };
            if bee.flower_cooldown.load(Relaxed) > 0 || bee.has_nectar() {
                return false;
            }
            if Self::is_raining(mob).await {
                return false;
            }

            let Some(flower_pos) = Self::find_nearby_flower(mob) else {
                bee.flower_cooldown.store(
                    mob.get_random().random_range(
                        MIN_FIND_FLOWER_RETRY_COOLDOWN..=MAX_FIND_FLOWER_RETRY_COOLDOWN,
                    ),
                    Relaxed,
                );
                return false;
            };

            bee.flower_pos.store(Some(flower_pos));
            let pos = mob.get_entity().pos.load();
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.set_progress(NavigatorGoal::new(
                pos,
                Self::flower_target(flower_pos),
                1.2,
            ));
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(bee) = self.bee.upgrade() else {
                return false;
            };
            if !self.pollinating || bee.flower_pos.load().is_none() {
                return false;
            }
            if Self::is_raining(mob).await {
                return false;
            }
            if self.has_pollinated_long_enough() {
                return mob.get_random().random::<f32>() < 0.2;
            }
            true
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.successful_pollinating_ticks = 0;
            self.pollinating_ticks = 0;
            self.last_sound_played_tick = 0;
            self.pollinating = true;
            if let Some(bee) = self.bee.upgrade() {
                bee.reset_ticks_without_nectar();
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(bee) = self.bee.upgrade() {
                if self.has_pollinated_long_enough() {
                    bee.set_has_nectar(true);
                }
                bee.flower_cooldown
                    .store(COOLDOWN_BEFORE_LOCATING_NEW_FLOWER, Relaxed);
            }
            self.pollinating = false;
            mob.get_mob_entity().navigator.lock().unwrap().stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(bee) = self.bee.upgrade() else {
                return;
            };
            let Some(flower_pos) = bee.flower_pos.load() else {
                return;
            };

            self.pollinating_ticks += 1;
            if self.pollinating_ticks > MAX_POLLINATING_TICKS {
                bee.drop_flower();
                self.pollinating = false;
                bee.flower_cooldown
                    .store(COOLDOWN_BEFORE_LOCATING_NEW_FLOWER, Relaxed);
                return;
            }

            let target = Self::flower_target(flower_pos);
            let pos = mob.get_entity().pos.load();
            if (target - pos).length() > 1.0 {
                let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
                if navigator.is_idle() {
                    navigator.set_progress(NavigatorGoal::new(pos, target, 1.2));
                }
                return;
            }

            mob.get_mob_entity().navigator.lock().unwrap().stop();
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at(mob, target.x, target.y, target.z);

            self.successful_pollinating_ticks += 1;
            if mob.get_random().random::<f32>() < 0.05
                && self.successful_pollinating_ticks > self.last_sound_played_tick + 60
            {
                self.last_sound_played_tick = self.successful_pollinating_ticks;
                let entity = mob.get_entity();
                entity.world.load().play_sound(
                    Sound::EntityBeePollinate,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

/// `BlockPos.CODEC` serializes as an `[x, y, z]` int array.
fn block_pos_to_nbt(pos: BlockPos) -> NbtTag {
    NbtTag::IntArray(vec![pos.0.x, pos.0.y, pos.0.z])
}

fn block_pos_from_nbt(nbt: &NbtCompound, name: &str) -> Option<BlockPos> {
    let &[x, y, z] = nbt.get_int_array(name)? else {
        return None;
    };
    Some(BlockPos::new(x, y, z))
}

impl NBTStorage for BeeEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            if let Some(hive_pos) = self.hive_pos.load() {
                nbt.put("hive_pos", block_pos_to_nbt(hive_pos));
            }
            if let Some(flower_pos) = self.flower_pos.load() {
                nbt.put("flower_pos", block_pos_to_nbt(flower_pos));
            }
            nbt.put_bool("HasNectar", self.has_nectar());
            nbt.put_bool("HasStung", self.has_stung());
            nbt.put_int(
                "TicksSincePollination",
                self.ticks_without_nectar.load(Relaxed),
            );
            nbt.put_int(
                "CannotEnterHiveTicks",
                self.stay_out_of_hive_countdown.load(Relaxed),
            );
            nbt.put_int(
                "CropsGrownSincePollination",
                self.crops_grown_since_pollination.load(Relaxed),
            );
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            // Store the flag bits directly: the entity has no viewers yet during load, so the
            // byte is published by `mob_init_data_tracker` instead of broadcast from here.
            let mut flags = 0u8;
            if nbt.get_bool("HasNectar").unwrap_or(false) {
                flags |= FLAG_HAS_NECTAR;
            }
            if nbt.get_bool("HasStung").unwrap_or(false) {
                flags |= FLAG_HAS_STUNG;
            }
            self.flags.store(flags, Relaxed);
            self.ticks_without_nectar
                .store(nbt.get_int("TicksSincePollination").unwrap_or(0), Relaxed);
            self.stay_out_of_hive_countdown
                .store(nbt.get_int("CannotEnterHiveTicks").unwrap_or(0), Relaxed);
            self.crops_grown_since_pollination.store(
                nbt.get_int("CropsGrownSincePollination").unwrap_or(0),
                Relaxed,
            );
            self.hive_pos.store(block_pos_from_nbt(nbt, "hive_pos"));
            self.flower_pos.store(block_pos_from_nbt(nbt, "flower_pos"));
        })
    }
}

impl Mob for BeeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::BEE_FLAGS,
                    MetaDataType::BYTE,
                    self.flags.load(Relaxed) as i8,
                )],
                None,
            );
        })
    }

    /// `Bee.doHurtTarget`: the poison, the stung flag and the sting sound are all gated on the
    /// hit actually landing, which is what `Mob::try_attack` gates this hook on.
    fn on_successful_attack<'a>(&'a self, target: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if let Some(living) = target.get_living_entity()
                && let Some(duration) =
                    poison_duration(self.get_entity().world.load().level_info.load().difficulty)
            {
                living
                    .add_effect(Effect {
                        effect_type: &StatusEffect::POISON,
                        duration,
                        amplifier: 0,
                        ambient: false,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    })
                    .await;
            }

            self.set_has_stung(true);
            // `Bee.doHurtTarget` calls `stopBeingAngry`. Pumpkin has no persistent anger state,
            // so clearing the target is the equivalent that keeps a stung bee from pursuing.
            self.set_mob_target(None).await;
            let entity = &self.mob_entity.living_entity.entity;
            entity.world.load().play_sound(
                Sound::EntityBeeSting,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
        })
    }

    /// `Bee.aiStep` and `Bee.customServerAiStep`.
    fn mob_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let living = &self.mob_entity.living_entity;
            if living.dead.load(Relaxed) {
                return;
            }

            if self.stay_out_of_hive_countdown.load(Relaxed) > 0 {
                self.stay_out_of_hive_countdown.fetch_sub(1, Relaxed);
            }

            if self.flower_cooldown.load(Relaxed) > 0 {
                self.flower_cooldown.fetch_sub(1, Relaxed);
            }

            if living.is_in_water() {
                self.under_water_ticks.fetch_add(1, Relaxed);
            } else {
                self.under_water_ticks.store(0, Relaxed);
            }

            if self.under_water_ticks.load(Relaxed) > 20 {
                caller.damage(caller.as_ref(), 1.0, DamageType::DROWN).await;
            }

            if self.has_stung() {
                let time_since_sting = self.time_since_sting.fetch_add(1, Relaxed) + 1;
                if time_since_sting % 5 == 0
                    && rand::rng().random_range(0..sting_death_roll_bound(time_since_sting)) == 0
                {
                    let health = living.health.load();
                    caller
                        .damage(caller.as_ref(), health, DamageType::GENERIC)
                        .await;
                }
            }

            if !self.has_nectar() {
                self.ticks_without_nectar.fetch_add(1, Relaxed);
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{is_tired_of_looking_for_nectar, poison_duration, sting_death_roll_bound};
    use pumpkin_util::Difficulty;

    #[test]
    fn bee_sting_poison_matches_vanilla_difficulty_durations() {
        assert_eq!(poison_duration(Difficulty::Peaceful), None);
        assert_eq!(poison_duration(Difficulty::Easy), None);
        assert_eq!(poison_duration(Difficulty::Normal), Some(200));
        assert_eq!(poison_duration(Difficulty::Hard), Some(360));
    }

    #[test]
    fn bee_sting_death_roll_bound_shrinks_and_clamps() {
        assert_eq!(sting_death_roll_bound(0), 1200);
        assert_eq!(sting_death_roll_bound(600), 600);
        assert_eq!(sting_death_roll_bound(1199), 1);
        assert_eq!(sting_death_roll_bound(1200), 1);
        assert_eq!(sting_death_roll_bound(5000), 1);
    }

    #[test]
    fn bee_attracts_bees_matches_vanilla_predicate() {
        use pumpkin_data::Block;
        use pumpkin_data::block_properties::{
            BlockProperties, DoubleBlockHalf, TallSeagrassLikeProperties,
        };
        use pumpkin_data::block_state::BlockState;

        assert!(super::attracts_bees(
            &Block::DANDELION,
            Block::DANDELION.default_state
        ));
        assert!(!super::attracts_bees(
            &Block::STONE,
            Block::STONE.default_state
        ));

        // `Bee.attractsBees` only accepts the upper half of a sunflower.
        let mut props = TallSeagrassLikeProperties::default(&Block::SUNFLOWER);
        props.half = DoubleBlockHalf::Lower;
        let lower = BlockState::from_id(props.to_state_id(&Block::SUNFLOWER));
        props.half = DoubleBlockHalf::Upper;
        let upper = BlockState::from_id(props.to_state_id(&Block::SUNFLOWER));
        assert!(!super::attracts_bees(&Block::SUNFLOWER, lower));
        assert!(super::attracts_bees(&Block::SUNFLOWER, upper));
    }

    #[test]
    fn bee_is_tired_of_looking_for_nectar_after_3600_ticks() {
        assert!(!is_tired_of_looking_for_nectar(3600));
        assert!(is_tired_of_looking_for_nectar(3601));
    }
}
