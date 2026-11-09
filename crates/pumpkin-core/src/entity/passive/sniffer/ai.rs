use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::Activity;

use super::{SnifferEntity, SnifferState};
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainTick,
    activity::ActivityData,
    behavior::{
        AnimalMakeLove, AnimalPanic, Behavior, CountDownCooldownTicks, DoNothing, FollowTemptation,
        LookAtTargetSink, MoveToTargetSink, Swim, Timed, interact::set_entity_look_target,
        random_stroll, run_one, set_walk_target_from_look_target,
    },
    memory::{MemoryModuleId, MemoryStatus, PackedMemories, types, walk_target::WalkTarget},
    sensing::SensorType,
};
use crate::entity::{ageable::AgeableMob, mob::Mob};

const MEMORIES: &[MemoryModuleId] = &[types::SNIFFER_EXPLORED_POSITIONS.id()];

const MAX_LOOK_DISTANCE: f32 = 6.0;
const SNIFFING_COOLDOWN_TICKS: i64 = 9600;
const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 1.0;
const SPEED_MULTIPLIER_WHEN_PANICKING: f32 = 2.0;
const SPEED_MULTIPLIER_WHEN_SNIFFING: f32 = 1.25;
const SPEED_MULTIPLIER_WHEN_TEMPTED: f32 = 1.25;

pub(super) fn make_brain(sniffer: &SnifferEntity, packed: &PackedMemories) -> Brain {
    BrainProvider {
        memory_types: MEMORIES,
        sensor_types: &[
            SensorType::NearestLivingEntities,
            SensorType::HurtBy,
            SensorType::NearestPlayers,
            SensorType::FoodTemptations,
        ],
        activities,
    }
    .make_brain(sniffer, packed)
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(Timed::new(Swim::new(0.8))),
                Box::new(Timed::new(ResetSniffing(AnimalPanic::new(
                    SPEED_MULTIPLIER_WHEN_PANICKING,
                )))),
                Box::new(Timed::new(MoveToTargetSink::new(500, 700))),
                Box::new(CountDownCooldownTicks::new(
                    types::TEMPTATION_COOLDOWN_TICKS,
                )),
            ],
        ),
        ActivityData::with_pairs_and_conditions(
            Activity::Idle,
            vec![
                (
                    0,
                    Box::new(Timed::new(ResetSniffing(AnimalMakeLove::new(
                        &EntityType::SNIFFER,
                    )))),
                ),
                (
                    1,
                    Box::new(Timed::new(ResetSniffing(FollowTemptation::new(
                        |_| SPEED_MULTIPLIER_WHEN_TEMPTED,
                        |mob| {
                            if mob.get_mob_entity().living_entity.is_baby() {
                                2.5
                            } else {
                                3.5
                            }
                        },
                    )))),
                ),
                (2, Box::new(Timed::new(LookAtTargetSink::new(45, 90)))),
                (3, Box::new(Timed::new(FeelingHappy))),
                (
                    4,
                    Box::new(run_one(vec![
                        (
                            Box::new(set_walk_target_from_look_target::create(
                                SPEED_MULTIPLIER_WHEN_IDLING,
                                3,
                            )),
                            2,
                        ),
                        (Box::new(Timed::new(Scenting)), 1),
                        (Box::new(Timed::new(Sniffing)), 1),
                        (
                            Box::new(set_entity_look_target::for_type(
                                &EntityType::PLAYER,
                                MAX_LOOK_DISTANCE,
                            )),
                            1,
                        ),
                        (
                            Box::new(random_stroll::stroll(SPEED_MULTIPLIER_WHEN_IDLING)),
                            1,
                        ),
                        (Box::new(DoNothing::new(5, 20)), 2),
                    ])),
                ),
            ],
            vec![(types::SNIFFER_DIGGING.id(), MemoryStatus::ValueAbsent)],
        ),
        ActivityData::with_pairs_and_conditions(
            Activity::Sniff,
            vec![(0, Box::new(Timed::new(Searching)))],
            vec![
                (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
                (
                    types::SNIFFER_SNIFFING_TARGET.id(),
                    MemoryStatus::ValuePresent,
                ),
                (types::WALK_TARGET.id(), MemoryStatus::ValuePresent),
            ],
        ),
        ActivityData::with_pairs_and_conditions(
            Activity::Dig,
            vec![
                (0, Box::new(Timed::new(Digging))),
                (0, Box::new(Timed::new(FinishedDigging))),
            ],
            vec![
                (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
                (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
                (types::SNIFFER_DIGGING.id(), MemoryStatus::ValuePresent),
            ],
        ),
    ]
}

fn sniffer<'a>(tick: &BrainTick<'a>) -> Option<&'a SnifferEntity> {
    tick.mob.cast_any().downcast_ref::<SnifferEntity>()
}

