use std::sync::Arc;

use pumpkin_data::entity::EntityPose;
use pumpkin_data::sound::{Sound, SoundCategory};

use super::FrogEntity;
use crate::entity::EntityBase;
use crate::entity::ai::brain::BrainTick;
use crate::entity::ai::brain::behavior::{Behavior, utils};
use crate::entity::ai::brain::memory::{
    MemoryModuleId, MemoryStatus, types, walk_target::WalkTarget,
};
use crate::entity::mob::Mob;
use crate::world::World;

const TIME_OUT_DURATION: i32 = 100;
const CATCH_ANIMATION_DURATION: i32 = 6;
const TONGUE_ANIMATION_DURATION: i32 = 10;
const EATING_DISTANCE: f64 = 1.75;
const EATING_MOVEMENT_FACTOR: f64 = 0.75;
const UNREACHABLE_TONGUE_TARGETS_COOLDOWN_DURATION: i64 = 100;
const MAX_UNREACHABLE_TONGUE_TARGETS_IN_MEMORY: usize = 5;
const WALK_SPEED_MODIFIER: f32 = 2.0;
const REPATH_INTERVAL: i32 = 10;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    MoveToTarget,
    CatchAnimation,
    EatAnimation,
    Done,
}

/// Walks up to an edible target, pulls it in with the tongue and eats it.
pub struct ShootTongue {
    tongue_sound: Sound,
    eat_sound: Sound,
    eat_animation_timer: i32,
    calculate_path_counter: i32,
    state: State,
    target_id: Option<i32>,
}

impl ShootTongue {
    pub const fn new(tongue_sound: Sound, eat_sound: Sound) -> Self {
        Self {
            tongue_sound,
            eat_sound,
            eat_animation_timer: 0,
            calculate_path_counter: 0,
            state: State::Done,
            target_id: None,
        }
    }

    fn can_pathfind_to_target(tick: &BrainTick<'_>, target: &dyn EntityBase) -> bool {
        let mob_entity = tick.mob.get_mob_entity();
        mob_entity
            .steered()
            .navigator()
            .create_path(
                &mob_entity.living_entity,
                target.get_entity().block_pos.load().0.to_f64(),
                0,
            )
            .is_some_and(|path| f64::from(path.get_dist_to_target()) < EATING_DISTANCE)
    }

    fn add_unreachable_target_to_memory(tick: &mut BrainTick<'_>, target: &dyn EntityBase) {
        let uuid = target.get_entity().entity_uuid;
        let mut targets = tick
            .state
            .get(types::UNREACHABLE_TONGUE_TARGETS)
            .cloned()
            .unwrap_or_default();
        let should_add = !targets.contains(&uuid);
        if targets.len() == MAX_UNREACHABLE_TONGUE_TARGETS_IN_MEMORY && should_add {
            targets.remove(0);
        }
        if should_add {
            targets.push(uuid);
        }
        tick.state.set_with_expiry(
            types::UNREACHABLE_TONGUE_TARGETS,
            targets,
            UNREACHABLE_TONGUE_TARGETS_COOLDOWN_DURATION,
        );
    }

    fn eat_entity(mob: &dyn Mob, target: &Arc<dyn EntityBase>, world: &Arc<World>) {
        let Some(frog) = mob.cast_any().downcast_ref::<FrogEntity>() else {
            return;
        };
        if !is_alive(target.as_ref()) || !tongue_target_is_current(frog, target, world) {
            return;
        }
        mob.do_hurt_target(target.as_ref());
        if !is_alive(target.as_ref()) {
            target.get_entity().remove();
        }
    }

    fn set_walk_target(tick: &mut BrainTick<'_>, target: &dyn EntityBase) {
        tick.state.set(
            types::WALK_TARGET,
            WalkTarget::from_vec(target.get_entity().pos.load(), WALK_SPEED_MODIFIER, 0),
        );
    }
}

