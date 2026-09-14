use std::sync::Arc;

use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob, player::Player};
use pumpkin_data::item::Item;

const TEMPT_RANGE: f64 = 10.0;
const STOP_DISTANCE: f64 = 2.5;

pub struct TemptGoal {
    goal_control: Controls,
    speed: f64,
    tempt_items: &'static [&'static Item],
    target_player: Option<Arc<Player>>,
    cooldown: i32,
}

impl TemptGoal {
    #[must_use]
    pub fn new(speed: f64, tempt_items: &'static [&'static Item]) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            speed,
            tempt_items,
            target_player: None,
            cooldown: 0,
        }
    }

    fn is_tempt_item(&self, stack: &pumpkin_data::item_stack::ItemStack) -> bool {
        stack.item_count > 0 && self.tempt_items.iter().any(|i| i.id == stack.item.id)
    }

    fn is_holding_tempt_item(&self, player: &Player) -> bool {
        let main = player.inventory().held_item();
        if self.is_tempt_item(&main) {
            return true;
        }
        let off = player.inventory().off_hand_item();
        self.is_tempt_item(&off)
    }

    fn find_tempting_player(&self, mob: &dyn Mob) -> Option<Arc<Player>> {
        let mob_entity = mob.get_mob_entity();
        let pos = mob_entity.living_entity.entity.pos.load();
        let world = mob_entity.living_entity.entity.world.load();

        world
            .get_nearby_players(pos, TEMPT_RANGE)
            .into_iter()
            .find(|player| self.is_holding_tempt_item(player))
    }

    fn is_player_still_tempting(&self, player: &Player, mob: &dyn Mob) -> bool {
        let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let player_pos = player.get_entity().pos.load();
        if mob_pos.squared_distance_to_vec(&player_pos) > TEMPT_RANGE * TEMPT_RANGE {
            return false;
        }
        self.is_holding_tempt_item(player)
    }
}

impl Goal for TemptGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.cooldown > 0 {
            self.cooldown -= 1;
            return false;
        }
        self.target_player = self.find_tempting_player(mob);
        self.target_player.is_some()
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.target_player
            .as_ref()
            .is_some_and(|player| self.is_player_still_tempting(player, mob))
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if let Some(player) = &self.target_player {
            let mob_entity = mob.get_mob_entity();
            let player_pos = player.get_entity().pos.load();

            mob_entity
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at(
                    mob,
                    player_pos.x,
                    player.get_entity().get_eye_y(),
                    player_pos.z,
                );

            let mob_pos = mob_entity.living_entity.entity.pos.load();
            if mob_pos.squared_distance_to_vec(&player_pos) > STOP_DISTANCE * STOP_DISTANCE {
                let mut navigator = mob_entity
                    .navigator
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                navigator.set_progress(NavigatorGoal::new(mob_pos, player_pos, self.speed));
            }
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target_player = None;
        self.cooldown = 100;
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
