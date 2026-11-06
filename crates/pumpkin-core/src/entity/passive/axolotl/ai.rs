use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::environment_attribute::Activity;
use pumpkin_data::potion::Effect;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;

use crate::entity::EntityBase;
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainTick,
    activity::ActivityData,
    behavior::{
        AnimalMakeLove, Behavior, CountDownCooldownTicks, FollowTemptation, GateBehavior,
        LookAtTargetSink, MoveToTargetSink, OneShot, OrderPolicy, RunningPolicy, Timed,
        baby_follow_adult, erase_memory_if, follow_temptation::DEFAULT_CLOSE_ENOUGH_DIST,
        melee_attack, one_shot::trigger_if, random_stroll, run_one,
        set_entity_look_target_sometimes,
        set_walk_target_from_attack_target_if_target_out_of_reach,
        set_walk_target_from_look_target, start_attacking, stop_attacking_if_target_invalid,
        try_find_liquid, utils,
    },
    memory::{MemoryModuleId, MemoryStatus, PackedMemories, types},
    sensing::SensorType,
};
use crate::entity::mob::Mob;

use super::AxolotlEntity;

/// Vanilla `ADULT_FOLLOW_RANGE`: a baby follows an adult 5 to 16 blocks away.
const ADULT_FOLLOW_RANGE: (i32, i32) = (5, 16);
const SPEED_MULTIPLIER_WHEN_MAKING_LOVE: f32 = 0.2;
const SPEED_MULTIPLIER_ON_LAND: f32 = 0.15;
const SPEED_MULTIPLIER_WHEN_IDLING_IN_WATER: f32 = 0.5;
const SPEED_MULTIPLIER_WHEN_CHASING_IN_WATER: f32 = 0.6;
const SPEED_MULTIPLIER_WHEN_FOLLOWING_ADULT_IN_WATER: f32 = 0.6;
/// Vanilla `MeleeAttack.create(20)`: ticks between two bites.
const ATTACK_COOLDOWN: i32 = 20;
/// Vanilla `PlayDead`'s duration (`super(..., 200)`).
const PLAY_DEAD_BEHAVIOR_DURATION: i32 = 200;
/// The regeneration `PlayDead.start` gives: `new MobEffectInstance(REGENERATION, 200, 0)`.
const PLAY_DEAD_REGENERATION_DURATION: i32 = 200;
/// Vanilla `AxolotlAi.updateActivity`: the hunting cooldown that follows a fight.
const HUNTING_COOLDOWN: i64 = 2400;

pub(super) fn make_brain(axolotl: &AxolotlEntity, packed: &PackedMemories) -> Brain {
    BrainProvider {
        memory_types: &[],
        sensor_types: &[
            SensorType::NearestLivingEntities,
            SensorType::NearestAdult,
            SensorType::HurtBy,
            SensorType::AxolotlAttackables,
            SensorType::FoodTemptations,
        ],
        activities,
    }
    .make_brain(axolotl, packed)
}

/// Vanilla `AxolotlAi.getActivities`: core, idle, fight and play dead.
fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        init_core_activity(),
        init_idle_activity(),
        init_fight_activity(),
        init_play_dead_activity(),
    ]
}

fn init_play_dead_activity() -> ActivityData {
    ActivityData::new(
        Activity::PlayDead,
        vec![
            (0, Box::new(Timed::new(PlayDead))),
            (
                1,
                Box::new(erase_memory_if::create(
                    |tick| utils::is_breeding(tick.state),
                    types::PLAY_DEAD_TICKS.id(),
                )),
            ),
        ],
        vec![(types::PLAY_DEAD_TICKS.id(), MemoryStatus::ValuePresent)],
        vec![types::PLAY_DEAD_TICKS.id()],
    )
}

