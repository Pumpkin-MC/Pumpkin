use std::ops::RangeInclusive;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering::Relaxed};

use crossbeam::atomic::AtomicCell;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use pumpkin_util::math::boundingbox::BoundingBox;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::predicate::EntityPredicate;
use crate::world::World;

/// No anger pending.
pub const NO_ANGER_END_TIME: i64 = -1;

/// Vanilla `PERSISTENT_ANGER_TIME`: 20 to 39 seconds.
pub const DEFAULT_ANGER_DURATION: RangeInclusive<i32> = 400..=780;

pub struct NeutralData {
    /// World age at which the anger runs out. `-1` means not angry.
    pub anger_end_time: AtomicI64,
    pub angry_at: AtomicCell<Option<Uuid>>,
    /// Set by NBT load: the saved grudge target is put back on the first tick in the world.
    pub restore_target: AtomicBool,
}

impl Default for NeutralData {
    fn default() -> Self {
        Self {
            anger_end_time: AtomicI64::new(NO_ANGER_END_TIME),
            angry_at: AtomicCell::new(None),
            restore_target: AtomicBool::new(false),
        }
    }
}

/// Angry while the end time still lies ahead. `NO_ANGER_END_TIME` and `0` mean calm.
#[must_use]
pub const fn is_angry_at_time(anger_end_time: i64, now: i64) -> bool {
    anger_end_time > 0 && anger_end_time - now > 0
}

/// End time from NBT: the current key, else the pre-`anger_end_time` countdown, else calm.
fn anger_end_time_from_nbt(nbt: &NbtCompound, now: i64) -> i64 {
    nbt.get_long("anger_end_time")
        .or_else(|| {
            nbt.get_int("AngerTime")
                .map(|remaining| now + i64::from(remaining))
        })
        .unwrap_or(NO_ANGER_END_TIME)
}

/// The grudge outlived its reason: timer out, and no valid player target to refresh it.
#[must_use]
pub const fn should_stop_being_angry(still_angry: bool, keeps_valid_player_target: bool) -> bool {
    !still_angry && !keeps_valid_player_target
}

/// Players live in their own list, everything else in the entity list.
#[must_use]
pub fn find_by_uuid(world: &World, uuid: Uuid) -> Option<Arc<dyn EntityBase>> {
    world
        .get_player_by_uuid(uuid)
        .map(|player| player as Arc<dyn EntityBase>)
        .or_else(|| world.get_entity_by_uuid(uuid))
}

/// A player the mob may hold a grudge against. Creative, spectator and peaceful are exempt.
fn is_valid_player_target(target: &dyn EntityBase, world: &World) -> bool {
    target.get_player().is_some_and(|player| {
        !player.is_creative()
            && !player.is_spectator()
            && world.level_info.load().difficulty != Difficulty::Peaceful
    })
}

/// a timed grudge plus the entity it is held against.
///
/// Not compiler-enforced: every implementor must also override `Mob::as_neutral`.
/// Without it the tick hook, NBT and `TargetCondition::AngryAt` silently skip the mob.
pub trait NeutralMob: Mob {
    fn get_neutral_data(&self) -> &NeutralData;

    /// Rolled fresh every time the anger restarts.
    fn anger_duration(&self) -> RangeInclusive<i32> {
        DEFAULT_ANGER_DURATION
    }

    /// Whether an already angry mob keeps refreshing its timer while the target stays.
    fn stays_angry_with_target(&self) -> bool {
        true
    }

    fn world_age(&self) -> i64 {
        self.get_entity()
            .world
            .load()
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .world_age
    }

    fn get_anger_end_time(&self) -> i64 {
        self.get_neutral_data().anger_end_time.load(Relaxed)
    }

    fn set_anger_end_time(&self, end_time: i64) {
        self.get_neutral_data()
            .anger_end_time
            .store(end_time, Relaxed);
        // TODO: sync DATA_ANGER_END_TIME so the client shows the angry pose.
    }

    fn set_time_to_remain_angry(&self, ticks: i32) {
        self.set_anger_end_time(self.world_age() + i64::from(ticks));
    }

    fn start_persistent_anger_timer(&self) {
        self.start_persistent_anger_timer_at(self.world_age());
    }

    /// `start_persistent_anger_timer` with a world age the caller already read.
    fn start_persistent_anger_timer_at(&self, now: i64) {
        let ticks = rand::random_range(self.anger_duration());
        self.set_anger_end_time(now + i64::from(ticks));
    }

    fn is_angry(&self) -> bool {
        // Calm mobs skip the level-time lock.
        self.get_anger_end_time() > 0 && self.is_angry_at_age(self.world_age())
    }

    /// `is_angry` against a world age the caller already read.
    fn is_angry_at_age(&self, now: i64) -> bool {
        is_angry_at_time(self.get_anger_end_time(), now)
    }

    fn get_persistent_anger_target(&self) -> Option<Uuid> {
        self.get_neutral_data().angry_at.load()
    }

