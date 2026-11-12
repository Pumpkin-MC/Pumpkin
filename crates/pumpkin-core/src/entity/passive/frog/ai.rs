use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::Activity;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use super::FrogEntity;
use super::shoot_tongue::ShootTongue;
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainState,
    activity::ActivityData,
    behavior::{
        AnimalMakeLove, AnimalPanic, BehaviorControl, CountDownCooldownTicks, FollowTemptation,
        GateBehavior, LookAtTargetSink, MoveToTargetSink, OrderPolicy, RunningPolicy, Timed,
        croak::Croak,
        follow_temptation::DEFAULT_CLOSE_ENOUGH_DIST,
        long_jump::{self, LongJumpMidJump, LongJumpToRandomPos, PreferredBlock},
        one_shot::trigger_if,
        random_stroll, run_one, run_one_with_conditions, set_entity_look_target_sometimes,
        set_walk_target_from_look_target, start_attacking, stop_attacking_if_target_invalid,
        try_find_land, try_lay_spawn_on_fluid_near_land, utils,
    },
    memory::{MemoryModuleId, MemoryStatus, PackedMemories, types},
    sensing::SensorType,
};
use crate::entity::mob::Mob;

const SPEED_MULTIPLIER_WHEN_PANICKING: f32 = 2.0;
const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 1.0;
const SPEED_MULTIPLIER_ON_LAND: f32 = 1.0;
const SPEED_MULTIPLIER_IN_WATER: f32 = 0.75;
const TIME_BETWEEN_LONG_JUMPS: long_jump::TimeBetweenLongJumps = (100, 140);
const MAX_LONG_JUMP_HEIGHT: i32 = 2;
const MAX_LONG_JUMP_WIDTH: i32 = 4;
const MAX_JUMP_VELOCITY_MULTIPLIER: f32 = 3.571_428_8;
const SPEED_MULTIPLIER_WHEN_TEMPTED: f32 = 1.25;
const PREFERRED_BLOCKS_CHANCE: f32 = 0.5;

pub(super) fn make_brain(frog: &FrogEntity, packed: &PackedMemories) -> Brain {
    BrainProvider {
        memory_types: &[],
        sensor_types: &[
            SensorType::NearestLivingEntities,
            SensorType::HurtBy,
            SensorType::FrogAttackables,
            SensorType::FrogTemptations,
            SensorType::IsInWater,
        ],
        activities,
    }
    .make_brain(frog, packed)
}

pub(super) fn init_memories(brain: &mut BrainState, frog: &FrogEntity) {
    let cooldown = frog
        .get_random()
        .random_range(TIME_BETWEEN_LONG_JUMPS.0..=TIME_BETWEEN_LONG_JUMPS.1);
    brain.set(types::LONG_JUMP_COOLDOWN_TICKS, cooldown);
}

pub(super) fn update_activity(brain: &mut BrainState) {
    brain.set_active_activity_to_first_valid(&[
        Activity::Tongue,
        Activity::LaySpawn,
        Activity::LongJump,
        Activity::Swim,
        Activity::Idle,
    ]);
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        core_activity(),
        idle_activity(),
        swim_activity(),
        lay_spawn_activity(),
        tongue_activity(),
        jump_activity(),
    ]
}

fn look_at_player() -> Box<dyn BehaviorControl> {
    Box::new(set_entity_look_target_sometimes::create_matching(
        Some(&EntityType::PLAYER),
        6.0,
        30,
        60,
    ))
}

fn follow_temptation() -> Box<dyn BehaviorControl> {
    Box::new(Timed::new(FollowTemptation::new(
        |_| SPEED_MULTIPLIER_WHEN_TEMPTED,
        |_| DEFAULT_CLOSE_ENOUGH_DIST,
    )))
}

fn start_attacking() -> Box<dyn BehaviorControl> {
    Box::new(start_attacking::create(
        |tick| !utils::is_breeding(tick.state),
        |tick| tick.state.get(types::NEAREST_ATTACKABLE).cloned(),
    ))
}

fn on_ground() -> Box<dyn BehaviorControl> {
    Box::new(trigger_if("TriggerIf", Vec::new(), |tick| {
        tick.mob
            .get_entity()
            .on_ground
            .load(std::sync::atomic::Ordering::Relaxed)
    }))
}

fn not_mid_jump_and(
    memory: MemoryModuleId,
    status: MemoryStatus,
) -> Vec<(MemoryModuleId, MemoryStatus)> {
    vec![
        (types::LONG_JUMP_MID_JUMP.id(), MemoryStatus::ValueAbsent),
        (memory, status),
    ]
}

fn core_activity() -> ActivityData {
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
            Box::new(CountDownCooldownTicks::new(types::LONG_JUMP_COOLDOWN_TICKS)),
        ],
    )
}

fn idle_activity() -> ActivityData {
    ActivityData::with_pairs_and_conditions(
        Activity::Idle,
        vec![
            (0, look_at_player()),
            (
                0,
                Box::new(Timed::new(AnimalMakeLove::new(&EntityType::FROG))),
            ),
            (1, follow_temptation()),
            (2, start_attacking()),
            (
                3,
                Box::new(try_find_land::create(6, SPEED_MULTIPLIER_WHEN_IDLING)),
            ),
            (
                4,
                Box::new(run_one_with_conditions(
                    vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
                    vec![
                        (
                            Box::new(random_stroll::stroll(SPEED_MULTIPLIER_WHEN_IDLING)),
                            1,
                        ),
                        (
                            Box::new(set_walk_target_from_look_target::create(
                                SPEED_MULTIPLIER_ON_LAND,
                                3,
                            )),
                            1,
                        ),
                        (Box::new(Timed::new(Croak::default())), 3),
                        (on_ground(), 2),
                    ],
                )),
            ),
        ],
        not_mid_jump_and(types::IS_IN_WATER.id(), MemoryStatus::ValueAbsent),
    )
}

