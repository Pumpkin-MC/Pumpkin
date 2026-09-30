use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::Activity;
use rand::RngExt;

use crate::entity::EntityBase;
use crate::entity::ai::brain::activity::ActivityData;
use crate::entity::ai::brain::behavior::{self, BehaviorControl};
use crate::entity::ai::brain::memory::{MemoryModuleId, types};
use crate::entity::ai::brain::sensing::{SensorType, is_entity_attackable_ignoring_line_of_sight};
use crate::entity::ai::brain::{BrainProvider, BrainTick, VisibilityContext};
use crate::entity::mob::piglin_ai;
use crate::entity::mob::piglin_brute::PiglinBruteEntity;

pub const ANGER_DURATION: i64 = 600;
pub const MELEE_ATTACK_COOLDOWN: i32 = 20;
pub const ACTIVITY_SOUND_LIKELIHOOD_PER_TICK: f32 = 0.0125;
pub const MAX_LOOK_DIST: f32 = 8.0;
pub const INTERACTION_RANGE: i32 = 8;
pub const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 0.6;
pub const HOME_CLOSE_ENOUGH_DISTANCE: i32 = 2;
pub const HOME_TOO_FAR_DISTANCE: i32 = 100;
pub const HOME_STROLL_AROUND_DISTANCE: i32 = 5;

/// Vanilla `PiglinBruteAi.findNearestValidAttackTarget`.
#[must_use]
pub fn find_nearest_valid_attack_target(
    ctx: &VisibilityContext<'_>,
) -> Option<Arc<dyn EntityBase>> {
    if let Some(angry_at) = piglin_ai::get_anger_target(ctx.brain, ctx.world)
        && is_entity_attackable_ignoring_line_of_sight(ctx, angry_at.as_ref())
    {
        return Some(angry_at);
    }
    if let Some(player) = ctx.brain.get(types::NEAREST_VISIBLE_ATTACKABLE_PLAYER) {
        return Some(player.clone() as Arc<dyn EntityBase>);
    }
    ctx.brain
        .get(types::NEAREST_VISIBLE_NEMESIS)
        .map(Arc::clone)
}

#[must_use]
fn is_nearest_valid_attack_target(
    ctx: &VisibilityContext<'_>,
    target: &Arc<dyn EntityBase>,
) -> bool {
    find_nearest_valid_attack_target(ctx)
        .is_some_and(|nearest| nearest.get_entity().entity_id == target.get_entity().entity_id)
}

pub fn init_memories(tick: &mut BrainTick<'_>) {
    let pos = tick.mob.get_entity().block_pos.load();
    if let Some(home) = behavior::utils::global_pos_in(tick.world, pos) {
        tick.brain.set(types::HOME, home);
    }
}

pub fn set_anger_target(tick: &mut BrainTick<'_>, target: &Arc<dyn EntityBase>) {
    tick.brain.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
    tick.brain.set_with_expiry(
        types::ANGRY_AT,
        target.get_entity().entity_uuid,
        ANGER_DURATION,
    );
}

/// Vanilla `PiglinBruteAi.wasHurtBy`: a piglin or brute attacker is ignored entirely.
pub fn was_hurt_by(tick: &mut BrainTick<'_>, attacker: &Arc<dyn EntityBase>) {
    let attacker_type = attacker.get_entity().entity_type;
    if attacker_type == &EntityType::PIGLIN || attacker_type == &EntityType::PIGLIN_BRUTE {
        return;
    }
    piglin_ai::maybe_retaliate(tick, attacker);
}

fn idle_look_behaviors() -> Box<dyn BehaviorControl> {
    Box::new(behavior::run_one(vec![
        (
            Box::new(behavior::set_entity_look_target_of_type(
                &EntityType::PLAYER,
                MAX_LOOK_DIST,
            )),
            1,
        ),
        (
            Box::new(behavior::set_entity_look_target_of_type(
                &EntityType::PIGLIN,
                MAX_LOOK_DIST,
            )),
            1,
        ),
        (
            Box::new(behavior::set_entity_look_target_of_type(
                &EntityType::PIGLIN_BRUTE,
                MAX_LOOK_DIST,
            )),
            1,
        ),
        (
            Box::new(behavior::set_entity_look_target_any(MAX_LOOK_DIST)),
            1,
        ),
        (Box::new(behavior::DoNothing::new(30, 60)), 1),
    ]))
}

