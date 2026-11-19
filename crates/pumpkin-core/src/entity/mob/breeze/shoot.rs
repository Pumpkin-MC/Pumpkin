use std::sync::Arc;

use pumpkin_data::{
    effect::StatusEffect,
    entity::{EntityPose, EntityType},
    item_stack::ItemStack,
    sound::Sound,
};
use pumpkin_util::math::vector3::Vector3;

use super::look_from_eyes;
use crate::entity::{
    Entity, EntityBase,
    ai::brain::{
        BrainTick,
        behavior::{Behavior, BehaviorControl, Timed},
        memory::{MemoryModuleId, MemoryStatus, types},
    },
    projectile::{
        ThrownItemEntity, apply_on_projectile_spawned,
        wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity},
    },
};

const ATTACK_RANGE_MAX_SQRT: f64 = 256.0;
const UNCERTAINTY_BASE: i32 = 5;
const UNCERTAINTY_MULTIPLIER: i32 = 4;
const PROJECTILE_MOVEMENT_SCALE: f32 = 0.7;
const SHOOT_INITIAL_DELAY_TICKS: i32 = 15;
const SHOOT_RECOVER_DELAY_TICKS: i32 = 4;
const SHOOT_COOLDOWN_TICKS: i64 = 10;
const SHOOT_DURATION_TICKS: i32 = SHOOT_INITIAL_DELAY_TICKS + 1 + SHOOT_RECOVER_DELAY_TICKS;
pub(super) const SHOOT_VOLUME: f32 = 1.5;

pub(super) fn shoot() -> Box<dyn BehaviorControl> {
    Box::new(Timed::new(Shoot))
}

pub(super) fn shoot_when_stuck() -> Box<dyn BehaviorControl> {
    Box::new(Timed::new(ShootWhenStuck))
}

struct Shoot;

impl Behavior for Shoot {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const CONDITIONS: [(MemoryModuleId, MemoryStatus); 7] = [
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (types::BREEZE_SHOOT_COOLDOWN.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_SHOOT_CHARGING.id(), MemoryStatus::ValueAbsent),
            (
                types::BREEZE_SHOOT_RECOVERING.id(),
                MemoryStatus::ValueAbsent,
            ),
            (types::BREEZE_SHOOT.id(), MemoryStatus::ValuePresent),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_JUMP_TARGET.id(), MemoryStatus::ValueAbsent),
        ];
        &CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        SHOOT_DURATION_TICKS
    }

    fn max_duration(&self) -> i32 {
        SHOOT_DURATION_TICKS
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let breeze = tick.mob.get_entity();
        if tick.pose() != EntityPose::Standing {
            return false;
        }

        let Some(target) = tick.state.get(types::ATTACK_TARGET) else {
            return false;
        };
        let distance = breeze
            .pos
            .load()
            .squared_distance_to_vec(&target.get_entity().pos.load());
        let in_range = distance < ATTACK_RANGE_MAX_SQRT;
        if !in_range {
            tick.state.erase(types::BREEZE_SHOOT.id());
        }
        in_range
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.state.has_memory_value(types::ATTACK_TARGET.id())
            && tick.state.has_memory_value(types::BREEZE_SHOOT.id())
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.set_with_expiry(
            types::BREEZE_SHOOT_CHARGING,
            (),
            i64::from(SHOOT_INITIAL_DELAY_TICKS),
        );
        tick.set_pose_then(EntityPose::Shooting, |breeze| {
            breeze.play_entity_sound(Sound::EntityBreezeInhale, 1.0, 1.0);
        });
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target) = tick.state.get(types::ATTACK_TARGET) else {
            return;
        };
        let target_entity = target.get_entity();
        let target_position = target_entity.pos.load();
        let breeze = tick.mob.get_entity();
        look_from_eyes(breeze, target_position);

        if tick
            .state
            .has_memory_value(types::BREEZE_SHOOT_CHARGING.id())
            || tick
                .state
                .has_memory_value(types::BREEZE_SHOOT_RECOVERING.id())
        {
            return;
        }

        let aim_fraction = if target.is_passenger() { 0.8 } else { 0.3 };
        let aim_at =
            target_position.add_raw(0.0, f64::from(target_entity.height()) * aim_fraction, 0.0);
        let target = target.clone();
        tick.state.set_with_expiry(
            types::BREEZE_SHOOT_RECOVERING,
            (),
            i64::from(SHOOT_RECOVER_DELAY_TICKS),
        );
        fire_wind_charge(tick, target, aim_at);
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        if tick.pose() == EntityPose::Shooting {
            tick.set_pose(EntityPose::Standing);
        }
        tick.state
            .set_with_expiry(types::BREEZE_SHOOT_COOLDOWN, (), SHOOT_COOLDOWN_TICKS);
        tick.state.erase(types::BREEZE_SHOOT.id());
    }

    fn debug_name(&self) -> &'static str {
        "Shoot"
    }
}

