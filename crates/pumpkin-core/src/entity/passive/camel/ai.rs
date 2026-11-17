use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::Activity;

use super::CamelEntity;
use crate::entity::EntityBase;
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainTick,
    activity::ActivityData,
    behavior::{
        AnimalMakeLove, AnimalPanic, Behavior, CountDownCooldownTicks, DoNothing, FollowTemptation,
        LookAtTargetSink, MoveToTargetSink, OneShot, RandomLookAround, Swim, Timed,
        baby_follow_adult, one_shot::sequence, random_stroll, run_one, run_one_with_conditions,
        set_entity_look_target_sometimes, set_walk_target_from_look_target,
    },
    memory::{MemoryModuleId, MemoryStatus, PackedMemories, types},
    sensing::SensorType,
};
use crate::entity::mob::Mob;
use crate::entity::passive::abstract_horse::AbstractHorse;

const SPEED_MULTIPLIER_WHEN_PANICKING: f32 = 4.0;
const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 2.0;
const SPEED_MULTIPLIER_WHEN_TEMPTED: f32 = 2.5;
const SPEED_MULTIPLIER_WHEN_FOLLOWING_ADULT: f32 = 2.5;
const ADULT_FOLLOW_RANGE: (i32, i32) = (5, 16);
const RANDOM_SITTING_MINIMAL_POSE_SECONDS: i64 = 20;

pub(super) fn make_brain(camel: &CamelEntity, packed: &PackedMemories) -> Brain {
    BrainProvider {
        memory_types: &[],
        sensor_types: &[
            SensorType::NearestLivingEntities,
            SensorType::HurtBy,
            SensorType::FoodTemptations,
            SensorType::NearestAdult,
        ],
        activities,
    }
    .make_brain(camel, packed)
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(Timed::new(Swim::new(0.8))),
                Box::new(Timed::new(CamelPanic(AnimalPanic::new(
                    SPEED_MULTIPLIER_WHEN_PANICKING,
                )))),
                Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
                Box::new(Timed::new(MoveToTargetSink::default())),
                Box::new(CountDownCooldownTicks::new(
                    types::TEMPTATION_COOLDOWN_TICKS,
                )),
                Box::new(CountDownCooldownTicks::new(types::GAZE_COOLDOWN_TICKS)),
            ],
        ),
        ActivityData::with_priority_start(
            Activity::Idle,
            0,
            vec![
                Box::new(set_entity_look_target_sometimes::create_matching(
                    Some(&EntityType::PLAYER),
                    6.0,
                    30,
                    60,
                )),
                Box::new(Timed::new(AnimalMakeLove::new(&EntityType::CAMEL))),
                Box::new(run_one(vec![
                    (
                        Box::new(Timed::new(FollowTemptation::new(
                            |_| SPEED_MULTIPLIER_WHEN_TEMPTED,
                            |mob| {
                                if mob.get_mob_entity().living_entity.is_baby() {
                                    2.5
                                } else {
                                    3.5
                                }
                            },
                        ))),
                        1,
                    ),
                    (
                        Box::new(when_able_to_move(baby_follow_adult::create(
                            ADULT_FOLLOW_RANGE.0,
                            ADULT_FOLLOW_RANGE.1,
                            SPEED_MULTIPLIER_WHEN_FOLLOWING_ADULT,
                        ))),
                        1,
                    ),
                ])),
                Box::new(Timed::new(RandomLookAround::new(150, 250, 30.0, 0.0, 0.0))),
                Box::new(run_one_with_conditions(
                    vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
                    vec![
                        (
                            Box::new(when_able_to_move(random_stroll::stroll(
                                SPEED_MULTIPLIER_WHEN_IDLING,
                            ))),
                            1,
                        ),
                        (
                            Box::new(when_able_to_move(set_walk_target_from_look_target::create(
                                SPEED_MULTIPLIER_WHEN_IDLING,
                                3,
                            ))),
                            1,
                        ),
                        (Box::new(Timed::new(RandomSitting)), 1),
                        (Box::new(DoNothing::new(30, 60)), 1),
                    ],
                )),
            ],
        ),
    ]
}

fn camel<'a>(tick: &BrainTick<'a>) -> Option<&'a CamelEntity> {
    tick.mob.cast_any().downcast_ref::<CamelEntity>()
}

fn when_able_to_move(behavior: OneShot) -> OneShot {
    sequence(
        "TriggerIf",
        |tick| camel(tick).is_some_and(|camel| !camel.refuse_to_move()),
        behavior,
    )
}

struct CamelPanic(AnimalPanic);

impl Behavior for CamelPanic {
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
            && camel(tick).is_some_and(|camel| !camel.is_mob_controlled())
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(camel) = camel(tick) {
            camel.stand_up_instantly();
        }
        self.0.start(tick);
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        self.0.tick(tick);
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        self.0.stop(tick);
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.0.can_still_use(tick)
    }

    fn debug_name(&self) -> &'static str {
        "CamelPanic"
    }
}

struct RandomSitting;

impl Behavior for RandomSitting {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &[]
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some(camel) = camel(tick) else {
            return false;
        };
        let entity = camel.get_entity();
        !entity.is_in_water()
            && camel.get_pose_time() >= RANDOM_SITTING_MINIMAL_POSE_SECONDS * 20
            && !entity.is_leashed()
            && entity.on_ground.load(Relaxed)
            && camel.get_controlling_passenger().is_none()
            && camel.can_camel_change_pose()
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let Some(camel) = camel(tick) else {
            return;
        };
        if camel.is_camel_sitting() {
            camel.stand_up();
        } else if !tick.state.has_memory_value(types::IS_PANICKING.id()) {
            camel.sit_down();
        }
    }

    fn debug_name(&self) -> &'static str {
        "RandomSitting"
    }
}
