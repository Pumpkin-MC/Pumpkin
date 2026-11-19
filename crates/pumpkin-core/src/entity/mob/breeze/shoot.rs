use std::sync::Arc;

use pumpkin_data::{
    effect::StatusEffect,
    entity::{EntityPose, EntityType},
    item_stack::ItemStack,
    sound::Sound,
};
use pumpkin_util::math::vector3::Vector3;

use super::{breeze_from_tick, look_from_eyes};
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
        if breeze.pose.load() != EntityPose::Standing {
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
        breeze_from_tick(tick).is_some_and(|breeze| breeze.planned_pose() == EntityPose::Shooting)
            && tick.state.has_memory_value(types::ATTACK_TARGET.id())
            && tick.state.has_memory_value(types::BREEZE_SHOOT.id())
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let Some(breeze) = breeze_from_tick(tick) else {
            return;
        };
        tick.state.set_with_expiry(
            types::BREEZE_SHOOT_CHARGING,
            (),
            i64::from(SHOOT_INITIAL_DELAY_TICKS),
        );
        breeze.change_pose(EntityPose::Shooting, |breeze| {
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
        tick.state.set_with_expiry(
            types::BREEZE_SHOOT_RECOVERING,
            (),
            i64::from(SHOOT_RECOVER_DELAY_TICKS),
        );
        fire_wind_charge(tick, aim_at);
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(breeze) = breeze_from_tick(tick)
            && breeze.planned_pose() == EntityPose::Shooting
        {
            breeze.change_pose(EntityPose::Standing, |_| {});
        }
        tick.state
            .set_with_expiry(types::BREEZE_SHOOT_COOLDOWN, (), SHOOT_COOLDOWN_TICKS);
        tick.state.erase(types::BREEZE_SHOOT.id());
    }

    fn debug_name(&self) -> &'static str {
        "Shoot"
    }
}

fn fire_wind_charge(tick: &BrainTick<'_>, aim_at: Vector3<f64>) {
    let Some(shooter) = breeze_from_tick(tick) else {
        return;
    };
    let breeze = shooter.get_entity();
    let mut firing_position = breeze.pos.load();
    firing_position.y = firing_position.y + f64::from(breeze.height() / 2.0) + f64::from(0.3f32);

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

    shooter.after_brain_tick(move |shooter| {
        let charge = Arc::new(WindChargeEntity::new_breeze(thrown));
        let world = charge.get_entity().world.load_full();
        if world.spawn_entity(charge.clone()) {
            apply_on_projectile_spawned(charge.get_entity(), ItemStack::EMPTY, None, None);
        }
        shooter.play_entity_sound(Sound::EntityBreezeShoot, 1.5, 1.0);
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
    async fn a_stuck_breeze_charges_then_fires_one_owned_projectile()
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
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_pairs(
            Activity::Fight,
            vec![(1, shoot()), (3, shoot_when_stuck())],
        ));
        brain.set_default_activity(Activity::Fight);
        brain.use_default_activity();
        brain.set(types::ATTACK_TARGET, target);

        let count_charges = || {
            world
                .entities
                .load()
                .iter()
                .filter(|entity| entity.get_entity().entity_type == &EntityType::BREEZE_WIND_CHARGE)
                .count()
        };
        for time in 0..=16 {
            brain.tick(&world, &*breeze, time);
            breeze.flush_brain_effects();
        }
        let during_charge = (breeze.get_entity().pose.load(), count_charges());

        brain.tick(&world, &*breeze, 17);
        breeze.flush_brain_effects();
        let after_shot = count_charges();
        let owner = world
            .entities
            .load()
            .iter()
            .find_map(|entity| entity.get_projectile_owner().and_then(|owner| owner.uuid()));
        for time in 18..=22 {
            brain.tick(&world, &*breeze, time);
            breeze.flush_brain_effects();
        }
        let after_recovery = (breeze.get_entity().pose.load(), count_charges());
        world.level.shutdown().await;

        assert_eq!(during_charge, (EntityPose::Shooting, 0));
        assert_eq!(after_shot, 1);
        assert_eq!(owner, Some(breeze.get_entity().entity_uuid));
        assert_eq!(after_recovery, (EntityPose::Standing, 1));
        assert_eq!(
            brain.time_until_expiry(types::BREEZE_SHOOT_COOLDOWN.id()),
            Some(10)
        );
        assert!(!brain.has_memory_value(types::BREEZE_SHOOT.id()));
        Ok(())
    }
}