fn init_fight_activity() -> ActivityData {
    ActivityData::with_gate_memory(
        Activity::Fight,
        0,
        vec![
            Box::new(stop_attacking_if_target_invalid::create_with_callback(
                AxolotlEntity::on_stop_attacking,
            )),
            Box::new(
                set_walk_target_from_attack_target_if_target_out_of_reach::create(
                    speed_modifier_chasing,
                ),
            ),
            Box::new(melee_attack::create(ATTACK_COOLDOWN)),
            Box::new(erase_memory_if::create(
                |tick| utils::is_breeding(tick.state),
                types::ATTACK_TARGET.id(),
            )),
        ],
        types::ATTACK_TARGET.id(),
    )
}

fn init_core_activity() -> ActivityData {
    ActivityData::with_priority_start(
        Activity::Core,
        0,
        vec![
            Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
            Box::new(Timed::new(MoveToTargetSink::default())),
            Box::new(validate_play_dead()),
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
                0,
                Box::new(set_entity_look_target_sometimes::create_matching(
                    Some(&EntityType::PLAYER),
                    6.0,
                    30,
                    60,
                )),
            ),
            (
                1,
                Box::new(Timed::new(AnimalMakeLove::with_speed_and_distance(
                    &EntityType::AXOLOTL,
                    SPEED_MULTIPLIER_WHEN_MAKING_LOVE,
                    2,
                ))),
            ),
            (
                2,
                Box::new(run_one(vec![
                    (
                        Box::new(Timed::new(FollowTemptation::new(speed_modifier, |_| {
                            DEFAULT_CLOSE_ENOUGH_DIST
                        }))),
                        1,
                    ),
                    (
                        Box::new(baby_follow_adult::create_with(
                            ADULT_FOLLOW_RANGE.0,
                            ADULT_FOLLOW_RANGE.1,
                            speed_modifier_following_adult,
                        )),
                        1,
                    ),
                ])),
            ),
            (
                3,
                Box::new(start_attacking::create(
                    |_| true,
                    find_nearest_valid_attack_target,
                )),
            ),
            (
                3,
                Box::new(try_find_liquid::create(
                    6,
                    SPEED_MULTIPLIER_ON_LAND,
                    &tag::Fluid::MINECRAFT_AXOLOTL_TRIES_TO_FIND,
                )),
            ),
            (
                4,
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
                            Box::new(random_stroll::stroll_with_water(
                                SPEED_MULTIPLIER_ON_LAND,
                                false,
                            )),
                            2,
                        ),
                        (
                            Box::new(set_walk_target_from_look_target::create_with(
                                can_set_walk_target_from_look_target,
                                speed_modifier,
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
                        (
                            Box::new(trigger_if("TriggerIf", Vec::new(), |tick| {
                                tick.mob.get_entity().on_ground.load(Relaxed)
                            })),
                            5,
                        ),
                    ],
                )),
            ),
        ],
    )
}

/// Vanilla `canSetWalkTargetFromLookTarget`: the look target has to be in the same kind of place
/// (water or not) as the axolotl is in.
fn can_set_walk_target_from_look_target(tick: &BrainTick<'_>) -> bool {
    tick.state.get(types::LOOK_TARGET).is_some_and(|look| {
        let in_water_at_target = tick
            .world
            .get_fluid(&look.current_block_position())
            .has_tag(&tag::Fluid::MINECRAFT_WATER);
        in_water_at_target == tick.mob.get_entity().is_in_water()
    })
}

/// Vanilla `AxolotlAi.updateActivity`: picks play dead, else fight, else idle, unless it is
/// playing dead already, and starts the hunting cooldown when a fight has just ended.
pub(super) fn update_activity(brain: &mut Brain) {
    let old_activity = brain.get_active_non_core_activity();
    // Playing dead holds the activity while the memory is there. When something erased the memory
    // (breeding does, `EraseMemoryIf` in the play dead activity) `ValidatePlayDead` has nothing to
    // end and the play dead activity has nothing left to run, so the choice is made again
    // instead of leaving the axolotl without behaviors (vanilla keeps the stale activity).
    if old_activity == Some(Activity::PlayDead)
        && brain.has_memory_value(types::PLAY_DEAD_TICKS.id())
    {
        return;
    }
    brain.set_active_activity_to_first_valid(&[
        Activity::PlayDead,
        Activity::Fight,
        Activity::Idle,
    ]);
    if old_activity == Some(Activity::Fight)
        && brain.get_active_non_core_activity() != Some(Activity::Fight)
    {
        brain.set_with_expiry(types::HAS_HUNTING_COOLDOWN, true, HUNTING_COOLDOWN);
    }
}