    fn set_persistent_anger_target(&self, target: Option<Uuid>) {
        self.get_neutral_data().angry_at.store(target);
    }

    fn stop_being_angry(&self) {
        // keeps the revenge goal from re-triggering.
        self.get_mob_entity()
            .living_entity
            .last_attacker_id
            .store(0, Relaxed);
        self.set_persistent_anger_target(None);
        self.set_mob_target(None);
        self.set_anger_end_time(NO_ANGER_END_TIME);
    }

    /// `universal_anger` turns an unfocused grudge against every player.
    fn is_angry_at_all_players(&self, world: &World) -> bool {
        world.level_info.load().game_rules.universal_anger
            && self.get_persistent_anger_target().is_none()
            && self.is_angry()
    }

    fn is_angry_at(&self, target: &dyn EntityBase, world: &World) -> bool {
        let Some(living) = target.get_living_entity() else {
            return false;
        };
        if !self.can_attack(living) {
            return false;
        }
        if is_valid_player_target(target, world) && self.is_angry_at_all_players(world) {
            return true;
        }
        self.get_persistent_anger_target()
            .is_some_and(|angry_at| angry_at == target.get_entity().entity_uuid)
    }

    fn forget_current_target_and_refresh_universal_anger(&self) {
        self.stop_being_angry();
        self.start_persistent_anger_timer();
    }

    /// Forgiven once the grudge holder dies, when the gamerule allows it.
    fn player_died(&self, player: &dyn EntityBase, world: &World) {
        if !world.level_info.load().game_rules.forgive_dead_players {
            return;
        }
        if self
            .get_persistent_anger_target()
            .is_some_and(|angry_at| angry_at == player.get_entity().entity_uuid)
        {
            self.stop_being_angry();
        }
    }

    /// Puts the saved grudge target back as the mob's target. Runs once after NBT load.
    fn restore_persistent_target(&self, world: &World) {
        let data = self.get_neutral_data();
        if !data.restore_target.load(Relaxed) {
            return;
        }
        data.restore_target.store(false, Relaxed);

        let Some(uuid) = self.get_persistent_anger_target() else {
            return;
        };
        if let Some(target) = find_by_uuid(world, uuid)
            && target.get_entity().is_alive()
            && target
                .get_living_entity()
                .is_some_and(|living| self.can_attack(living))
        {
            self.set_mob_target(Some(target));
        }
    }

    /// Ticked for every neutral mob -> adopts a new target as the grudge expires.
    fn update_persistent_anger(&self) {
        let world = self.get_entity().world.load();
        self.restore_persistent_target(&world);
        let target = self.get_mob_entity().get_target();
        let anger_target = self.get_persistent_anger_target();

        // Calm and unprovoked: nothing to track, nothing to lock.
        if target.is_none() && anger_target.is_none() {
            return;
        }
        let now = self.world_age();

        // Grudge holder killed by someone else
        if let Some(previous) = &target
            && !previous.get_entity().is_alive()
            && previous.get_mob().is_some()
            && anger_target.is_some_and(|angry_at| angry_at == previous.get_entity().entity_uuid)
        {
            self.stop_being_angry();
            return;
        }

        let stays_angry = self.stays_angry_with_target();

        if let Some(target) = &target {
            let is_new =
                anger_target.is_none_or(|angry_at| angry_at != target.get_entity().entity_uuid);
            if is_new {
                self.set_persistent_anger_target(Some(target.get_entity().entity_uuid));
            }
            if is_new || stays_angry {
                self.start_persistent_anger_timer_at(now);
            }
        }

        let keeps_valid_player_target = stays_angry
            && target
                .as_ref()
                .is_some_and(|target| is_valid_player_target(target.as_ref(), &world));
        if anger_target.is_some()
            && should_stop_being_angry(self.is_angry_at_age(now), keeps_valid_player_target)
        {
            self.stop_being_angry();
            return;
        }

        // Grudge target switched to creative or spectator
        if let Some(angry_at) = anger_target
            && let Some(player) = world.get_player_by_uuid(angry_at)
            && (player.is_creative()
                || player.is_spectator()
                || world.level_info.load().difficulty == Difficulty::Peaceful)
        {
            self.stop_being_angry();
        }
    }

    fn write_anger_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_long("anger_end_time", self.get_anger_end_time());
        if let Some(angry_at) = self.get_persistent_anger_target() {
            nbt.put_uuid("angry_at", angry_at);
        }
    }

    fn read_anger_nbt(&self, nbt: &NbtCompound) {
        self.set_anger_end_time(anger_end_time_from_nbt(nbt, self.world_age()));
        let angry_at = nbt.get_uuid("angry_at");
        self.get_neutral_data()
            .restore_target
            .store(angry_at.is_some(), Relaxed);
        self.set_persistent_anger_target(angry_at);
    }
}

