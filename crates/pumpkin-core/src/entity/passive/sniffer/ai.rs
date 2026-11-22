use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::Activity;
use pumpkin_util::math::position::BlockPos;

use super::{SnifferEntity, SnifferState};
use crate::entity::EntityBase;
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainState, BrainTick,
    activity::ActivityData,
    behavior::{
        AnimalMakeLove, AnimalPanic, Behavior, CountDownCooldownTicks, DoNothing, FollowTemptation,
        LookAtTargetSink, MoveToTargetSink, Swim, Timed, interact::set_entity_look_target,
        random_stroll, run_one, set_walk_target_from_look_target,
    },
    memory::{
        GlobalPos, MemoryModuleId, MemoryStatus, PackedMemories, types, walk_target::WalkTarget,
    },
    sensing::SensorType,
};
use crate::entity::mob::Mob;

pub const MAX_LOOK_DISTANCE: f32 = 6.0;
pub const SNIFFING_COOLDOWN_TICKS: i32 = 9600;
pub const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 1.0;
pub const SPEED_MULTIPLIER_WHEN_PANICKING: f32 = 2.0;
pub const SPEED_MULTIPLIER_WHEN_SNIFFING: f32 = 1.25;
pub const SPEED_MULTIPLIER_WHEN_TEMPTED: f32 = 1.25;

const MEMORIES: [MemoryModuleId; 1] = [types::SNIFFER_EXPLORED_POSITIONS.id()];

pub(super) fn make_brain(sniffer: &SnifferEntity, packed: &PackedMemories) -> Brain {
    BrainProvider {
        memory_types: &MEMORIES,
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

pub(super) fn update_activity(brain: &mut BrainState) {
    brain.set_active_activity_to_first_valid(&[Activity::Dig, Activity::Sniff, Activity::Idle]);
}

pub(super) fn reset_sniffing(state: &mut BrainState, sniffer: &SnifferEntity) {
    state.erase(types::SNIFFER_DIGGING.id());
    state.erase(types::SNIFFER_SNIFFING_TARGET.id());
    sniffer.transition_to(SnifferState::Idling);
}

pub(super) fn store_explored_position(
    state: &mut BrainState,
    sniffer: &SnifferEntity,
    pos: BlockPos,
) {
    let world = sniffer.get_entity().world.load();
    let Some(dimension) = Dimension::from_name(world.dimension.minecraft_name) else {
        return;
    };
    let global_pos = GlobalPos::new(dimension, pos);
    let mut updated: Vec<GlobalPos> = state
        .get(types::SNIFFER_EXPLORED_POSITIONS)
        .map(|list| list.iter().take(20).copied().collect())
        .unwrap_or_default();
    updated.insert(0, global_pos);
    state.set(types::SNIFFER_EXPLORED_POSITIONS, updated);
}

fn sniffer<'a>(tick: &BrainTick<'a>) -> Option<&'a SnifferEntity> {
    tick.mob.cast_any().downcast_ref::<SnifferEntity>()
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        core_activity(),
        idle_activity(),
        sniff_activity(),
        dig_activity(),
    ]
}

fn core_activity() -> ActivityData {
    ActivityData::with_priority_start(
        Activity::Core,
        0,
        vec![
            Box::new(Timed::new(Swim::new(0.8))),
            Box::new(Timed::new(ResetSniffingBefore(AnimalPanic::new(
                SPEED_MULTIPLIER_WHEN_PANICKING,
            )))),
            Box::new(Timed::new(MoveToTargetSink::new(500, 700))),
            Box::new(CountDownCooldownTicks::new(
                types::TEMPTATION_COOLDOWN_TICKS,
            )),
        ],
    )
}

