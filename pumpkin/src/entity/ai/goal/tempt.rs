use std::sync::Arc;

use super::{Controls, Goal, GoalFuture};
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

    fn is_tempt_item(&self, stack: &pumpkin_world::item::ItemStack) -> bool {
        stack.item_count > 0 && self.tempt_items.iter().any(|i| i.id == stack.item.id)
    }

    async fn is_holding_tempt_item(&self, player: &Player) -> bool {
        let main = player.inventory.held_item();
        if self.is_tempt_item(&*main.lock().await) {
            return true;
        }
        let off = player.inventory.off_hand_item().await;
        self.is_tempt_item(&*off.lock().await)
    }

    async fn find_tempting_player(&self, mob: &dyn Mob) -> Option<Arc<Player>> {
        let mob_entity = mob.get_mob_entity();
        let pos = mob_entity.living_entity.entity.pos.load();
        let world = mob_entity.living_entity.entity.world.load();

        for player in world.get_nearby_players(pos, TEMPT_RANGE) {
            if self.is_holding_tempt_item(&player).await {
                return Some(player);
            }
        }
        None
    }

    async fn is_player_still_tempting(&self, player: &Player, mob: &dyn Mob) -> bool {
        let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let player_pos = player.living_entity.entity.pos.load();
        if mob_pos.squared_distance_to_vec(&player_pos) > TEMPT_RANGE * TEMPT_RANGE {
            return false;
        }
        self.is_holding_tempt_item(player).await
    }
}

impl Goal for TemptGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.cooldown > 0 {
                self.cooldown -= 1;
                return false;
            }
            self.target_player = self.find_tempting_player(mob).await;
            self.target_player.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if let Some(player) = &self.target_player {
                self.is_player_still_tempting(player, mob).await
            } else {
                false
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(player) = &self.target_player {
                let mob_entity = mob.get_mob_entity();
                let player_pos = player.living_entity.entity.pos.load();

                mob_entity.look_control.lock().await.look_at(
                    mob,
                    player_pos.x,
                    player.living_entity.entity.get_eye_y(),
                    player_pos.z,
                );

                let mob_pos = mob_entity.living_entity.entity.pos.load();
                if mob_pos.squared_distance_to_vec(&player_pos) > STOP_DISTANCE * STOP_DISTANCE {
                    let mut navigator = mob_entity.navigator.lock().await;
                    navigator.set_progress(NavigatorGoal::new(mob_pos, player_pos, self.speed));
                }
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target_player = None;
            self.cooldown = 100;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
