use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::Hand;

use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::{Entity, EntityBase};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrossbowState {
    Uncharged,
    Charging,
    Charged,
    ReadyToAttack,
}

/// Ranged crossbow attack used by Pillagers, Piglins, and other mobs.
/// Mirrors vanilla `RangedCrossbowAttackGoal`.
pub struct RangedCrossbowAttackGoal {
    goal_control: Controls,
    speed: f64,
    squared_range: f64,
    state: CrossbowState,
    see_time: i32,
    attack_delay: i32,
    update_path_delay: i32,
    charge_ticks: i32,
}

impl RangedCrossbowAttackGoal {
    /// Vanilla crossbow charge duration (ticks).
    const CHARGE_DURATION: i32 = 25;
    /// Vanilla arrow speed for crossbow shots.
    const ARROW_SPEED: f64 = 1.6;

    #[must_use]
    pub fn new(speed: f64, range: f32) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            speed,
            squared_range: f64::from(range * range),
            state: CrossbowState::Uncharged,
            see_time: 0,
            attack_delay: 0,
            update_path_delay: 0,
            charge_ticks: 0,
        }
    }

    fn is_holding_crossbow(mob: &dyn Mob) -> bool {
        let equipment_guard = mob
            .get_mob_entity()
            .living_entity
            .entity_equipment
            .try_lock();
        equipment_guard.is_ok_and(|equipment| {
            equipment.get(&EquipmentSlot::MAIN_HAND).item.id == Item::CROSSBOW.id
                || equipment.get(&EquipmentSlot::OFF_HAND).item.id == Item::CROSSBOW.id
        })
    }

    fn shoot(mob: &dyn Mob, target: &Arc<dyn EntityBase>) {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let world_full = entity.world.load_full();

        let mob_pos = entity.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();

        let arrow_entity = Entity::new(world.clone(), mob_pos, &EntityType::ARROW);
        let projectile = ItemStack::new(1, &Item::ARROW);
        let arrow = ArrowEntity::new_shot(arrow_entity, entity, &projectile, ArrowPickup::Allowed);

        let dx = target_pos.x - mob_pos.x;
        let dy = (target_pos.y + f64::from(target_entity.entity_dimension.load().height) / 3.0)
            - arrow.entity.pos.load().y;
        let dz = target_pos.z - mob_pos.z;
        let horizontal_distance = dx.hypot(dz);

        let difficulty = world.level_info.load().difficulty as i32;
        let divergence = f64::from(14 - difficulty * 4);

        arrow.set_velocity(
            dx,
            horizontal_distance.mul_add(0.2, dy),
            dz,
            Self::ARROW_SPEED,
            divergence,
        );

        world.play_sound(Sound::ItemCrossbowShoot, SoundCategory::Hostile, &mob_pos);

        let arrow: Arc<dyn EntityBase> = Arc::new(arrow);
        let entity_id = entity.entity_id;
        if let Some(server) = world_full.server.upgrade() {
            let mut event =
                crate::plugin::api::events::entity::entity_shoot_bow::EntityShootBowEvent::new(
                    entity_id,
                    "minecraft:crossbow".to_string(),
                    1.0,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }
        world_full.spawn_entity(arrow);

        if let Some(crossbow_mob) = mob.as_crossbow_attack_mob() {
            crossbow_mob.on_crossbow_attack_performed();
        }
    }
}

impl Goal for RangedCrossbowAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().get_target().clone();
        let Some(target) = target else {
            return false;
        };
        if !target.get_entity().is_alive() {
            return false;
        }
        Self::is_holding_crossbow(mob)
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().get_target().clone();
        let Some(target) = target else {
            return false;
        };
        target.get_entity().is_alive() && Self::is_holding_crossbow(mob)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.state = CrossbowState::Uncharged;
        self.see_time = 0;
        self.attack_delay = 0;
        self.update_path_delay = 0;
        self.charge_ticks = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(crossbow_mob) = mob.as_crossbow_attack_mob() {
            crossbow_mob.set_charging_crossbow(false);
        }
        mob.get_mob_entity().living_entity.clear_active_hand();
        self.state = CrossbowState::Uncharged;
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    #[expect(clippy::too_many_lines)]
    fn tick(&mut self, mob: &dyn Mob) {
        let target = mob.get_mob_entity().get_target().clone();
        let Some(target) = target else {
            return;
        };

        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let distance_sq = mob_pos.squared_distance_to_vec(&target_pos);

        let has_line_of_sight = true; // In future: raycast check
        if has_line_of_sight {
            self.see_time += 1;
        } else {
            self.see_time = 0;
        }

        let needs_to_move =
            (distance_sq > self.squared_range || self.see_time < 5) && self.attack_delay == 0;

        if needs_to_move {
            self.update_path_delay -= 1;
            if self.update_path_delay <= 0 {
                let move_speed = if self.state == CrossbowState::Uncharged {
                    self.speed
                } else {
                    self.speed * 0.5
                };
                mob.get_mob_entity()
                    .navigator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .set_progress(NavigatorGoal {
                        current_progress: mob_pos,
                        destination: target_pos,
                        speed: move_speed,
                    });
                self.update_path_delay = 20 + rand::random_range(0..20);
            }
        } else {
            self.update_path_delay = 0;
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .stop();
        }

        mob.get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity_with_range(&target, 30.0, 30.0);

        match self.state {
            CrossbowState::Uncharged => {
                if !needs_to_move {
                    let stack = mob
                        .get_mob_entity()
                        .living_entity
                        .entity_equipment
                        .try_lock()
                        .map_or_else(
                            |_| ItemStack::EMPTY.clone(),
                            |eq| eq.get(&EquipmentSlot::MAIN_HAND),
                        );
                    mob.get_mob_entity().living_entity.set_active_hand(
                        Hand::Right,
                        stack,
                        i32::MAX,
                    );
                    self.state = CrossbowState::Charging;
                    self.charge_ticks = 0;
                    if let Some(crossbow_mob) = mob.as_crossbow_attack_mob() {
                        crossbow_mob.set_charging_crossbow(true);
                    }
                    mob.get_entity().world.load().play_sound(
                        Sound::ItemCrossbowLoadingStart,
                        SoundCategory::Hostile,
                        &mob_pos,
                    );
                }
            }
            CrossbowState::Charging => {
                self.charge_ticks += 1;
                if self.charge_ticks == 10 {
                    mob.get_entity().world.load().play_sound(
                        Sound::ItemCrossbowLoadingMiddle,
                        SoundCategory::Hostile,
                        &mob_pos,
                    );
                }
                if self.charge_ticks >= Self::CHARGE_DURATION {
                    mob.get_mob_entity().living_entity.clear_active_hand();
                    self.state = CrossbowState::Charged;
                    self.attack_delay = 20 + rand::random_range(0..20);
                    if let Some(crossbow_mob) = mob.as_crossbow_attack_mob() {
                        crossbow_mob.set_charging_crossbow(false);
                    }
                    mob.get_entity().world.load().play_sound(
                        Sound::ItemCrossbowLoadingEnd,
                        SoundCategory::Hostile,
                        &mob_pos,
                    );
                }
            }
            CrossbowState::Charged => {
                self.attack_delay -= 1;
                if self.attack_delay <= 0 {
                    self.state = CrossbowState::ReadyToAttack;
                }
            }
            CrossbowState::ReadyToAttack => {
                if has_line_of_sight {
                    Self::shoot(mob, &target);
                    self.state = CrossbowState::Uncharged;
                }
            }
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
