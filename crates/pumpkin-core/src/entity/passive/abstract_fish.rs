use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

use crate::entity::{
    EntityBase,
    ai::{
        control::fish_move_control::FishMoveControl,
        goal::{Controls, Goal, random_swimming::RandomSwimmingGoal},
        pathfinder::{Navigator, node::PathType},
    },
    mob::{Mob, MobEntity},
};
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{boundingbox::EntityDimensions, vector3::Vector3};
use rand::RngExt;

use super::water_animal::WaterAnimal;

#[derive(Default)]
pub struct FishData {
    pub from_bucket: AtomicBool,
}

const WATER_ACCELERATION: f32 = 0.01;
const WATER_FRICTION: f64 = 0.9;
const SINK_SPEED: f64 = 0.005;
const FLOP_HORIZONTAL: f32 = 0.05;
const FLOP_VERTICAL: f32 = 0.4;

pub fn initialize_fish_movement(mob: &mut MobEntity) {
    let navigator = mob
        .navigator
        .get_mut()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *navigator = Navigator::water_bound(false);
    navigator.set_pathfinding_malus(PathType::Water, 0.0);
    *mob.move_control
        .get_mut()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Box::new(FishMoveControl::default());
}

const PANIC_SPEED: f64 = 1.25;
const RANDOM_SWIM_SPEED: f64 = 1.0;
const RANDOM_SWIM_INTERVAL: i32 = 40;

pub fn register_fish_goals(mob: &MobEntity) {
    use crate::entity::ai::{
        behavior::fleeing::{self, FISH_FLEES},
        goal::escape_danger::EscapeDangerGoal,
    };
    let goals = &mob.goals_selector;
    goals.add_goal(0, EscapeDangerGoal::new(PANIC_SPEED));
    fleeing::apply(goals, FISH_FLEES);
    goals.add_goal(4, Box::new(FishSwimGoal::default()));
}

struct FishSwimGoal {
    inner: RandomSwimmingGoal,
}

impl Default for FishSwimGoal {
    fn default() -> Self {
        Self {
            inner: RandomSwimmingGoal::new(RANDOM_SWIM_SPEED, RANDOM_SWIM_INTERVAL),
        }
    }
}

impl Goal for FishSwimGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        mob.as_fish().is_some_and(AbstractFish::can_random_swim) && self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

pub trait AbstractFish: WaterAnimal + crate::entity::bucketable::Bucketable {
    fn flop_sound(&self) -> Sound;

    fn can_random_swim(&self) -> bool {
        self.as_schooling_fish()
            .is_none_or(|fish| !fish.is_follower())
    }

    fn fish_travel_in_water(&self, caller: &dyn EntityBase) {
        let mob = self.get_mob_entity();
        let living = &mob.living_entity;
        let entity = &living.entity;
        entity.update_velocity_from_input(
            living.movement_input.load(),
            f64::from(WATER_ACCELERATION),
        );
        living.make_move(caller);
        let mut velocity = entity.velocity.load() * WATER_FRICTION;
        if mob
            .target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none()
        {
            velocity.y -= SINK_SPEED;
        }
        entity.velocity.store(velocity);
    }

    fn fish_ai_step(&self) {
        let entity = self.get_entity();
        if !entity.is_in_water()
            && entity.on_ground.load(Relaxed)
            && entity.vertical_collision.load(Relaxed)
        {
            let mut rng = self.get_random();
            entity.velocity.store(
                entity.velocity.load()
                    + Vector3::new(
                        f64::from((rng.random::<f32>() * 2.0 - 1.0) * FLOP_HORIZONTAL),
                        f64::from(FLOP_VERTICAL),
                        f64::from((rng.random::<f32>() * 2.0 - 1.0) * FLOP_HORIZONTAL),
                    ),
            );
            entity.on_ground.store(false, Relaxed);
            entity.velocity_dirty.store(true, Relaxed);
            self.make_sound(self.flop_sound());
        }
    }

    fn fish_data(&self) -> &FishData;

    fn ambient_sound(&self) -> Option<Sound>;

    fn fish_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("FromBucket", self.is_from_bucket());
    }

    fn fish_read_nbt(&self, nbt: &NbtCompound) {
        self.set_from_bucket(nbt.get_bool("FromBucket").unwrap_or(false));
    }

    fn set_dimension_scale(&self, scale: f32) {
        let entity = self.get_entity();
        let dimensions = EntityDimensions::new(
            entity.entity_type.dimension[0] * scale,
            entity.entity_type.dimension[1] * scale,
            entity.entity_type.eye_height * scale,
        );
        entity.set_default_dimensions(dimensions);
    }

    fn fish_base_tick(&self, caller: &dyn crate::entity::EntityBase) {
        self.handle_air_supply(self.get_entity().air_supply.load(Relaxed), caller);
    }

    fn fish_remove_when_far_away(&self) -> bool {
        !self.is_from_bucket() && self.get_entity().custom_name.load().is_none()
    }

    fn fish_requires_custom_persistence(&self) -> bool {
        let entity = self.get_entity();
        self.is_from_bucket()
            || entity
                .vehicle
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
            || entity
                .leashed_to
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
    }
}