fn reset_sniffing(tick: &mut BrainTick<'_>) {
    tick.state.erase(types::SNIFFER_DIGGING.id());
    tick.state.erase(types::SNIFFER_SNIFFING_TARGET.id());
    if let Some(sniffer) = sniffer(tick) {
        sniffer.transition_to(SnifferState::Idling);
    }
}

struct ResetSniffing<B>(B);

impl<B: Behavior> Behavior for ResetSniffing<B> {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        self.0.entry_conditions()
    }
    fn min_duration(&self) -> i32 {
        self.0.min_duration()
    }
    fn max_duration(&self) -> i32 {
        self.0.max_duration()
    }
    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.0.check_extra_start_conditions(tick)
    }
    fn start(&mut self, tick: &mut BrainTick<'_>) {
        reset_sniffing(tick);
        self.0.start(tick);
    }
    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.0.can_still_use(tick)
    }
    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        self.0.tick(tick);
    }
    fn stop_with_timeout(&mut self, tick: &mut BrainTick<'_>, timed_out: bool) {
        self.0.stop_with_timeout(tick, timed_out);
    }
    fn debug_name(&self) -> &'static str {
        self.0.debug_name()
    }
}

struct Digging;

impl Behavior for Digging {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const ENTRY_CONDITIONS: &[(MemoryModuleId, MemoryStatus)] = &[
            (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::SNIFFER_DIGGING.id(), MemoryStatus::ValuePresent),
            (types::SNIFF_COOLDOWN.id(), MemoryStatus::ValueAbsent),
        ];
        ENTRY_CONDITIONS
    }
    fn min_duration(&self) -> i32 {
        160
    }
    fn max_duration(&self) -> i32 {
        180
    }
    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        sniffer(tick).is_some_and(|sniffer| sniffer.can_sniff(tick.state))
    }
    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.state.has_memory_value(types::SNIFFER_DIGGING.id())
            && sniffer(tick).is_some_and(|sniffer| {
                sniffer.can_dig(tick.state) && !sniffer.mob_entity.is_in_love()
            })
    }
    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Digging);
        }
    }
    fn stop_with_timeout(&mut self, tick: &mut BrainTick<'_>, timed_out: bool) {
        if timed_out {
            tick.state
                .set_with_expiry(types::SNIFF_COOLDOWN, (), SNIFFING_COOLDOWN_TICKS);
        } else {
            reset_sniffing(tick);
        }
    }
    fn debug_name(&self) -> &'static str {
        "Digging"
    }
}

struct FinishedDigging;

impl Behavior for FinishedDigging {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const ENTRY_CONDITIONS: &[(MemoryModuleId, MemoryStatus)] = &[
            (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::SNIFFER_DIGGING.id(), MemoryStatus::ValuePresent),
            (types::SNIFF_COOLDOWN.id(), MemoryStatus::ValuePresent),
        ];
        ENTRY_CONDITIONS
    }
    fn min_duration(&self) -> i32 {
        40
    }
    fn max_duration(&self) -> i32 {
        40
    }
    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.state.has_memory_value(types::SNIFFER_DIGGING.id())
    }
    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Rising);
        }
    }
    fn stop_with_timeout(&mut self, tick: &mut BrainTick<'_>, timed_out: bool) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Idling);
            sniffer.on_digging_complete(tick.state, timed_out);
        }
        tick.state.erase(types::SNIFFER_DIGGING.id());
        tick.state.set(types::SNIFFER_HAPPY, true);
    }
    fn debug_name(&self) -> &'static str {
        "FinishedDigging"
    }
}