fn idle_movement_behaviors() -> Box<dyn BehaviorControl> {
    Box::new(behavior::run_one(vec![
        (
            Box::new(behavior::stroll(SPEED_MULTIPLIER_WHEN_IDLING, true)),
            2,
        ),
        (
            Box::new(behavior::interact_with(
                &EntityType::PIGLIN,
                INTERACTION_RANGE,
                |_| true,
                |_| true,
                types::INTERACTION_TARGET,
                SPEED_MULTIPLIER_WHEN_IDLING,
                2,
            )),
            2,
        ),
        (
            Box::new(behavior::interact_with(
                &EntityType::PIGLIN_BRUTE,
                INTERACTION_RANGE,
                |_| true,
                |_| true,
                types::INTERACTION_TARGET,
                SPEED_MULTIPLIER_WHEN_IDLING,
                2,
            )),
            2,
        ),
        (
            Box::new(behavior::stroll_to_poi(
                types::HOME,
                SPEED_MULTIPLIER_WHEN_IDLING,
                HOME_CLOSE_ENOUGH_DISTANCE,
                HOME_TOO_FAR_DISTANCE,
            )),
            2,
        ),
        (
            Box::new(behavior::stroll_around_poi(
                types::HOME,
                SPEED_MULTIPLIER_WHEN_IDLING,
                HOME_STROLL_AROUND_DISTANCE,
            )),
            2,
        ),
        (Box::new(behavior::DoNothing::new(30, 60)), 1),
    ]))
}

fn init_core_activity() -> ActivityData {
    ActivityData::with_priority_start(
        Activity::Core,
        0,
        vec![
            Box::new(behavior::Timed::new(behavior::LookAtTargetSink::new(
                45, 90,
            ))),
            Box::new(behavior::Timed::new(behavior::MoveToTargetSink::default())),
            Box::new(behavior::interact_with_door()),
            Box::new(behavior::stop_being_angry_if_target_dead()),
        ],
    )
}

fn init_idle_activity() -> ActivityData {
    ActivityData::with_priority_start(
        Activity::Idle,
        10,
        vec![
            Box::new(behavior::start_attacking(
                |_| true,
                |tick| find_nearest_valid_attack_target(&tick.visibility()),
            )),
            idle_look_behaviors(),
            idle_movement_behaviors(),
            Box::new(behavior::set_look_and_interact(&EntityType::PLAYER, 4)),
        ],
    )
}

fn init_fight_activity() -> ActivityData {
    ActivityData::with_gate_memory(
        Activity::Fight,
        10,
        vec![
            Box::new(behavior::stop_attacking_if_target_invalid(
                |tick, target| !is_nearest_valid_attack_target(&tick.visibility(), target),
                |_, _| {},
                true,
            )),
            Box::new(behavior::set_walk_target_from_attack_target_if_target_out_of_reach(1.0)),
            Box::new(behavior::melee_attack(|_| true, MELEE_ATTACK_COOLDOWN)),
        ],
        types::ATTACK_TARGET.id(),
    )
}

/// Vanilla `PiglinBruteAi.updateActivity`.
pub fn update_activity(tick: &mut BrainTick<'_>) {
    let old_activity = tick.brain.get_active_non_core_activity();
    tick.brain
        .set_active_activity_to_first_valid(&[Activity::Fight, Activity::Idle]);
    let new_activity = tick.brain.get_active_non_core_activity();
    if old_activity != new_activity {
        play_activity_sound(tick);
    }
    let has_attack_target = tick.brain.has_memory_value(types::ATTACK_TARGET.id());
    tick.mob.get_mob_entity().set_attacking(has_attack_target);
}

pub fn maybe_play_activity_sound(tick: &mut BrainTick<'_>) {
    if tick.mob.get_random().random::<f32>() < ACTIVITY_SOUND_LIKELIHOOD_PER_TICK {
        play_activity_sound(tick);
    }
}

fn play_activity_sound(tick: &BrainTick<'_>) {
    if tick.brain.get_active_non_core_activity() != Some(Activity::Fight) {
        return;
    }
    if let Some(brute) = tick.mob.cast_any().downcast_ref::<PiglinBruteEntity>() {
        brute.play_angry_sound();
    }
}

pub const MEMORY_TYPES: &[MemoryModuleId] = &[types::HOME.id()];

pub const SENSOR_TYPES: &[SensorType] = &[
    SensorType::NearestLivingEntities,
    SensorType::NearestPlayers,
    SensorType::NearestItems,
    SensorType::HurtBy,
    SensorType::PiglinBruteSpecific,
];

pub static PIGLIN_BRUTE_PROVIDER: BrainProvider = BrainProvider {
    memory_types: MEMORY_TYPES,
    sensor_types: SENSOR_TYPES,
    activities: |_| {
        vec![
            init_core_activity(),
            init_idle_activity(),
            init_fight_activity(),
        ]
    },
};
