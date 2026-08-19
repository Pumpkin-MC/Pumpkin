use super::{Controls, Goal, GoalFuture};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::player::Player;
use pumpkin_data::item::Item;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_protocol::java::client::play::Metadata;
use rand::RngExt;
use std::sync::Arc;

pub struct BegGoal {
    look_distance_sq: f64,
    look_time: i32,
    player: Option<Arc<Player>>,
}

impl BegGoal {
    #[must_use]
    pub fn new(look_distance: f32) -> Box<Self> {
        Box::new(Self {
            look_distance_sq: f64::from(look_distance) * f64::from(look_distance),
            look_time: 0,
            player: None,
        })
    }

    fn is_interesting_item(item: &Item) -> bool {
        item.id == Item::BONE.id || item.has_tag(&tag::Item::MINECRAFT_WOLF_FOOD)
    }

    async fn player_holding_interesting(&self, player: &Player) -> bool {
        let main_stack = player.inventory().held_item().await;
        if main_stack.item_count > 0 && Self::is_interesting_item(main_stack.item) {
            return true;
        }

        let off_stack = player.inventory().off_hand_item().await;
        off_stack.item_count > 0 && Self::is_interesting_item(off_stack.item)
    }

    fn distance_sq(mob: &dyn Mob, player: &Player) -> f64 {
        let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let player_pos = player.get_entity().pos.load();
        mob_pos.squared_distance_to_vec(&player_pos)
    }

    fn set_is_interested(mob: &dyn Mob, value: bool) {
        mob.get_mob_entity().living_entity.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::wolf::INTERESTED_ID,
                value,
            )],
            None,
        );
    }
}

impl Goal for BegGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load_full();
            let pos = entity.pos.load();
            let radius = self.look_distance_sq.sqrt();

            let Some(player) = world.get_closest_player(pos, radius) else {
                return false;
            };

            if !self.player_holding_interesting(&player).await {
                return false;
            }

            self.player = Some(player);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(player) = &self.player else {
                return false;
            };

            if !player.get_entity().is_alive() {
                return false;
            }

            if Self::distance_sq(mob, player) > self.look_distance_sq {
                return false;
            }

            self.look_time > 0 && self.player_holding_interesting(player).await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            Self::set_is_interested(mob, true);
            let ticks = 40 + mob.get_random().random_range(0..40);
            self.look_time = self.get_tick_count(ticks);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            Self::set_is_interested(mob, false);
            self.player = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(player) = &self.player {
                let player_pos = player.get_entity().get_eye_pos();
                let mut look_control = mob
                    .get_mob_entity()
                    .look_control
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                look_control.look_at_with_range(
                    player_pos.x,
                    player_pos.y,
                    player_pos.z,
                    10.0,
                    mob.get_max_look_pitch_change(),
                );
            }
            self.look_time -= 1;
        })
    }

    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}