struct FeelingHappy;

impl Behavior for FeelingHappy {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const ENTRY_CONDITIONS: &[(MemoryModuleId, MemoryStatus)] =
            &[(types::SNIFFER_HAPPY.id(), MemoryStatus::ValuePresent)];
        ENTRY_CONDITIONS
    }
    fn min_duration(&self) -> i32 {
        40
    }
    fn max_duration(&self) -> i32 {
        100
    }
    fn can_still_use(&mut self, _tick: &mut BrainTick<'_>) -> bool {
        true
    }
    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::FeelingHappy);
        }
    }
    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Idling);
        }
        tick.state.erase(types::SNIFFER_HAPPY.id());
    }
    fn debug_name(&self) -> &'static str {
        "FeelingHappy"
    }
}

struct Scenting;

impl Behavior for Scenting {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const ENTRY_CONDITIONS: &[(MemoryModuleId, MemoryStatus)] = &[
            (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
            (types::SNIFFER_DIGGING.id(), MemoryStatus::ValueAbsent),
            (
                types::SNIFFER_SNIFFING_TARGET.id(),
                MemoryStatus::ValueAbsent,
            ),
            (types::SNIFFER_HAPPY.id(), MemoryStatus::ValueAbsent),
            (types::BREED_TARGET.id(), MemoryStatus::ValueAbsent),
        ];
        ENTRY_CONDITIONS
    }
    fn min_duration(&self) -> i32 {
        40
    }
    fn max_duration(&self) -> i32 {
        80
    }
    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        !tick.state.has_memory_value(types::TEMPTING_PLAYER.id())
    }
    fn can_still_use(&mut self, _tick: &mut BrainTick<'_>) -> bool {
        true
    }
    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Scenting);
        }
    }
    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Idling);
        }
    }
    fn debug_name(&self) -> &'static str {
        "Scenting"
    }
}

struct Searching;

impl Behavior for Searching {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const ENTRY_CONDITIONS: &[(MemoryModuleId, MemoryStatus)] = &[
            (types::WALK_TARGET.id(), MemoryStatus::ValuePresent),
            (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
            (
                types::SNIFFER_SNIFFING_TARGET.id(),
                MemoryStatus::ValuePresent,
            ),
        ];
        ENTRY_CONDITIONS
    }
    fn min_duration(&self) -> i32 {
        600
    }
    fn max_duration(&self) -> i32 {
        600
    }
    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        sniffer(tick).is_some_and(|sniffer| sniffer.can_sniff(tick.state))
    }
    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if let Some(sniffer) = sniffer(tick) {
            if !sniffer.can_sniff(tick.state) {
                sniffer.transition_to(SnifferState::Idling);
                return false;
            }
            return tick
                .state
                .get(types::WALK_TARGET)
                .zip(tick.state.get(types::SNIFFER_SNIFFING_TARGET))
                .is_some_and(|(walk, sniff)| walk.target().current_block_position() == *sniff);
        }
        false
    }
    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Searching);
        }
    }
    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        if sniffer(tick)
            .is_some_and(|sniffer| sniffer.can_dig(tick.state) && sniffer.can_sniff(tick.state))
        {
            tick.state.set(types::SNIFFER_DIGGING, true);
        }
        tick.state.erase(types::WALK_TARGET.id());
        tick.state.erase(types::SNIFFER_SNIFFING_TARGET.id());
    }
    fn debug_name(&self) -> &'static str {
        "Searching"
    }
}

struct Sniffing;