fn tongue_target_is_current(
    frog: &FrogEntity,
    target: &Arc<dyn EntityBase>,
    world: &Arc<World>,
) -> bool {
    // The effect runner checks the frog. A listener can still move or remove the prey.
    !target.get_entity().is_removed()
        && Arc::ptr_eq(&target.get_entity().world.load(), world)
        && frog
            .get_mob_entity()
            .stored_target()
            .is_some_and(|current| Arc::ptr_eq(&current, target))
        && frog
            .get_tongue_target()
            .is_some_and(|current| Arc::ptr_eq(&current, target))
}

fn is_alive(entity: &dyn EntityBase) -> bool {
    entity.get_living_entity().map_or_else(
        || entity.get_entity().is_alive(),
        crate::entity::living::LivingEntity::is_alive,
    )
}

fn frog<'a>(tick: &BrainTick<'a>) -> Option<&'a FrogEntity> {
    tick.mob.cast_any().downcast_ref::<FrogEntity>()
}

fn attack_target(tick: &BrainTick<'_>) -> Option<Arc<dyn EntityBase>> {
    tick.state.get(types::ATTACK_TARGET).cloned()
}

impl Behavior for ShootTongue {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const CONDITIONS: [(MemoryModuleId, MemoryStatus); 4] = [
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
        ];
        &CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        TIME_OUT_DURATION
    }

    fn max_duration(&self) -> i32 {
        TIME_OUT_DURATION
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some(target) = attack_target(tick) else {
            return false;
        };
        let reachable = Self::can_pathfind_to_target(tick, target.as_ref());
        if !reachable {
            tick.state.erase(types::ATTACK_TARGET.id());
            Self::add_unreachable_target_to_memory(tick, target.as_ref());
        }
        reachable
            && tick.mob.get_entity().pose.load() != EntityPose::Croaking
            && super::can_eat(target.as_ref())
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        attack_target(tick)
            .is_some_and(|target| self.target_id == Some(target.get_entity().entity_id))
            && self.state != State::Done
            && !tick.state.has_memory_value(types::IS_PANICKING.id())
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let (Some(target), Some(frog)) = (attack_target(tick), frog(tick)) else {
            return;
        };
        utils::look_at_entity(tick.state, target.clone());
        frog.set_tongue_target(target.as_ref());
        Self::set_walk_target(tick, target.as_ref());
        self.target_id = Some(target.get_entity().entity_id);
        self.calculate_path_counter = REPATH_INTERVAL;
        self.state = State::MoveToTarget;
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let (Some(target), Some(frog)) = (attack_target(tick), frog(tick)) else {
            return;
        };
        if self.target_id != Some(target.get_entity().entity_id) {
            return;
        }
        frog.set_tongue_target(target.as_ref());
        match self.state {
            State::MoveToTarget => {
                let frog_pos = frog.get_entity().pos.load();
                let target_pos = target.get_entity().pos.load();
                if target_pos.squared_distance_to_vec(&frog_pos).sqrt() < EATING_DISTANCE {
                    let world = tick.world.clone();
                    let sound_world = world.clone();
                    let sound = self.tongue_sound;
                    tick.defer(move |_| {
                        sound_world.play_sound_fine(
                            sound,
                            SoundCategory::Neutral,
                            &frog_pos,
                            2.0,
                            1.0,
                        );
                    });
                    tick.set_pose(EntityPose::UsingTongue);
                    tick.defer(move |mob| {
                        let Some(frog) = mob.cast_any().downcast_ref::<FrogEntity>() else {
                            return;
                        };
                        if !tongue_target_is_current(frog, &target, &world) {
                            return;
                        }
                        let prey = target.get_entity();
                        let offset = frog.get_entity().pos.load() - prey.pos.load();
                        prey.set_velocity(offset.normalize() * EATING_MOVEMENT_FACTOR);
                    });
                    self.eat_animation_timer = 0;
                    self.state = State::CatchAnimation;
                } else if self.calculate_path_counter <= 0 {
                    Self::set_walk_target(tick, target.as_ref());
                    self.calculate_path_counter = REPATH_INTERVAL;
                } else {
                    self.calculate_path_counter -= 1;
                }
            }
            State::CatchAnimation => {
                let elapsed = self.eat_animation_timer;
                self.eat_animation_timer += 1;
                if elapsed >= CATCH_ANIMATION_DURATION {
                    self.state = State::EatAnimation;
                    let world = tick.world.clone();
                    let sound_world = world.clone();
                    let sound = self.eat_sound;
                    tick.defer(move |mob| {
                        sound_world.play_sound_fine(
                            sound,
                            SoundCategory::Neutral,
                            &mob.get_entity().pos.load(),
                            2.0,
                            1.0,
                        );
                    });
                    tick.defer(move |mob| Self::eat_entity(mob, &target, &world));
                }
            }
            State::EatAnimation => {
                if self.eat_animation_timer >= TONGUE_ANIMATION_DURATION {
                    self.state = State::Done;
                } else {
                    self.eat_animation_timer += 1;
                }
            }
            State::Done => {}
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>, _timed_out: bool) {
        let previous = self.target_id.take();
        if attack_target(tick).is_some_and(|target| previous == Some(target.get_entity().entity_id))
        {
            tick.state.erase(types::ATTACK_TARGET.id());
        }
        if let Some(frog) = frog(tick) {
            frog.erase_tongue_target();
            tick.set_pose(EntityPose::Standing);
        }
    }

    fn debug_name(&self) -> &'static str {
        "ShootTongue"
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

    use pumpkin_data::dimension::Dimension;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_data::packet::clientbound::play::SOUND;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::{
        Entity,
        ai::brain::{Brain, activity::ActivityData, behavior::OneShot},
        ai::test_support::{java_sound_recipient, server},
        passive::cow::CowEntity,
    };
    use crate::plugin::{
        BoxFuture, EventRwHandler, api::events::EventPriority, server::packet::PacketSentEvent,
    };
    use crate::server::Server;

    struct TongueSoundListener {
        frog: Arc<FrogEntity>,
        replace_brain: AtomicBool,
        clear_target: AtomicBool,
        prey: Arc<CowEntity>,
        observations: Mutex<Vec<(EntityPose, f32)>>,
    }

    impl EventRwHandler<PacketSentEvent> for TongueSoundListener {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == SOUND.0 {
                    let mut observations = self
                        .observations
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if !observations.is_empty() {
                        return;
                    }
                    observations.push((
                        self.frog.get_entity().pose.load(),
                        self.prey.mob_entity.living_entity.health.load(),
                    ));
                    drop(observations);
                    if self.replace_brain.load(Relaxed) {
                        self.frog.get_mob_entity().set_brain(Brain::default());
                    } else if self.clear_target.load(Relaxed) {
                        self.frog.get_mob_entity().set_target(None);
                    }
                }
            })
        }
    }

    #[tokio::test]
    async fn tongue_interrupt_preserves_a_redirected_target()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let origin = Vector3::new(0.0, 64.0, 0.0);
        let frog = FrogEntity::new(Entity::new(world.clone(), origin, &EntityType::FROG));
        let prey = CowEntity::new(Entity::new(
            world.clone(),
            origin + Vector3::new(1.0, 0.0, 0.0),
            &EntityType::COW,
        ));
        let replacement = CowEntity::new(Entity::new(
            world.clone(),
            origin + Vector3::new(12.0, 0.0, 0.0),
            &EntityType::COW,
        ));
        let old_id = prey.get_entity().entity_id;
        let next_id = replacement.get_entity().entity_id;
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(OneShot::new(
                "InterruptTongue",
                vec![(types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent)],
                move |tick| {
                    let mut tongue =
                        ShootTongue::new(Sound::EntityFrogTongue, Sound::EntityFrogEat);
                    tongue.state = State::CatchAnimation;
                    tongue.target_id = Some(old_id);
                    assert!(!tongue.can_still_use(tick));
                    tongue.stop(tick, false);
                    true
                },
            ))],
        ));
        brain.set(types::ATTACK_TARGET, replacement);
        frog.get_entity().set_pose(EntityPose::UsingTongue);
        frog.set_tongue_target(prey.as_ref());
        let body = frog.get_mob_entity();
        body.set_brain(brain);
        body.tick_brain(frog.as_ref());
        assert_eq!(
            body.stored_target()
                .map(|target| target.get_entity().entity_id),
            Some(next_id)
        );
        assert_eq!(frog.get_entity().pose.load(), EntityPose::Standing);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn tongue_sounds_precede_and_guard_follow_up_effects()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let (player, _peer) = java_sound_recipient(&server, &world, "TongueTest")?;
        let origin = player.get_entity().pos.load();
        let frog = FrogEntity::new(Entity::new(world.clone(), origin, &EntityType::FROG));
        let prey = CowEntity::new(Entity::new(
            world.clone(),
            origin + Vector3::new(1.0, 0.0, 0.0),
            &EntityType::COW,
        ));
        world.add_entity_silent(prey.clone());

        let listener = Arc::new(TongueSoundListener {
            frog: frog.clone(),
            replace_brain: AtomicBool::new(false),
            clear_target: AtomicBool::new(false),
            prey: prey.clone(),
            observations: Mutex::new(Vec::new()),
        });
        server
            .plugin_manager
            .register_blocking(listener.clone(), EventPriority::Normal, None);
        let body = frog.get_mob_entity();
        let health = &prey.mob_entity.living_entity.health;
        let full_health = health.load();
        for (state, replace, clear, initial_health) in [
            (State::MoveToTarget, false, false, full_health),
            (State::MoveToTarget, true, false, full_health),
            (State::MoveToTarget, false, true, full_health),
            (State::CatchAnimation, true, false, full_health),
            (State::CatchAnimation, false, true, full_health),
            (State::CatchAnimation, false, false, 0.0),
            (State::CatchAnimation, false, false, full_health),
        ] {
            let mut tongue = ShootTongue::new(Sound::EntityFrogTongue, Sound::EntityFrogEat);
            // Enter the relevant tick directly so this fixture need not pathfind.
            tongue.state = state;
            tongue.target_id = Some(prey.get_entity().entity_id);
            tongue.eat_animation_timer = CATCH_ANIMATION_DURATION;
            let mut brain = Brain::default();
            brain.add_activity(ActivityData::with_priority_start(
                Activity::Core,
                0,
                vec![Box::new(OneShot::new(
                    "CatchPrey",
                    vec![(types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent)],
                    move |tick| {
                        tongue.tick(tick);
                        true
                    },
                ))],
            ));
            brain.set(types::ATTACK_TARGET, prey.clone());
            let initial_pose = if state == State::MoveToTarget {
                EntityPose::Standing
            } else {
                EntityPose::UsingTongue
            };
            frog.get_entity().set_pose(initial_pose);
            health.store(initial_health);
            prey.get_entity().velocity.store(Vector3::default());
            listener.replace_brain.store(replace, Relaxed);
            listener.clear_target.store(clear, Relaxed);
            body.set_brain(brain);
            body.tick_brain(frog.as_ref());

            let observations = std::mem::take(
                &mut *listener
                    .observations
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
            assert_eq!(observations, [(initial_pose, initial_health)]);

            let attempted_catch = state == State::MoveToTarget && !replace;
            let pose = if attempted_catch {
                EntityPose::UsingTongue
            } else {
                initial_pose
            };
            let velocity = if attempted_catch && !clear {
                Vector3::new(-EATING_MOVEMENT_FACTOR, 0.0, 0.0)
            } else {
                Vector3::default()
            };
            assert_eq!(frog.get_entity().pose.load(), pose);
            if state == State::CatchAnimation && !replace && !clear && initial_health > 0.0 {
                assert!(health.load() < initial_health);
            } else {
                assert_eq!(prey.get_entity().velocity.load(), velocity);
                assert_eq!(health.load().to_bits(), initial_health.to_bits());
            }
        }
        server.shutdown().await;
        Ok(())
    }
}
