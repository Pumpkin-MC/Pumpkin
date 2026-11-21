use pumpkin_data::environment_attribute::Activity;

use super::HappyGhastEntity;
use crate::entity::ai::brain::{
    Brain, BrainProvider,
    activity::ActivityData,
    behavior::{
        AnimalPanic, CountDownCooldownTicks, LookAtTargetSink, MoveToTargetSink, Swim, Timed,
        baby_follow_adult, follow_temptation::FollowTemptation, random_stroll, run_one,
        set_walk_target_from_look_target,
    },
    memory::{MemoryModuleId, MemoryStatus, PackedMemories, types},
    sensing::SensorType,
};
use crate::entity::mob::Mob;

pub const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 1.0;
pub const SPEED_MULTIPLIER_WHEN_TEMPTED: f32 = 1.25;
pub const SPEED_MULTIPLIER_WHEN_FOLLOWING_ADULT: f32 = 1.1;
pub const SPEED_MULTIPLIER_WHEN_PANICKING: f32 = 2.0;
pub const BABY_GHAST_CLOSE_ENOUGH_DIST: f64 = 3.0;
pub const ADULT_FOLLOW_RANGE: (i32, i32) = (3, 16);

const MEMORIES: [MemoryModuleId; 13] = [
    types::IS_PANICKING.id(),
    types::HURT_BY.id(),
    types::HURT_BY_ENTITY.id(),
    types::WALK_TARGET.id(),
    types::LOOK_TARGET.id(),
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
    types::NEAREST_PLAYERS.id(),
    types::NEAREST_VISIBLE_PLAYER.id(),
    types::NEAREST_VISIBLE_ADULT.id(),
    types::TEMPTATION_COOLDOWN_TICKS.id(),
    types::TEMPTING_PLAYER.id(),
    types::IS_TEMPTED.id(),
    types::BREED_TARGET.id(),
];

pub(super) fn make_brain(happy_ghast: &HappyGhastEntity, packed: &PackedMemories) -> Brain {
    BrainProvider {
        memory_types: &MEMORIES,
        sensor_types: &[
            SensorType::NearestLivingEntities,
            SensorType::HurtBy,
            SensorType::FoodTemptations,
            SensorType::NearestAdult,
            SensorType::NearestPlayers,
        ],
        activities,
    }
    .make_brain(happy_ghast, packed)
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        init_core_activity(),
        init_idle_activity(),
        init_panic_activity(),
    ]
}

fn init_core_activity() -> ActivityData {
    ActivityData::with_priority_start(
        Activity::Core,
        0,
        vec![
            Box::new(Timed::new(Swim::new(0.8))),
            Box::new(Timed::new(AnimalPanic::new(
                SPEED_MULTIPLIER_WHEN_PANICKING,
            ))),
            Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
            Box::new(Timed::new(MoveToTargetSink::default())),
            Box::new(CountDownCooldownTicks::new(
                types::TEMPTATION_COOLDOWN_TICKS,
            )),
        ],
    )
}

fn init_idle_activity() -> ActivityData {
    ActivityData::with_pairs(
        Activity::Idle,
        vec![
            (
                1,
                Box::new(Timed::new(FollowTemptation::new_with_eyes(
                    |_| SPEED_MULTIPLIER_WHEN_TEMPTED,
                    |_| BABY_GHAST_CLOSE_ENOUGH_DIST,
                    true,
                ))),
            ),
            (
                2,
                Box::new(baby_follow_adult::create_for_player(
                    ADULT_FOLLOW_RANGE.0,
                    ADULT_FOLLOW_RANGE.1,
                    |_| SPEED_MULTIPLIER_WHEN_FOLLOWING_ADULT,
                )),
            ),
            (
                3,
                Box::new(baby_follow_adult::create_with(
                    ADULT_FOLLOW_RANGE.0,
                    ADULT_FOLLOW_RANGE.1,
                    |_| SPEED_MULTIPLIER_WHEN_FOLLOWING_ADULT,
                )),
            ),
            (
                4,
                Box::new(run_one(vec![
                    (
                        Box::new(random_stroll::fly(SPEED_MULTIPLIER_WHEN_IDLING)),
                        1,
                    ),
                    (
                        Box::new(set_walk_target_from_look_target::create(1.0, 3)),
                        1,
                    ),
                ])),
            ),
        ],
    )
}

fn init_panic_activity() -> ActivityData {
    ActivityData::new(
        Activity::Panic,
        vec![],
        vec![(types::IS_PANICKING.id(), MemoryStatus::ValuePresent)],
        vec![],
    )
}

pub fn update_activity(body: &HappyGhastEntity) {
    body.mob_entity.with_brain(body, |brain| {
        brain.set_active_activity_to_first_valid(&[Activity::Panic, Activity::Idle]);
    });
}
