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
}

impl ShootTongue {
    pub const fn new(tongue_sound: Sound, eat_sound: Sound) -> Self {
        Self {
            tongue_sound,
            eat_sound,
            eat_animation_timer: 0,
            calculate_path_counter: 0,
            state: State::Done,
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

    fn eat_entity(&self, tick: &BrainTick<'_>, frog: &FrogEntity) {
        tick.world.play_sound_fine(
            self.eat_sound,
            SoundCategory::Neutral,
            &frog.get_entity().pos.load(),
            2.0,
            1.0,
        );
        let Some(target) = frog.get_tongue_target() else {
            return;
        };
        if !is_alive(target.as_ref()) {
            return;
        }
        tick.mob.do_hurt_target(target.as_ref());
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
        tick.state.has_memory_value(types::ATTACK_TARGET.id())
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
        self.calculate_path_counter = REPATH_INTERVAL;
        self.state = State::MoveToTarget;
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let (Some(target), Some(frog)) = (attack_target(tick), frog(tick)) else {
            return;
        };
        frog.set_tongue_target(target.as_ref());
        match self.state {
            State::MoveToTarget => {
                let frog_pos = frog.get_entity().pos.load();
                let target_pos = target.get_entity().pos.load();
                if target_pos.squared_distance_to_vec(&frog_pos).sqrt() < EATING_DISTANCE {
                    tick.world.play_sound_fine(
                        self.tongue_sound,
                        SoundCategory::Neutral,
                        &frog_pos,
                        2.0,
                        1.0,
                    );
                    frog.get_entity().set_pose(EntityPose::UsingTongue);
                    target
                        .get_entity()
                        .set_velocity((frog_pos - target_pos).normalize() * EATING_MOVEMENT_FACTOR);
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
                    self.eat_entity(tick, frog);
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

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.erase(types::ATTACK_TARGET.id());
        if let Some(frog) = frog(tick) {
            frog.erase_tongue_target();
            frog.get_entity().set_pose(EntityPose::Standing);
        }
    }

    fn debug_name(&self) -> &'static str {
        "ShootTongue"
    }
}