fn fire_wind_charge(tick: &mut BrainTick<'_>, target: Arc<dyn EntityBase>, aim_at: Vector3<f64>) {
    let breeze = tick.mob.get_entity();
    let position = breeze.pos.load();
    let firing_position = position.add_raw(
        0.0,
        f64::from(breeze.height() / 2.0) + f64::from(0.3f32),
        0.0,
    );

    let direction = aim_at - firing_position;
    let difficulty = i32::from(u8::from(tick.world.level_info.load().difficulty));
    let projectile = Entity::new(
        tick.world.clone(),
        firing_position,
        &EntityType::BREEZE_WIND_CHARGE,
    );
    let thrown = ThrownItemEntity::owned(projectile, breeze, WIND_CHARGE_GRAVITY);
    thrown.set_velocity(
        direction.x,
        direction.y,
        direction.z,
        f64::from(PROJECTILE_MOVEMENT_SCALE),
        f64::from(UNCERTAINTY_BASE - difficulty * UNCERTAINTY_MULTIPLIER),
    );

    tick.defer(move |shooter| {
        let charge = Arc::new(WindChargeEntity::new_breeze(thrown));
        let world = charge.get_entity().world.load_full();
        let entity = shooter.get_entity();
        let target_entity = target.get_entity();
        let body = shooter.get_mob_entity();
        let shot_is_current = || {
            body.living_entity.is_alive()
                && entity.pos.load() == position
                && Arc::ptr_eq(&entity.world.load(), &world)
                && target
                    .get_living_entity()
                    .is_some_and(crate::entity::living::LivingEntity::is_alive)
                && Arc::ptr_eq(&target_entity.world.load(), &world)
                && body.with_brain(shooter, |brain| {
                    brain.has_memory_value(types::BREEZE_SHOOT.id())
                        && brain
                            .get(types::ATTACK_TARGET)
                            .is_some_and(|current| Arc::ptr_eq(current, &target))
                })
        };
        if !shot_is_current() {
            return;
        }
        if world.spawn_entity_if(charge.clone(), shot_is_current) {
            apply_on_projectile_spawned(charge.get_entity(), ItemStack::EMPTY, None, None);
        }
        if shot_is_current() {
            shooter.play_entity_sound(Sound::EntityBreezeShoot, SHOOT_VOLUME, 1.0);
        }
    });
}

struct ShootWhenStuck;

impl Behavior for ShootWhenStuck {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const CONDITIONS: [(MemoryModuleId, MemoryStatus); 5] = [
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (types::BREEZE_JUMP_INHALING.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_JUMP_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_SHOOT.id(), MemoryStatus::ValueAbsent),
        ];
        &CONDITIONS
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.mob.is_passenger()
            || tick.mob.get_entity().is_in_water()
            || tick
                .mob
                .get_mob_entity()
                .living_entity
                .has_effect(&StatusEffect::LEVITATION)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.set_with_expiry(types::BREEZE_SHOOT, (), 60);
    }

    fn debug_name(&self) -> &'static str {
        "ShootWhenStuck"
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use pumpkin_data::environment_attribute::Activity;

    use super::*;
    use crate::entity::{
        ai::{
            brain::{Brain, activity::ActivityData},
            test_support::world,
        },
        mob::breeze::BreezeEntity,
        passive::iron_golem::IronGolemEntity,
    };

    #[tokio::test]
    async fn a_stuck_breeze_fires_one_owned_projectile_unless_its_brain_is_rebuilt()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let breeze = BreezeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::BREEZE,
        ));
        let target = IronGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(8.5, 64.0, 0.5),
            &EntityType::IRON_GOLEM,
        ));
        breeze
            .get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);
        let start_attack = || {
            let mut brain = Brain::default();
            brain.add_activity(ActivityData::with_pairs(
                Activity::Fight,
                vec![(1, shoot()), (3, shoot_when_stuck())],
            ));
            brain.set_default_activity(Activity::Fight);
            brain.use_default_activity();
            brain.set(types::ATTACK_TARGET, target.clone());
            breeze.mob_entity.set_brain(brain);
        };
        let tick = |time, refresh| {
            let effects = breeze.mob_entity.with_brain(breeze.as_ref(), |brain| {
                brain.tick(&world, breeze.as_ref(), time);
                if refresh {
                    brain.rebuild(&world, breeze.as_ref(), time)
                } else {
                    brain.take_effects()
                }
            });
            for effect in effects {
                effect(breeze.as_ref());
            }
        };

        let count_charges = || {
            world
                .entities
                .load()
                .iter()
                .filter(|entity| entity.get_entity().entity_type == &EntityType::BREEZE_WIND_CHARGE)
                .count()
        };
        start_attack();
        for time in 0..=16 {
            tick(time, false);
        }
        let during_charge = (breeze.get_entity().pose.load(), count_charges());

        tick(17, false);
        let after_shot = count_charges();
        let owner = world
            .entities
            .load()
            .iter()
            .find_map(|entity| entity.get_projectile_owner().and_then(|owner| owner.uuid()));
        for time in 18..=22 {
            tick(time, false);
        }
        let after_recovery = (breeze.get_entity().pose.load(), count_charges());

        assert_eq!(during_charge, (EntityPose::Shooting, 0));
        assert_eq!(after_shot, 1);
        assert_eq!(owner, Some(breeze.get_entity().entity_uuid));
        assert_eq!(after_recovery, (EntityPose::Standing, 1));
        breeze.mob_entity.with_brain(breeze.as_ref(), |brain| {
            assert_eq!(
                brain.time_until_expiry(types::BREEZE_SHOOT_COOLDOWN.id()),
                Some(10)
            );
            assert!(!brain.has_memory_value(types::BREEZE_SHOOT.id()));
        });

        start_attack();
        for time in 0..=16 {
            tick(time, false);
        }
        assert_eq!(breeze.get_entity().pose.load(), EntityPose::Shooting);
        tick(17, true);
        assert_eq!(breeze.get_entity().pose.load(), EntityPose::Standing);
        assert_eq!(count_charges(), 1);
        world.level.shutdown().await;
        Ok(())
    }
}