impl Behavior for Sniffing {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const ENTRY_CONDITIONS: &[(MemoryModuleId, MemoryStatus)] = &[
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (
                types::SNIFFER_SNIFFING_TARGET.id(),
                MemoryStatus::ValueAbsent,
            ),
            (types::SNIFF_COOLDOWN.id(), MemoryStatus::ValueAbsent),
        ];
        ENTRY_CONDITIONS
    }
    fn min_duration(&self) -> i32 {
        40
    }
    fn max_duration(&self) -> i32 {
        80
    }
    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        sniffer(tick).is_some_and(|sniffer| !sniffer.is_baby() && sniffer.can_sniff(tick.state))
    }
    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        sniffer(tick).is_some_and(|sniffer| sniffer.can_sniff(tick.state))
    }
    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Sniffing);
        }
    }
    fn stop_with_timeout(&mut self, tick: &mut BrainTick<'_>, timed_out: bool) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Idling);
            if timed_out && let Some(pos) = sniffer.calculate_dig_position(tick.state) {
                tick.state.set(types::SNIFFER_SNIFFING_TARGET, pos);
                tick.state.set(
                    types::WALK_TARGET,
                    WalkTarget::from_block_pos(pos, SPEED_MULTIPLIER_WHEN_SNIFFING, 0),
                );
            }
        }
    }
    fn debug_name(&self) -> &'static str {
        "Sniffing"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{
        Entity, EntityBase,
        ai::{brain::behavior::BehaviorControl, test_support::world},
    };
    use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
    use std::sync::atomic::Ordering::Relaxed;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn interrupted_digs_do_not_cool_down_and_only_completed_rises_remember_the_site()
    -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let sniffer = SnifferEntity::new(Entity::new(
            world.clone(),
            Vector3::new(8.5, 64.0, 8.5),
            &EntityType::SNIFFER,
        ));
        sniffer.get_entity().on_ground.store(true, Relaxed);
        sniffer
            .get_entity()
            .supporting_block_pos
            .store(Some(BlockPos::new(8, 63, 8)));
        let mut brain = make_brain(&sniffer, &PackedMemories::empty());
        brain.set(types::SNIFFER_DIGGING, true);
        let mut tick = BrainTick {
            state: &mut brain,
            world: &world,
            mob: sniffer.as_ref(),
            time: 100,
        };
        let mut digging = Timed::new(Digging);
        assert!(digging.try_start(&mut tick));
        assert_eq!(sniffer.get_state(), SnifferState::Digging);
        tick.time = 101;
        digging.do_stop(&mut tick);
        assert_eq!(sniffer.get_state(), SnifferState::Idling);
        assert!(!tick.state.has_memory_value(types::SNIFFER_DIGGING.id()));
        assert!(!tick.state.has_memory_value(types::SNIFF_COOLDOWN.id()));

        tick.state.set(types::SNIFFER_DIGGING, true);
        tick.time = 200;
        assert!(digging.try_start(&mut tick));
        tick.time = 381;
        digging.tick_or_stop(&mut tick);
        assert!(tick.state.has_memory_value(types::SNIFF_COOLDOWN.id()));
        let mut rising = Timed::new(FinishedDigging);
        assert!(rising.try_start(&mut tick));
        assert_eq!(sniffer.get_state(), SnifferState::Rising);
        tick.time = 421;
        rising.tick_or_stop(&mut tick);
        assert_eq!(sniffer.get_state(), SnifferState::Rising);
        tick.time = 422;
        rising.tick_or_stop(&mut tick);
        assert_eq!(sniffer.get_state(), SnifferState::Idling);
        assert!(tick.state.has_memory_value(types::SNIFFER_HAPPY.id()));
        let explored = tick
            .state
            .get(types::SNIFFER_EXPLORED_POSITIONS)
            .ok_or("missing dig memory")?;
        assert_eq!(explored[0].pos, BlockPos::new(8, 63, 8));
        assert!(!tick.state.has_memory_value(types::SNIFFER_DIGGING.id()));

        tick.state.set(types::SNIFFER_DIGGING, true);
        tick.time = 500;
        assert!(rising.try_start(&mut tick));
        tick.time = 501;
        rising.do_stop(&mut tick);
        assert_eq!(
            tick.state
                .get(types::SNIFFER_EXPLORED_POSITIONS)
                .map(Vec::len),
            Some(1)
        );
        world.level.shutdown().await;
        Ok(())
    }
}
