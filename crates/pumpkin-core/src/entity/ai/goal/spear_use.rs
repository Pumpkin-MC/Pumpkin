use pumpkin_data::data_component_impl::{EquipmentSlot, KineticWeaponImpl};
use pumpkin_util::Hand;
use pumpkin_util::math::vector3::Vector3;

use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::ai::util::land_random_pos;
use crate::entity::mob::Mob;

const MIN_REPOSITION_DISTANCE: f64 = 6.0;
const MAX_REPOSITION_DISTANCE: f64 = 7.0;
const MIN_COOLDOWN_DISTANCE: f64 = 9.0;
const MAX_COOLDOWN_DISTANCE: f64 = 11.0;
const MAX_FLEEING_TIME: i32 = to_goal_ticks(100);
const SPEAR_USE_DURATION: i32 = 72_000;

#[derive(Default)]
struct SpearUseState {
    engage_time: Option<i32>,
    fleeing_time: Option<i32>,
    away_pos: Option<Vector3<f64>>,
    done: bool,
}

impl SpearUseState {
    const fn tick_and_check_engagement(&mut self) -> bool {
        if let Some(engage_time) = self.engage_time.as_mut()
            && *engage_time > 0
        {
            *engage_time -= 1;
            return *engage_time == 0;
        }
        false
    }

    const fn tick_and_check_fleeing(&mut self) -> bool {
        if let Some(fleeing_time) = self.fleeing_time.as_mut()
            && *fleeing_time > 0
        {
            *fleeing_time += 1;
            if *fleeing_time > MAX_FLEEING_TIME {
                self.done = true;
                return true;
            }
        }
        false
    }
}

/// Charges the target with a held kinetic weapon, then backs off before the next charge.
pub struct SpearUseGoal {
    speed_modifier_when_charging: f64,
    speed_modifier_when_repositioning: f64,
    approach_distance_sq: f64,
    target_in_range_radius_sq: f64,
    state: Option<SpearUseState>,
}

impl SpearUseGoal {
    #[must_use]
    pub fn new(
        speed_modifier_when_charging: f64,
        speed_modifier_when_repositioning: f64,
        approach_distance: f32,
        target_in_range_radius: f32,
    ) -> Self {
        Self {
            speed_modifier_when_charging,
            speed_modifier_when_repositioning,
            approach_distance_sq: f64::from(approach_distance * approach_distance),
            target_in_range_radius_sq: f64::from(target_in_range_radius * target_in_range_radius),
            state: None,
        }
    }

    fn kinetic_weapon(mob: &dyn Mob) -> Option<KineticWeaponImpl> {
        mob.get_mob_entity()
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND)
            .get_data_component::<KineticWeaponImpl>()
            .cloned()
    }

    fn able_to_attack(mob: &dyn Mob) -> bool {
        mob.get_mob_entity().get_target().is_some() && Self::kinetic_weapon(mob).is_some()
    }

    fn is_using_item(mob: &dyn Mob) -> bool {
        mob.get_mob_entity()
            .living_entity
            .item_in_use
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    fn kinetic_weapon_use_duration(mob: &dyn Mob) -> i32 {
        to_goal_ticks(Self::kinetic_weapon(mob).map_or(0, |weapon| {
            weapon.delay_ticks
                + weapon
                    .damage_conditions
                    .as_ref()
                    .map_or(0, |condition| condition.max_duration_ticks)
        }))
    }

    fn move_to(mob: &dyn Mob, destination: Vector3<f64>, speed: f64) {
        let steered = mob.get_mob_entity().steered();
        steered.navigator().set_progress(NavigatorGoal {
            current_progress: mob.get_entity().pos.load(),
            destination,
            speed,
        });
    }
}

impl Goal for SpearUseGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        Self::able_to_attack(mob) && !Self::is_using_item(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.state.as_ref().is_some_and(|state| !state.done) && Self::able_to_attack(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity().set_attacking(true);
        self.state = Some(SpearUseState::default());
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        mob_entity.steered().navigator().stop();
        mob_entity.set_attacking(false);
        self.state = None;
        mob_entity.living_entity.clear_active_hand();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let mob_entity = mob.get_mob_entity();
        let Some(target) = mob_entity.get_target() else {
            return;
        };
        let entity = mob.get_entity();
        let target_pos = target.get_entity().pos.load();
        let target_dist_sq = entity.pos.load().squared_distance_to_vec(&target_pos);
        let speed_modifier = entity
            .get_root_vehicle()
            .as_deref()
            .and_then(|vehicle| vehicle.get_mob())
            .map_or(1.0, |vehicle| f64::from(vehicle.charge_speed_modifier()));
        let mount_distance = if entity.has_vehicle() { 2.0 } else { 0.0 };
        mob_entity.look_at(target.get_entity(), 30.0, 30.0);
        mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity_with_range(&target, 30.0, 30.0);

        if state.engage_time.is_none() {
            if target_dist_sq > self.approach_distance_sq {
                Self::move_to(
                    mob,
                    target_pos,
                    speed_modifier * self.speed_modifier_when_repositioning,
                );
                return;
            }
            state.engage_time = Some(Self::kinetic_weapon_use_duration(mob));
            let stack = mob_entity
                .living_entity
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&EquipmentSlot::MAIN_HAND);
            mob_entity
                .living_entity
                .set_active_hand(Hand::Right, stack, SPEAR_USE_DURATION);
        }

        if state.tick_and_check_engagement() {
            mob_entity.living_entity.clear_active_hand();
            let distance = target_dist_sq.sqrt();
            state.away_pos = land_random_pos::get_pos_away_between(
                mob,
                (MIN_COOLDOWN_DISTANCE + mount_distance - distance).max(0.0),
                (MAX_COOLDOWN_DISTANCE + mount_distance - distance).max(1.0),
                7,
                target_pos,
            );
            state.fleeing_time = Some(1);
        }

        if state.tick_and_check_fleeing() {
            return;
        }
        if let Some(away_pos) = state.away_pos {
            Self::move_to(
                mob,
                away_pos,
                speed_modifier * self.speed_modifier_when_repositioning,
            );
            if mob.is_navigator_idle() {
                if state
                    .fleeing_time
                    .is_some_and(|fleeing_time| fleeing_time > 0)
                {
                    state.done = true;
                    return;
                }
                state.away_pos = None;
            }
        } else {
            Self::move_to(
                mob,
                target_pos,
                speed_modifier * self.speed_modifier_when_charging,
            );
            if target_dist_sq < self.target_in_range_radius_sq || mob.is_navigator_idle() {
                let distance = target_dist_sq.sqrt();
                state.away_pos = land_random_pos::get_pos_away_between(
                    mob,
                    MIN_REPOSITION_DISTANCE + mount_distance - distance,
                    MAX_REPOSITION_DISTANCE + mount_distance - distance,
                    7,
                    target_pos,
                );
            }
        }
    }

    fn should_run_every_tick(&self) -> bool {
        false
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}