fn speed_modifier_chasing(mob: &dyn Mob) -> f32 {
    if mob.get_entity().is_in_water() {
        SPEED_MULTIPLIER_WHEN_CHASING_IN_WATER
    } else {
        SPEED_MULTIPLIER_ON_LAND
    }
}

fn speed_modifier_following_adult(mob: &dyn Mob) -> f32 {
    if mob.get_entity().is_in_water() {
        SPEED_MULTIPLIER_WHEN_FOLLOWING_ADULT_IN_WATER
    } else {
        SPEED_MULTIPLIER_ON_LAND
    }
}

fn speed_modifier(mob: &dyn Mob) -> f32 {
    if mob.get_entity().is_in_water() {
        SPEED_MULTIPLIER_WHEN_IDLING_IN_WATER
    } else {
        SPEED_MULTIPLIER_ON_LAND
    }
}

/// Vanilla `findNearestValidAttackTarget`: what the sensor found, unless the axolotl is breeding.
fn find_nearest_valid_attack_target(tick: &BrainTick<'_>) -> Option<Arc<dyn EntityBase>> {
    if utils::is_breeding(tick.state) {
        None
    } else {
        tick.state.get(types::NEAREST_ATTACKABLE).cloned()
    }
}

/// Vanilla `ValidatePlayDead`: counts the play dead ticks down and, once they are used up,
/// forgets them and the attacker and goes back to the default activity.
fn validate_play_dead() -> OneShot {
    OneShot::new(
        "ValidatePlayDead",
        vec![
            (types::PLAY_DEAD_TICKS.id(), MemoryStatus::ValuePresent),
            (types::HURT_BY_ENTITY.id(), MemoryStatus::Registered),
        ],
        |tick| {
            let Some(ticks) = tick.state.get(types::PLAY_DEAD_TICKS).copied() else {
                return false;
            };
            if ticks <= 0 {
                tick.state.erase(types::PLAY_DEAD_TICKS.id());
                tick.state.erase(types::HURT_BY_ENTITY.id());
                tick.state.use_default_activity();
            } else {
                tick.state.set(types::PLAY_DEAD_TICKS, ticks - 1);
            }
            true
        },
    )
}

/// Vanilla `PlayDead`: the axolotl lies still in the water and regenerates.
struct PlayDead;

const PLAY_DEAD_CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
    (types::PLAY_DEAD_TICKS.id(), MemoryStatus::ValuePresent),
    (types::HURT_BY_ENTITY.id(), MemoryStatus::ValuePresent),
];