fn idle_activity() -> ActivityData {
    ActivityData::with_pairs_and_conditions(
        Activity::Idle,
        vec![
            (
                0,
                Box::new(Timed::new(ResetSniffingBefore(AnimalMakeLove::new(
                    &EntityType::SNIFFER,
                )))),
            ),
            (
                1,
                Box::new(Timed::new(ResetSniffingBefore(FollowTemptation::new(
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
            (3, Box::new(Timed::new(FeelingHappy::new(40, 100)))),
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
                    (Box::new(Timed::new(Scenting::new(40, 80))), 1),
                    (Box::new(Timed::new(Sniffing::new(40, 80))), 1),
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
    )
}

fn sniff_activity() -> ActivityData {
    ActivityData::with_pairs_and_conditions(
        Activity::Sniff,
        vec![(0, Box::new(Timed::new(Searching::new(600))))],
        vec![
            (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
            (
                types::SNIFFER_SNIFFING_TARGET.id(),
                MemoryStatus::ValuePresent,
            ),
            (types::WALK_TARGET.id(), MemoryStatus::ValuePresent),
        ],
    )
}

fn dig_activity() -> ActivityData {
    ActivityData::with_pairs_and_conditions(
        Activity::Dig,
        vec![
            (0, Box::new(Timed::new(Digging::new(160, 180)))),
            (0, Box::new(Timed::new(FinishedDigging::new(40)))),
        ],
        vec![
            (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::SNIFFER_DIGGING.id(), MemoryStatus::ValuePresent),
        ],
    )
}

struct ResetSniffingBefore<B>(B);

impl<B: Behavior> Behavior for ResetSniffingBefore<B> {
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
        if let Some(sniffer) = sniffer(tick) {
            reset_sniffing(tick.state, sniffer);
        }
        self.0.start(tick);
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        self.0.tick(tick);
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>, timed_out: bool) {
        self.0.stop(tick, timed_out);
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.0.can_still_use(tick)
    }

    fn debug_name(&self) -> &'static str {
        self.0.debug_name()
    }
}

struct Sniffing {
    conditions: [(MemoryModuleId, MemoryStatus); 3],
    min: i32,
    max: i32,
}

impl Sniffing {
    const fn new(min: i32, max: i32) -> Self {
        Self {
            conditions: [
                (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
                (
                    types::SNIFFER_SNIFFING_TARGET.id(),
                    MemoryStatus::ValueAbsent,
                ),
                (types::SNIFF_COOLDOWN.id(), MemoryStatus::ValueAbsent),
            ],
            min,
            max,
        }
    }
}

impl Behavior for Sniffing {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &self.conditions
    }

    fn min_duration(&self) -> i32 {
        self.min
    }

    fn max_duration(&self) -> i32 {
        self.max
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        sniffer(tick).is_some_and(|s| {
            !s.mob_entity.living_entity.is_baby() && s.can_sniff_with_brain(tick.state)
        })
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        sniffer(tick).is_some_and(|s| s.can_sniff_with_brain(tick.state))
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Sniffing);
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>, timed_out: bool) {
        let Some(sniffer) = sniffer(tick) else { return };
        sniffer.transition_to(SnifferState::Idling);
        if timed_out && let Some(position) = sniffer.calculate_dig_position(tick.state) {
            tick.state.set(types::SNIFFER_SNIFFING_TARGET, position);
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::from_block_pos(position, SPEED_MULTIPLIER_WHEN_SNIFFING, 0),
            );
        }
    }

    fn debug_name(&self) -> &'static str {
        "Sniffing"
    }
}

struct Searching {
    conditions: [(MemoryModuleId, MemoryStatus); 3],
    duration: i32,
}

impl Searching {
    const fn new(duration: i32) -> Self {
        Self {
            conditions: [
                (types::WALK_TARGET.id(), MemoryStatus::ValuePresent),
                (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
                (
                    types::SNIFFER_SNIFFING_TARGET.id(),
                    MemoryStatus::ValuePresent,
                ),
            ],
            duration,
        }
    }
}

impl Behavior for Searching {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &self.conditions
    }

    fn min_duration(&self) -> i32 {
        self.duration
    }

    fn max_duration(&self) -> i32 {
        self.duration
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        sniffer(tick).is_some_and(|s| s.can_sniff_with_brain(tick.state))
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some(sniffer) = sniffer(tick) else {
            return false;
        };
        if !sniffer.can_sniff_with_brain(tick.state) {
            sniffer.transition_to(SnifferState::Idling);
            return false;
        }

        let walk_target = tick
            .state
            .get(types::WALK_TARGET)
            .map(|target| target.target().current_block_position());
        let sniffing_target = tick.state.get(types::SNIFFER_SNIFFING_TARGET).copied();

        match (walk_target, sniffing_target) {
            (Some(walk), Some(sniff)) => walk == sniff,
            _ => false,
        }
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Searching);
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>, _timed_out: bool) {
        if let Some(sniffer) = sniffer(tick)
            && sniffer.can_dig_with_brain(tick.state)
            && sniffer.can_sniff_with_brain(tick.state)
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

struct Digging {
    conditions: [(MemoryModuleId, MemoryStatus); 4],
    min: i32,
    max: i32,
}

impl Digging {
    const fn new(min: i32, max: i32) -> Self {
        Self {
            conditions: [
                (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
                (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
                (types::SNIFFER_DIGGING.id(), MemoryStatus::ValuePresent),
                (types::SNIFF_COOLDOWN.id(), MemoryStatus::ValueAbsent),
            ],
            min,
            max,
        }
    }
}

impl Behavior for Digging {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &self.conditions
    }

    fn min_duration(&self) -> i32 {
        self.min
    }

    fn max_duration(&self) -> i32 {
        self.max
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        sniffer(tick).is_some_and(|s| s.can_sniff_with_brain(tick.state))
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        sniffer(tick).is_some_and(|s| {
            tick.state.get(types::SNIFFER_DIGGING).is_some()
                && s.can_dig_with_brain(tick.state)
                && !s.mob_entity.is_in_love()
        })
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Digging);
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>, timed_out: bool) {
        let Some(sniffer) = sniffer(tick) else { return };
        if timed_out {
            tick.state
                .set_with_expiry(types::SNIFF_COOLDOWN, (), SNIFFING_COOLDOWN_TICKS as i64);
        } else {
            reset_sniffing(tick.state, sniffer);
        }
    }

    fn debug_name(&self) -> &'static str {
        "Digging"
    }
}

struct FinishedDigging {
    conditions: [(MemoryModuleId, MemoryStatus); 4],
    duration: i32,
}

impl FinishedDigging {
    const fn new(duration: i32) -> Self {
        Self {
            conditions: [
                (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
                (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
                (types::SNIFFER_DIGGING.id(), MemoryStatus::ValuePresent),
                (types::SNIFF_COOLDOWN.id(), MemoryStatus::ValuePresent),
            ],
            duration,
        }
    }
}

impl Behavior for FinishedDigging {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &self.conditions
    }

    fn min_duration(&self) -> i32 {
        self.duration
    }

    fn max_duration(&self) -> i32 {
        self.duration
    }

    fn check_extra_start_conditions(&mut self, _tick: &mut BrainTick<'_>) -> bool {
        true
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.state.get(types::SNIFFER_DIGGING).is_some()
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Rising);
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>, timed_out: bool) {
        let Some(sniffer) = sniffer(tick) else { return };
        sniffer.transition_to(SnifferState::Idling);
        if timed_out {
            store_explored_position(tick.state, sniffer, sniffer.get_entity().get_on_pos());
        }
        tick.state.erase(types::SNIFFER_DIGGING.id());
        tick.state.set(types::SNIFFER_HAPPY, true);
    }

    fn debug_name(&self) -> &'static str {
        "FinishedDigging"
    }
}

struct FeelingHappy {
    conditions: [(MemoryModuleId, MemoryStatus); 1],
    min: i32,
    max: i32,
}

impl FeelingHappy {
    const fn new(min: i32, max: i32) -> Self {
        Self {
            conditions: [(types::SNIFFER_HAPPY.id(), MemoryStatus::ValuePresent)],
            min,
            max,
        }
    }
}

impl Behavior for FeelingHappy {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &self.conditions
    }

    fn min_duration(&self) -> i32 {
        self.min
    }

    fn max_duration(&self) -> i32 {
        self.max
    }

    fn can_still_use(&mut self, _tick: &mut BrainTick<'_>) -> bool {
        true
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::FeelingHappy);
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>, _timed_out: bool) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Idling);
        }
        tick.state.erase(types::SNIFFER_HAPPY.id());
    }

    fn debug_name(&self) -> &'static str {
        "FeelingHappy"
    }
}

struct Scenting {
    conditions: [(MemoryModuleId, MemoryStatus); 5],
    min: i32,
    max: i32,
}

impl Scenting {
    const fn new(min: i32, max: i32) -> Self {
        Self {
            conditions: [
                (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
                (types::SNIFFER_DIGGING.id(), MemoryStatus::ValueAbsent),
                (
                    types::SNIFFER_SNIFFING_TARGET.id(),
                    MemoryStatus::ValueAbsent,
                ),
                (types::SNIFFER_HAPPY.id(), MemoryStatus::ValueAbsent),
                (types::BREED_TARGET.id(), MemoryStatus::ValueAbsent),
            ],
            min,
            max,
        }
    }
}

impl Behavior for Scenting {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &self.conditions
    }

    fn min_duration(&self) -> i32 {
        self.min
    }

    fn max_duration(&self) -> i32 {
        self.max
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

    fn stop(&mut self, tick: &mut BrainTick<'_>, _timed_out: bool) {
        if let Some(sniffer) = sniffer(tick) {
            sniffer.transition_to(SnifferState::Idling);
        }
    }

    fn debug_name(&self) -> &'static str {
        "Scenting"
    }
}
