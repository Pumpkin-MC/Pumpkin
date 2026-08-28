use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::Hand;
use std::sync::Arc;

use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::projectile::arrow::{ArrowEntity, ArrowPickup};
use crate::entity::{Entity, EntityBase};

/// Ranged bow attack used by skeletons and their variants.
/// Mirrors vanilla `RangedBowAttackGoal`: the mob keeps its distance, draws the bow
/// and releases an arrow once it has been drawn long enough.
pub struct BowAttackGoal {
    goal_control: Controls,
    speed: f64,
    attack_interval: i32,
    squared_range: f64,
    cooldown: i32,
    draw_ticks: i32,
    drawing: bool,
}

impl BowAttackGoal {
    /// Ticks the bow has to be drawn before the arrow is released.
    const DRAW_TIME: i32 = 20;
    /// Vanilla arrow speed for mob shots.
    const ARROW_SPEED: f64 = 1.6;

    #[must_use]
    pub fn new(speed: f64, attack_interval: i32, range: f32) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            speed,
            attack_interval,
            squared_range: f64::from(range * range),
            cooldown: -1,
            draw_ticks: 0,
            drawing: false,
        }
    }

    fn main_hand_item(mob: &dyn Mob) -> ItemStack {
        mob.get_mob_entity()
            .living_entity
            .entity_equipment
            .try_lock()
            .map_or_else(
                |_| ItemStack::EMPTY.clone(),
                |eq| eq.get(&EquipmentSlot::MAIN_HAND),
            )
    }

    fn is_holding_bow(mob: &dyn Mob) -> bool {
        Self::main_hand_item(mob).item.id == Item::BOW.id
    }

    fn stop_drawing(&mut self, mob: &dyn Mob) {
        if self.drawing {
            mob.get_mob_entity().living_entity.clear_active_hand();
            self.drawing = false;
            self.draw_ticks = 0;
        }
    }

    /// Spawns the arrow, matching vanilla `AbstractSkeleton::performRangedAttack`.
    fn shoot(mob: &dyn Mob, target: &Arc<dyn EntityBase>) {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let world_full = entity.world.load_full();

        let arrow_entity = Entity::new(world.clone(), entity.pos.load(), &EntityType::ARROW);
        let projectile = ItemStack::new(1, &Item::ARROW);
        let arrow = ArrowEntity::new_shot(arrow_entity, entity, &projectile, ArrowPickup::Allowed);

        let mob_pos = entity.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();

        let dx = target_pos.x - mob_pos.x;
        let dy = (target_pos.y + f64::from(target_entity.entity_dimension.load().height) / 3.0)
            - arrow.entity.pos.load().y;
        let dz = target_pos.z - mob_pos.z;
        let horizontal_distance = dx.hypot(dz);

        // Vanilla scales the spread with the world difficulty: 14 - difficulty * 4.
        let difficulty = world.level_info.load().difficulty as i32;
        let divergence = f64::from(14 - difficulty * 4);

        arrow.set_velocity(
            dx,
            horizontal_distance.mul_add(0.2, dy),
            dz,
            Self::ARROW_SPEED,
            divergence,
        );

        world.play_sound(Sound::EntityArrowShoot, SoundCategory::Hostile, &mob_pos);

        let arrow: Arc<dyn EntityBase> = Arc::new(arrow);
        let entity_id = entity.entity_id;
        if let Some(server) = world_full.server.upgrade() {
            let mut event =
                crate::plugin::api::events::entity::entity_shoot_bow::EntityShootBowEvent::new(
                    entity_id,
                    "minecraft:bow".to_string(),
                    1.0,
                );
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }
        world_full.spawn_entity(arrow);
    }
}

impl Goal for BowAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().get_target().clone();
        let Some(target) = target else {
            return false;
        };
        if !target.get_entity().is_alive() {
            return false;
        }
        Self::is_holding_bow(mob)
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().get_target().clone();
        let Some(target) = target else {
            return false;
        };
        target.get_entity().is_alive() && Self::is_holding_bow(mob)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.cooldown = -1;
        self.draw_ticks = 0;
        self.drawing = false;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.stop_drawing(mob);
        self.cooldown = -1;
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let target = mob.get_mob_entity().get_target().clone();
        let Some(target) = target else {
            return;
        };

        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let distance_sq = mob_pos.squared_distance_to_vec(&target_pos);

        mob.get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity_with_range(&target, 30.0, 30.0);

        // Close the gap while out of shooting range, otherwise hold position.
        {
            let mut navigator = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if distance_sq > self.squared_range {
                navigator.set_progress(NavigatorGoal {
                    current_progress: mob_pos,
                    destination: target_pos,
                    speed: self.speed,
                });
            } else {
                navigator.stop();
            }
        }

        if self.drawing {
            self.draw_ticks += 1;
            if self.draw_ticks >= Self::DRAW_TIME {
                self.stop_drawing(mob);
                Self::shoot(mob, &target);
                self.cooldown = self.attack_interval;
            }
            return;
        }

        self.cooldown -= 1;
        if self.cooldown <= 0 && distance_sq <= self.squared_range {
            let stack = Self::main_hand_item(mob);
            mob.get_mob_entity()
                .living_entity
                .set_active_hand(Hand::Right, stack, i32::MAX);
            self.drawing = true;
            self.draw_ticks = 0;
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
