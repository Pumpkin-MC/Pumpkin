use std::sync::Arc;

use pumpkin_data::environment_attribute::Activity;

use super::{BreezeEntity, movement, shoot};
use crate::entity::EntityBase;
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainState, BrainTick,
    activity::ActivityData,
    behavior::{
        DoNothing, LookAtTargetSink, OneShot, Swim, Timed, random_stroll, run_one, start_attacking,
        stop_attacking_if_target_invalid,
    },
    memory::{MemoryStatus, PackedMemories, types},
    sensing::{SensorType, is_entity_attackable, remember_positives},
};
use crate::entity::mob::Mob;

const TICKS_TO_REMEMBER_SEEN_TARGET: i32 = 100;

pub(super) fn make_brain(breeze: &BreezeEntity, packed: &PackedMemories) -> Brain {
    let mut brain = BrainProvider {
        memory_types: &[],
        sensor_types: &[
            SensorType::HurtBy,
            SensorType::NearestPlayers,
            SensorType::BreezeAttackEntity,
        ],
        activities,
    }
    .make_brain(breeze, packed);
    brain.set_default_activity(Activity::Fight);
    brain.use_default_activity();
    brain
}

pub(super) fn update_activity(brain: &mut BrainState) {
    brain.set_active_activity_to_first_valid(&[Activity::Fight, Activity::Idle]);
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(Timed::new(Swim::new(0.8))),
                Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
            ],
        ),
        ActivityData::with_priority_start(
            Activity::Idle,
            0,
            vec![
                Box::new(start_attacking::create(|_| true, nearest_attackable)),
                Box::new(start_attacking::create(|_| true, hurt_by)),
                movement::slide_to_target_sink(),
                Box::new(run_one(vec![
                    (Box::new(DoNothing::new(20, 100)), 1),
                    (Box::new(random_stroll::stroll(0.6)), 2),
                ])),
            ],
        ),
        ActivityData::with_priority_start_and_conditions(
            Activity::Fight,
            0,
            vec![
                Box::new(stop_attacking()),
                shoot::shoot(),
                movement::long_jump(),
                shoot::shoot_when_stuck(),
                movement::slide(),
            ],
            vec![
                (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
                (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            ],
        ),
    ]
}

fn nearest_attackable(tick: &BrainTick<'_>) -> Option<Arc<dyn EntityBase>> {
    tick.state.get(types::NEAREST_ATTACKABLE).cloned()
}

fn hurt_by(tick: &BrainTick<'_>) -> Option<Arc<dyn EntityBase>> {
    let target = tick.state.get(types::HURT_BY)?.attacker.clone()?;
    target.get_living_entity()?;
    Some(target)
}

fn stop_attacking() -> OneShot {
    let mut recently_attackable = remember_positives(TICKS_TO_REMEMBER_SEEN_TARGET, |&value| value);
    stop_attacking_if_target_invalid::create_with_condition(
        move |tick, target| !recently_attackable(&is_entity_attackable(&tick.visibility(), target)),
        |_, _| {},
    )
}