impl Behavior for PlayDead {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &PLAY_DEAD_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        PLAY_DEAD_BEHAVIOR_DURATION
    }

    fn max_duration(&self) -> i32 {
        PLAY_DEAD_BEHAVIOR_DURATION
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.mob.get_entity().is_in_water()
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.mob.get_entity().is_in_water()
            && tick.state.has_memory_value(types::PLAY_DEAD_TICKS.id())
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.erase(types::WALK_TARGET.id());
        tick.state.erase(types::LOOK_TARGET.id());
        tick.mob.get_mob_entity().living_entity.add_effect(Effect {
            effect_type: &StatusEffect::REGENERATION,
            duration: PLAY_DEAD_REGENERATION_DURATION,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        });
    }

    fn debug_name(&self) -> &'static str {
        "PlayDead"
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use pumpkin_data::Block;
    use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
    use pumpkin_world::{chunk::ChunkData, world::BlockFlags};

    use super::*;
    use crate::entity::ai::brain::BrainState;
    use crate::entity::ai::brain::activity::ActivityData;
    use crate::entity::ai::brain::behavior::BehaviorControl;
    use crate::entity::ai::brain::behavior::Status;
    use crate::entity::ai::brain::memory::position_tracker::{
        BlockPosTracker, EntityTracker, PositionTracker,
    };
    use crate::entity::ai::brain::memory::walk_target::WalkTarget;
    use crate::entity::ai::test_support::world;
    use crate::entity::effect::get_mob_effect;
    use crate::entity::passive::cod::CodEntity;
    use crate::entity::{Entity, EntityBase};
    use crate::world::World;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn axolotl(world: &Arc<World>) -> Arc<AxolotlEntity> {
        AxolotlEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::AXOLOTL,
        ))
    }

    fn axolotl_at(world: &Arc<World>, x: f64) -> Arc<AxolotlEntity> {
        AxolotlEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, 0.0),
            &EntityType::AXOLOTL,
        ))
    }

    fn cod(world: &Arc<World>, x: f64) -> Arc<CodEntity> {
        let entity = Entity::new(world.clone(), Vector3::new(x, 64.0, 0.0), &EntityType::COD);
        entity.touching_water.store(true, Ordering::Relaxed);
        CodEntity::new(entity)
    }

    fn with_brain<R>(axolotl: &Arc<AxolotlEntity>, callback: impl FnOnce(&mut Brain) -> R) -> R {
        axolotl.mob_entity.with_brain(axolotl.as_ref(), callback)
    }

    /// The tracked activity after an activity update, and whether the hunting cooldown is set.
    fn update(axolotl: &Arc<AxolotlEntity>) -> (Option<Activity>, bool) {
        with_brain(axolotl, |brain| {
            update_activity(brain);
            (
                brain.get_active_non_core_activity(),
                brain.has_memory_value(types::HAS_HUNTING_COOLDOWN.id()),
            )
        })
    }

    /// A fight is what the target memory puts the axolotl in, and ending it starts the 2400 tick
    /// hunting cooldown as a real expiry; no fight, no cooldown.
    #[tokio::test]
    async fn a_finished_fight_starts_the_hunting_cooldown() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let prey: Arc<dyn EntityBase> = cod(&world, 2.0);

        let idle = update(&axolotl);
        with_brain(&axolotl, |brain| brain.set(types::ATTACK_TARGET, prey));
        let fighting = update(&axolotl);
        with_brain(&axolotl, |brain| brain.erase(types::ATTACK_TARGET.id()));
        let after = update(&axolotl);
        let cooldown = with_brain(&axolotl, |brain| {
            (
                brain.time_until_expiry(types::HAS_HUNTING_COOLDOWN.id()),
                brain.is_value(types::HAS_HUNTING_COOLDOWN, &true),
            )
        });
        world.level.shutdown().await;

        assert_eq!(idle, (Some(Activity::Idle), false));
        assert_eq!(fighting, (Some(Activity::Fight), false));
        assert_eq!(after, (Some(Activity::Idle), true));
        assert_eq!(cooldown, (Some(HUNTING_COOLDOWN), true));
        Ok(())
    }

    /// Playing dead outranks a fight (which ends it, so the cooldown starts), and while it lasts
    /// the activity is not touched; once the memory is gone the choice is made again.
    #[tokio::test]
    async fn playing_dead_outranks_a_fight_and_holds_the_activity() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let prey: Arc<dyn EntityBase> = cod(&world, 2.0);

        with_brain(&axolotl, |brain| brain.set(types::ATTACK_TARGET, prey));
        let fighting = update(&axolotl);
        with_brain(&axolotl, |brain| brain.set(types::PLAY_DEAD_TICKS, 150));
        let playing_dead = update(&axolotl);
        let held = update(&axolotl);
        with_brain(&axolotl, |brain| brain.erase(types::PLAY_DEAD_TICKS.id()));
        let released = update(&axolotl);
        world.level.shutdown().await;

        assert_eq!(fighting, (Some(Activity::Fight), false));
        assert_eq!(playing_dead, (Some(Activity::PlayDead), true));
        assert_eq!(held.0, Some(Activity::PlayDead));
        // Breeding erased the memory: the choice is made again and falls to idle (the fight had
        // already been replaced by playing dead, which started the cooldown and dropped the
        // attack target).
        assert_eq!(released, (Some(Activity::Idle), true));
        Ok(())
    }

    /// `ValidatePlayDead` alone: a tick off the memory per tick, and at 0 the memory and the
    /// attacker are forgotten and the default activity is back, on the tick after the count
    /// reached 0.
    #[tokio::test]
    async fn validate_play_dead_counts_down_and_ends_the_activity() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let attacker: Arc<dyn EntityBase> = cod(&world, 2.0);
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(validate_play_dead())],
        ));
        brain.add_activity(ActivityData::with_gate_memory(
            Activity::PlayDead,
            0,
            Vec::new(),
            types::PLAY_DEAD_TICKS.id(),
        ));
        brain.set(types::PLAY_DEAD_TICKS, 2);
        brain.set(types::HURT_BY_ENTITY, attacker);
        brain.set_active_activity_if_possible(Activity::PlayDead);
        let before = brain.get_active_non_core_activity();

        let mut trace = Vec::new();
        for time in 1..=3 {
            brain.tick(&world, axolotl.as_ref(), time);
            trace.push((
                brain.get(types::PLAY_DEAD_TICKS).copied(),
                brain.has_memory_value(types::HURT_BY_ENTITY.id()),
                brain.get_active_non_core_activity(),
            ));
        }
        world.level.shutdown().await;

        assert_eq!(before, Some(Activity::PlayDead));
        assert_eq!(
            trace,
            vec![
                (Some(1), true, Some(Activity::PlayDead)),
                (Some(0), true, Some(Activity::PlayDead)),
                (None, false, Some(Activity::Idle)),
            ]
        );
        Ok(())
    }

    /// The state `PlayDead` starts from: play dead ticks, an attacker, and somewhere it was going
    /// and looking.
    fn play_dead_state(attacker: &Arc<dyn EntityBase>) -> BrainState {
        let mut state = BrainState::default();
        for id in [
            types::PLAY_DEAD_TICKS.id(),
            types::HURT_BY_ENTITY.id(),
            types::WALK_TARGET.id(),
            types::LOOK_TARGET.id(),
        ] {
            state.register_memory(id);
        }
        state.set(types::PLAY_DEAD_TICKS, 200);
        state.set(types::HURT_BY_ENTITY, attacker.clone());
        state.set(
            types::WALK_TARGET,
            WalkTarget::from_entity(attacker.clone(), 1.0, 0),
        );
        state.set(
            types::LOOK_TARGET,
            Arc::new(EntityTracker::new(attacker.clone(), true)) as Arc<dyn PositionTracker>,
        );
        state
    }

    fn has_targets(state: &BrainState) -> (bool, bool) {
        (
            state.has_memory_value(types::WALK_TARGET.id()),
            state.has_memory_value(types::LOOK_TARGET.id()),
        )
    }

    /// `PlayDead` starts only in the water: it forgets where it was going and looking and gets
    /// 200 ticks of regeneration I, which heal 4 health over their duration. Out of the water it
    /// does not start and nothing is forgotten or given.
    #[tokio::test]
    async fn playing_dead_regenerates_in_the_water_only() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let wet = axolotl(&world);
        let dry = axolotl(&world);
        wet.get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);
        wet.mob_entity.living_entity.health.store(5.0);
        let attacker: Arc<dyn EntityBase> = cod(&world, 2.0);
        let (mut wet_state, mut dry_state) =
            (play_dead_state(&attacker), play_dead_state(&attacker));
        let (mut wet_behavior, mut dry_behavior) = (Timed::new(PlayDead), Timed::new(PlayDead));

        let wet_started = wet_behavior.try_start(&mut BrainTick {
            state: &mut wet_state,
            world: &world,
            mob: wet.as_ref(),
            time: 0,
        });
        let dry_started = dry_behavior.try_start(&mut BrainTick {
            state: &mut dry_state,
            world: &world,
            mob: dry.as_ref(),
            time: 0,
        });
        let wet_effect = wet
            .mob_entity
            .living_entity
            .get_effect(&StatusEffect::REGENERATION)
            .ok_or("no regeneration in the water")?;
        let dry_effect = dry
            .mob_entity
            .living_entity
            .get_effect(&StatusEffect::REGENERATION);

        // The heal the effect brings: its own tick rule over every tick of its duration.
        let mob_effect =
            get_mob_effect(wet_effect.effect_type).ok_or("no regeneration effect rule")?;
        for remaining in (1..=wet_effect.duration).rev() {
            if mob_effect.should_apply_effect_tick(remaining, wet_effect.amplifier) {
                mob_effect.apply_effect_tick(&wet.mob_entity.living_entity, wet_effect.amplifier);
            }
        }
        let healed = wet.mob_entity.living_entity.health.load();
        world.level.shutdown().await;

        assert!(wet_started);
        assert!(!dry_started);
        assert_eq!((wet_effect.duration, wet_effect.amplifier), (200, 0));
        assert!((healed - 9.0).abs() < 1e-6, "healed to {healed}");
        assert_eq!(has_targets(&wet_state), (false, false));
        assert!(dry_effect.is_none());
        assert_eq!(has_targets(&dry_state), (true, true));
        Ok(())
    }

    /// `PlayDead` lasts 200 ticks (it is still on at the 200th, off after it), and ends when the
    /// axolotl leaves the water or the play dead memory is gone.
    #[tokio::test]
    async fn playing_dead_ends_after_200_ticks_or_when_it_cannot_go_on() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        axolotl
            .get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);
        let attacker: Arc<dyn EntityBase> = cod(&world, 2.0);

        let run = |state: &mut BrainState, behavior: &mut Timed<PlayDead>, time: i64| {
            behavior.tick_or_stop(&mut BrainTick {
                state,
                world: &world,
                mob: axolotl.as_ref(),
                time,
            });
            behavior.status()
        };
        let start = |state: &mut BrainState| {
            let mut behavior = Timed::new(PlayDead);
            behavior.try_start(&mut BrainTick {
                state,
                world: &world,
                mob: axolotl.as_ref(),
                time: 0,
            });
            behavior
        };

        let mut state = play_dead_state(&attacker);
        let mut behavior = start(&mut state);
        let at_200 = run(&mut state, &mut behavior, 200);
        let at_201 = run(&mut state, &mut behavior, 201);

        let mut state = play_dead_state(&attacker);
        let mut behavior = start(&mut state);
        let in_water = run(&mut state, &mut behavior, 10);
        axolotl
            .get_entity()
            .touching_water
            .store(false, Ordering::Relaxed);
        let out_of_water = run(&mut state, &mut behavior, 11);
        axolotl
            .get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);

        let mut state = play_dead_state(&attacker);
        let mut behavior = start(&mut state);
        state.erase(types::PLAY_DEAD_TICKS.id());
        let forgotten = run(&mut state, &mut behavior, 10);
        world.level.shutdown().await;

        assert_eq!(
            (at_200, at_201, in_water, out_of_water, forgotten),
            (
                Status::Running,
                Status::Stopped,
                Status::Running,
                Status::Stopped,
                Status::Stopped
            )
        );
        Ok(())
    }

    /// The walk speeds of `AxolotlAi`: 0.5 idling in the water, 0.6 chasing and following an
    /// adult in the water, and 0.15 for everything on land.
    #[tokio::test]
    async fn the_walk_speeds_depend_on_the_water_and_the_task() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let speeds = |axolotl: &Arc<AxolotlEntity>| {
            (
                speed_modifier(axolotl.as_ref()),
                speed_modifier_chasing(axolotl.as_ref()),
                speed_modifier_following_adult(axolotl.as_ref()),
            )
        };

        axolotl
            .get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);
        let wet = speeds(&axolotl);
        axolotl
            .get_entity()
            .touching_water
            .store(false, Ordering::Relaxed);
        let dry = speeds(&axolotl);
        world.level.shutdown().await;

        assert_eq!(wet, (0.5, 0.6, 0.6));
        assert_eq!(dry, (0.15, 0.15, 0.15));
        Ok(())
    }

    /// The attack target `StartAttacking` is offered is what the sensor found, unless the
    /// axolotl is breeding.
    #[tokio::test]
    async fn a_breeding_axolotl_is_offered_no_attack_target() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let prey: Arc<dyn EntityBase> = cod(&world, 2.0);
        let partner: Arc<dyn EntityBase> = axolotl_at(&world, 1.0);
        let mut state = BrainState::default();
        state.register_memory(types::NEAREST_ATTACKABLE.id());
        state.register_memory(types::BREED_TARGET.id());
        state.set(types::NEAREST_ATTACKABLE, prey.clone());
        let offered = |state: &mut BrainState| {
            find_nearest_valid_attack_target(&BrainTick {
                state,
                world: &world,
                mob: axolotl.as_ref(),
                time: 0,
            })
            .map(|target| target.get_entity().entity_id)
        };

        let free = offered(&mut state);
        state.set(types::BREED_TARGET, partner);
        let breeding = offered(&mut state);
        world.level.shutdown().await;

        assert_eq!(free, Some(prey.get_entity().entity_id));
        assert_eq!(breeding, None);
        Ok(())
    }

    /// An axolotl walks to what it looked at only when that spot is water exactly when the
    /// axolotl is in the water itself; with nothing looked at, it does not walk.
    #[tokio::test]
    async fn a_look_target_is_walked_to_only_in_the_same_kind_of_place() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        world.set_block_state(
            &BlockPos::new(3, 64, 0),
            Block::WATER.default_state.id,
            BlockFlags::NOTIFY_LISTENERS | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
        );
        let wet = axolotl(&world);
        wet.get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);
        let dry = axolotl(&world);
        let look_at = |axolotl: &Arc<AxolotlEntity>, x: Option<i32>| {
            let mut state = BrainState::default();
            state.register_memory(types::LOOK_TARGET.id());
            if let Some(x) = x {
                state.set(
                    types::LOOK_TARGET,
                    Arc::new(BlockPosTracker::new(BlockPos::new(x, 64, 0)))
                        as Arc<dyn PositionTracker>,
                );
            }
            can_set_walk_target_from_look_target(&BrainTick {
                state: &mut state,
                world: &world,
                mob: axolotl.as_ref(),
                time: 0,
            })
        };

        let results = [
            look_at(&wet, Some(3)),
            look_at(&wet, Some(6)),
            look_at(&dry, Some(3)),
            look_at(&dry, Some(6)),
            look_at(&wet, None),
        ];
        world.level.shutdown().await;

        assert_eq!(results, [true, false, false, true, false]);
        Ok(())
    }

    /// The bite of the fight activity starts a 20 tick cooldown.
    #[tokio::test]
    async fn the_fight_bites_with_a_cooldown_of_20_ticks() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        axolotl
            .get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);
        let prey = cod(&world, 1.0);
        let everyone: Vec<Arc<dyn EntityBase>> = vec![axolotl.clone(), prey];
        world.entities.store(Arc::new(everyone));

        let mut cooldown = None;
        for time in 0..80 {
            with_brain(&axolotl, |brain| {
                brain.tick(&world, axolotl.as_ref(), time);
                update_activity(brain);
                if cooldown.is_none() && brain.has_memory_value(types::ATTACK_COOLING_DOWN.id()) {
                    cooldown = brain.time_until_expiry(types::ATTACK_COOLING_DOWN.id());
                }
            });
        }
        world.level.shutdown().await;

        assert_eq!(cooldown, Some(20));
        Ok(())
    }
}
