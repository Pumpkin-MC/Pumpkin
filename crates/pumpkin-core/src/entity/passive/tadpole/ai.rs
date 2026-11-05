use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::Activity;

use super::TadpoleEntity;
use crate::entity::ai::brain::{
    Brain, BrainProvider,
    activity::ActivityData,
    behavior::{
        AnimalPanic, CountDownCooldownTicks, FollowTemptation, GateBehavior, LookAtTargetSink,
        MoveToTargetSink, OrderPolicy, RunningPolicy, Timed,
        follow_temptation::DEFAULT_CLOSE_ENOUGH_DIST, one_shot::trigger_if, random_stroll,
        set_entity_look_target_sometimes, set_walk_target_from_look_target,
    },
    memory::{MemoryStatus, PackedMemories, types},
    sensing::SensorType,
};
use crate::entity::mob::Mob;

const SPEED_MULTIPLIER_WHEN_PANICKING: f32 = 2.0;
const SPEED_MULTIPLIER_WHEN_IDLING_IN_WATER: f32 = 0.5;
const SPEED_MULTIPLIER_WHEN_TEMPTED: f32 = 1.25;

pub(super) fn make_brain(tadpole: &TadpoleEntity, packed: &PackedMemories) -> Brain {
    BrainProvider {
        memory_types: &[],
        sensor_types: &[
            SensorType::NearestLivingEntities,
            SensorType::NearestPlayers,
            SensorType::HurtBy,
            SensorType::FrogTemptations,
        ],
        activities,
    }
    .make_brain(tadpole, packed)
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(Timed::new(AnimalPanic::new(
                    SPEED_MULTIPLIER_WHEN_PANICKING,
                ))),
                Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
                Box::new(Timed::new(MoveToTargetSink::default())),
                Box::new(CountDownCooldownTicks::new(
                    types::TEMPTATION_COOLDOWN_TICKS,
                )),
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
                Box::new(Timed::new(FollowTemptation::new(
                    |_| SPEED_MULTIPLIER_WHEN_TEMPTED,
                    |_| DEFAULT_CLOSE_ENOUGH_DIST,
                ))),
                Box::new(GateBehavior::new(
                    "GateBehavior",
                    vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
                    Vec::new(),
                    OrderPolicy::Ordered,
                    RunningPolicy::TryAll,
                    vec![
                        (
                            Box::new(random_stroll::swim(SPEED_MULTIPLIER_WHEN_IDLING_IN_WATER)),
                            2,
                        ),
                        (
                            Box::new(set_walk_target_from_look_target::create(
                                SPEED_MULTIPLIER_WHEN_IDLING_IN_WATER,
                                3,
                            )),
                            3,
                        ),
                        (
                            Box::new(trigger_if("TriggerIf", Vec::new(), |tick| {
                                tick.mob.get_entity().is_in_water()
                            })),
                            5,
                        ),
                    ],
                )),
            ],
        ),
    ]
}