/// Vanilla `tellNeutralMobsThatIDied` range.
const DEATH_NOTICE_RANGE_XZ: f64 = 32.0;
const DEATH_NOTICE_RANGE_Y: f64 = 10.0;

/// Neutral mobs near a dead player drop the grudge, when `forgive_dead_players` is on.
pub fn tell_neutral_mobs_player_died(player: &dyn EntityBase, world: &World) {
    if !world.level_info.load().game_rules.forgive_dead_players {
        return;
    }

    let pos = player.get_entity().block_pos.load().to_f64();
    let search_box = BoundingBox::new(pos, pos.add_raw(1.0, 1.0, 1.0)).expand(
        DEATH_NOTICE_RANGE_XZ,
        DEATH_NOTICE_RANGE_Y,
        DEATH_NOTICE_RANGE_XZ,
    );
    for other in world.get_entities_at_box(&search_box) {
        if !EntityPredicate::ExceptSpectator.test(other.get_entity()) {
            continue;
        }
        if let Some(neutral) = other.get_mob().and_then(Mob::as_neutral) {
            neutral.player_died(player, world);
        }
    }
}

/// Generates the `Mob` glue every neutral mob repeats.
#[macro_export]
macro_rules! impl_neutral_mob {
    ($mob:ty, $field:ident) => {
        $crate::impl_neutral_mob!(
            $mob,
            $field,
            $crate::entity::mob::neutral::DEFAULT_ANGER_DURATION
        );
    };
    ($mob:ty, $field:ident, $duration:expr) => {
        impl $crate::entity::mob::neutral::NeutralMob for $mob {
            fn get_neutral_data(&self) -> &$crate::entity::mob::neutral::NeutralData {
                &self.$field
            }

            fn anger_duration(&self) -> std::ops::RangeInclusive<i32> {
                $duration
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_ANGER_DURATION, NO_ANGER_END_TIME, NeutralData, anger_end_time_from_nbt,
        is_angry_at_time, should_stop_being_angry,
    };
    use pumpkin_nbt::compound::NbtCompound;
    use std::sync::atomic::Ordering::Relaxed;

    #[test]
    fn fresh_data_is_calm() {
        let data = NeutralData::default();
        assert_eq!(data.anger_end_time.load(Relaxed), NO_ANGER_END_TIME);
        assert!(data.angry_at.load().is_none());
        assert!(!is_angry_at_time(NO_ANGER_END_TIME, 0));
    }

    #[test]
    fn anger_expires_on_the_end_tick() {
        let now = 1_000;
        let end_time = now + 5;
        assert!(is_angry_at_time(end_time, now));
        assert!(is_angry_at_time(end_time, end_time - 1));
        // The end tick itself is already calm.
        assert!(!is_angry_at_time(end_time, end_time));
        assert!(!is_angry_at_time(end_time, end_time + 1));
    }

    #[test]
    fn zero_end_time_is_never_angry() {
        // Legacy worlds can store 0; it must not read as "angry forever".
        assert!(!is_angry_at_time(0, -10));
        assert!(!is_angry_at_time(0, 0));
    }

    #[test]
    fn default_duration_matches_vanilla_seconds() {
        // PERSISTENT_ANGER_TIME: 20 to 39 seconds at 20 ticks per second.
        // Both ends inclusive, like `UniformInt`.
        assert_eq!(*DEFAULT_ANGER_DURATION.start(), 20 * 20);
        assert_eq!(*DEFAULT_ANGER_DURATION.end(), 39 * 20);
    }

    #[test]
    fn running_timer_never_drops_the_grudge() {
        for keeps in [false, true] {
            assert!(!should_stop_being_angry(true, keeps));
        }
    }

    #[test]
    fn expired_grudge_is_dropped_without_a_valid_player_target() {
        // Bee: never keeps a player target alive, so an expired timer ends it.
        assert!(should_stop_being_angry(false, false));
    }

    #[test]
    fn expired_grudge_survives_while_a_valid_player_target_refreshes_it() {
        // Piglin, wolf and friends restart the timer while the target stays.
        assert!(!should_stop_being_angry(false, true));
    }

    #[test]
    fn nbt_current_key_wins_over_legacy_countdown() {
        let mut nbt = NbtCompound::new();
        nbt.put_long("anger_end_time", 5_000);
        nbt.put_int("AngerTime", 100);
        assert_eq!(anger_end_time_from_nbt(&nbt, 1_000), 5_000);
    }

    #[test]
    fn nbt_legacy_countdown_becomes_an_absolute_end_time() {
        let mut nbt = NbtCompound::new();
        nbt.put_int("AngerTime", 100);
        assert_eq!(anger_end_time_from_nbt(&nbt, 1_000), 1_100);
    }

    #[test]
    fn nbt_without_anger_is_calm() {
        assert_eq!(
            anger_end_time_from_nbt(&NbtCompound::new(), 1_000),
            NO_ANGER_END_TIME
        );
    }
}
