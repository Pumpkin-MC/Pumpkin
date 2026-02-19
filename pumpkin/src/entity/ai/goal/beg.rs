use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::entity::player::Player;
use pumpkin_data::item::Item;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::java::client::play::Metadata;
use rand::RngExt;
use std::sync::Arc;

pub struct BegGoal {
    beg_distance_sq: f64,
    attractive_items: &'static [&'static Item],
    timer: i32,
    target: Option<Arc<Player>>,
}

impl BegGoal {
    #[must_use]
    pub fn new(beg_distance: f32, attractive_items: &'static [&'static Item]) -> Box<Self> {
        Box::new(Self {
            beg_distance_sq: (beg_distance * beg_distance) as f64,
            attractive_items,
            timer: 0,
            target: None,
        })
    }

    fn is_attractive(&self, item: &Item) -> bool {
        self.attractive_items.iter().any(|i| i.id == item.id)
    }

    async fn is_player_holding_attractive(&self, player: &Player) -> bool {
        let main_hand = player.inventory.held_item();
        let main_stack = main_hand.lock().await;
        if main_stack.item_count > 0 && self.is_attractive(main_stack.item) {
            return true;
        }
        drop(main_stack);

        let off_hand = player.inventory.off_hand_item().await;
        let off_stack = off_hand.lock().await;
        off_stack.item_count > 0 && self.is_attractive(off_stack.item)
    }

    fn distance_sq(mob: &dyn Mob, player: &Player) -> f64 {
        let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let player_pos = player.living_entity.entity.pos.load();
        mob_pos.squared_distance_to_vec(&player_pos)
    }

    async fn set_begging(mob: &dyn Mob, begging: bool) {
        mob.get_mob_entity()
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_BEGGING,
                MetaDataType::Boolean,
                begging,
            )])
            .await;
    }
}

impl Goal for BegGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load_full();
            let pos = entity.pos.load();
            let radius = self.beg_distance_sq.sqrt();

            let Some(player) = world.get_closest_player(pos, radius) else {
                return false;
            };

            if !self.is_player_holding_attractive(&player).await {
                return false;
            }

            self.target = Some(player);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let Some(player) = &self.target else {
                return false;
            };

            if !player.living_entity.entity.is_alive() {
                return false;
            }

            if Self::distance_sq(mob, player) > self.beg_distance_sq {
                return false;
            }

            self.timer > 0 && self.is_player_holding_attractive(player).await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            Self::set_begging(mob, true).await;
            let ticks = 40 + mob.get_random().random_range(0..40);
            self.timer = self.get_tick_count(ticks);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            Self::set_begging(mob, false).await;
            self.target = None;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if let Some(player) = &self.target {
                let player_pos = player.living_entity.entity.get_eye_pos();
                let mut look_control = mob.get_mob_entity().look_control.lock().await;
                look_control.look_at_with_range(
                    player_pos.x,
                    player_pos.y,
                    player_pos.z,
                    10.0,
                    mob.get_max_look_pitch_change(),
                );
            }
            self.timer -= 1;
        })
    }

    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}