/// Generates the trait impls every fish repeats. The type needs `mob_entity` and `fish_data` fields.
#[macro_export]
macro_rules! impl_abstract_fish {
    ($fish:ty, bucket: $bucket:expr, flop: $flop:expr, ambient: $ambient:expr) => {
        $crate::impl_abstract_fish!($fish, bucket: $bucket, flop: $flop, ambient: $ambient, bucket_extra: {});
    };
    ($fish:ty, bucket: $bucket:expr, flop: $flop:expr, ambient: $ambient:expr, bucket_extra: { $($extra:tt)* }) => {
        impl $crate::entity::passive::water_animal::WaterAnimal for $fish {}

        impl $crate::entity::passive::abstract_fish::AbstractFish for $fish {
            fn flop_sound(&self) -> pumpkin_data::sound::Sound {
                $flop
            }

            fn fish_data(&self) -> &$crate::entity::passive::abstract_fish::FishData {
                &self.fish_data
            }

            fn ambient_sound(&self) -> Option<pumpkin_data::sound::Sound> {
                $ambient
            }
        }

        impl $crate::entity::custom_sound::CustomSound for $fish {
            fn death_sound(&self) -> Option<pumpkin_data::sound::Sound> {
                $crate::entity::EntityBase::get_entity(self).entity_type.death_sound
            }

            fn hurt_sound(&self) -> Option<pumpkin_data::sound::Sound> {
                $crate::entity::EntityBase::get_entity(self).entity_type.hurt_sound
            }

            fn sound_category(&self) -> pumpkin_data::sound::SoundCategory {
                pumpkin_data::sound::SoundCategory::Neutral
            }
        }

        impl $crate::entity::bucketable::Bucketable for $fish {
            fn is_from_bucket(&self) -> bool {
                self.fish_data
                    .from_bucket
                    .load(std::sync::atomic::Ordering::Relaxed)
            }

            fn set_from_bucket(&self, from_bucket: bool) {
                self.fish_data
                    .from_bucket
                    .store(from_bucket, std::sync::atomic::Ordering::Relaxed);
                $crate::entity::EntityBase::get_entity(self).set_synced_data(
                    pumpkin_data::tracked_data::abstract_fish::FROM_BUCKET,
                    from_bucket,
                );
            }

            fn bucket_item(&self) -> &'static pumpkin_data::item::Item {
                $bucket
            }

            fn pickup_sound(&self) -> pumpkin_data::sound::Sound {
                pumpkin_data::sound::Sound::ItemBucketFillFish
            }

            $($extra)*
        }
    };
}

/// Generates the `Mob` methods every fish shares. Use inside `impl Mob for ...`.
#[macro_export]
macro_rules! fish_mob_hooks {
    () => {
        /// Vanilla `AbstractFish` steps silently.
        fn mob_play_step_sound(
            &self,
            _pos: &pumpkin_util::math::position::BlockPos,
            _state: &pumpkin_data::BlockState,
        ) {
        }

        fn get_mob_entity(&self) -> &$crate::entity::mob::MobEntity {
            &self.mob_entity
        }

        fn as_fish(&self) -> Option<&dyn $crate::entity::passive::abstract_fish::AbstractFish> {
            Some(self)
        }

        fn as_bucketable(&self) -> Option<&dyn $crate::entity::bucketable::Bucketable> {
            Some(self)
        }

        fn as_custom_sound(&self) -> Option<&dyn $crate::entity::custom_sound::CustomSound> {
            Some(self)
        }

        fn get_ambient_sound(&self) -> Option<pumpkin_data::sound::Sound> {
            $crate::entity::passive::abstract_fish::AbstractFish::ambient_sound(self)
        }

        fn get_ambient_sound_interval(&self) -> i32 {
            $crate::entity::passive::water_animal::AMBIENT_SOUND_INTERVAL
        }

        fn mob_interact_with_hand(
            &self,
            player: &std::sync::Arc<$crate::entity::player::Player>,
            stack: &mut pumpkin_data::item_stack::ItemStack,
            _hand: pumpkin_util::Hand,
        ) -> Option<bool> {
            $crate::entity::bucketable::bucket_mob_pickup(player, stack, self)
        }

        fn before_mob_ai_step(&self) {
            $crate::entity::passive::abstract_fish::AbstractFish::fish_ai_step(self);
        }

        fn travel_in_water(
            &self,
            caller: &dyn $crate::entity::EntityBase,
            _gravity: f64,
            _falling: bool,
        ) {
            $crate::entity::passive::abstract_fish::AbstractFish::fish_travel_in_water(
                self, caller,
            );
        }

        fn base_tick(&self) {
            $crate::entity::passive::abstract_fish::AbstractFish::fish_base_tick(self, self);
        }

        fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
            $crate::entity::passive::abstract_fish::AbstractFish::fish_remove_when_far_away(self)
        }

        fn requires_custom_persistence(&self) -> bool {
            $crate::entity::passive::abstract_fish::AbstractFish::fish_requires_custom_persistence(
                self,
            )
        }

        fn check_spawn_obstruction(&self, world: &$crate::world::World) -> bool {
            $crate::entity::passive::water_animal::WaterAnimal::water_animal_spawn_obstruction(
                self, world,
            )
        }

        fn get_base_experience_reward(&self) -> u32 {
            $crate::entity::passive::water_animal::WaterAnimal::water_animal_experience_reward(self)
        }

        fn mob_is_pushed_by_fluids(&self) -> bool {
            false
        }

        fn can_be_leashed(&self) -> bool {
            false
        }
    };
}

/// Generates the `Mob` methods schooling fish add on top of `fish_mob_hooks!`.
#[macro_export]
macro_rules! schooling_fish_mob_hooks {
    () => {
        fn as_schooling_fish(
            &self,
        ) -> Option<&dyn $crate::entity::passive::abstract_schooling_fish::AbstractSchoolingFish> {
            Some(self)
        }

        fn post_tick(&self) {
            $crate::entity::passive::abstract_schooling_fish::AbstractSchoolingFish::schooling_tick(
                self,
            );
        }
    };
}