fn swim_activity() -> ActivityData {
    ActivityData::with_pairs_and_conditions(
        Activity::Swim,
        vec![
            (0, look_at_player()),
            (1, follow_temptation()),
            (2, start_attacking()),
            (3, Box::new(try_find_land::create(8, 1.5))),
            (
                5,
                Box::new(GateBehavior::new(
                    "GateBehavior",
                    vec![(types::WALK_TARGET.id(), MemoryStatus::ValueAbsent)],
                    Vec::new(),
                    OrderPolicy::Ordered,
                    RunningPolicy::TryAll,
                    vec![
                        (Box::new(random_stroll::swim(SPEED_MULTIPLIER_IN_WATER)), 1),
                        (
                            Box::new(random_stroll::stroll_with_water(
                                SPEED_MULTIPLIER_ON_LAND,
                                true,
                            )),
                            1,
                        ),
                        (
                            Box::new(set_walk_target_from_look_target::create(
                                SPEED_MULTIPLIER_ON_LAND,
                                3,
                            )),
                            1,
                        ),
                        (
                            Box::new(trigger_if("TriggerIf", Vec::new(), |tick| {
                                tick.mob.get_entity().is_in_water()
                            })),
                            5,
                        ),
                    ],
                )),
            ),
        ],
        not_mid_jump_and(types::IS_IN_WATER.id(), MemoryStatus::ValuePresent),
    )
}

fn lay_spawn_activity() -> ActivityData {
    ActivityData::with_pairs_and_conditions(
        Activity::LaySpawn,
        vec![
            (0, look_at_player()),
            (1, start_attacking()),
            (
                2,
                Box::new(try_find_land::create_near_liquid(
                    8,
                    SPEED_MULTIPLIER_ON_LAND,
                    &tag::Fluid::MINECRAFT_FROG_TRIES_TO_FIND_LAND_NEAR,
                )),
            ),
            (
                3,
                Box::new(try_lay_spawn_on_fluid_near_land::create(
                    &pumpkin_data::Block::FROGSPAWN,
                )),
            ),
            (
                4,
                Box::new(run_one(vec![
                    (
                        Box::new(random_stroll::stroll(SPEED_MULTIPLIER_WHEN_IDLING)),
                        2,
                    ),
                    (
                        Box::new(set_walk_target_from_look_target::create(
                            SPEED_MULTIPLIER_ON_LAND,
                            3,
                        )),
                        1,
                    ),
                    (Box::new(Timed::new(Croak::default())), 2),
                    (on_ground(), 1),
                ])),
            ),
        ],
        not_mid_jump_and(types::IS_PREGNANT.id(), MemoryStatus::ValuePresent),
    )
}

fn jump_activity() -> ActivityData {
    ActivityData::with_pairs_and_conditions(
        Activity::LongJump,
        vec![
            (
                0,
                Box::new(Timed::new(LongJumpMidJump::new(
                    TIME_BETWEEN_LONG_JUMPS,
                    Sound::EntityFrogStep,
                ))),
            ),
            (
                1,
                Box::new(Timed::new(LongJumpToRandomPos::new(
                    TIME_BETWEEN_LONG_JUMPS,
                    MAX_LONG_JUMP_HEIGHT,
                    MAX_LONG_JUMP_WIDTH,
                    MAX_JUMP_VELOCITY_MULTIPLIER,
                    Sound::EntityFrogLongJump,
                    is_acceptable_landing_spot,
                    Some(PreferredBlock {
                        tag: &tag::Block::MINECRAFT_FROG_PREFER_JUMP_TO,
                        chance: PREFERRED_BLOCKS_CHANCE,
                    }),
                ))),
            ),
        ],
        vec![
            (types::TEMPTING_PLAYER.id(), MemoryStatus::ValueAbsent),
            (types::BREED_TARGET.id(), MemoryStatus::ValueAbsent),
            (
                types::LONG_JUMP_COOLDOWN_TICKS.id(),
                MemoryStatus::ValueAbsent,
            ),
            (types::IS_IN_WATER.id(), MemoryStatus::ValueAbsent),
        ],
    )
}

fn tongue_activity() -> ActivityData {
    ActivityData::with_gate_memory(
        Activity::Tongue,
        0,
        vec![
            Box::new(stop_attacking_if_target_invalid::create()),
            Box::new(Timed::new(ShootTongue::new(
                Sound::EntityFrogTongue,
                Sound::EntityFrogEat,
            ))),
        ],
        types::ATTACK_TARGET.id(),
    )
}

fn is_acceptable_landing_spot(mob: &dyn Mob, target_pos: BlockPos) -> bool {
    let world = mob.get_entity().world.load();
    let below = target_pos.down();
    let no_fluid = |pos: &BlockPos| world.get_fluid(pos).id == pumpkin_data::fluid::Fluid::EMPTY.id;
    if !no_fluid(&target_pos) || !no_fluid(&below) || !no_fluid(&target_pos.up()) {
        return false;
    }
    let preferred = &tag::Block::MINECRAFT_FROG_PREFER_JUMP_TO;
    if world.get_block(&target_pos).has_tag(preferred) || world.get_block(&below).has_tag(preferred)
    {
        return true;
    }
    long_jump::lands_on_trapdoor(mob, target_pos)
        || long_jump::default_acceptable_landing_spot(mob, target_pos)
}
