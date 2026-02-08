use super::{Controls, Goal};
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::path::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::player::Player;
use pumpkin_data::item::Item;
use std::sync::Arc;

/// A goal that makes passive mobs follow players holding specific food items.
///
/// When a nearby player holds a tempt item (e.g. wheat for cows), the mob
/// walks toward them. Stops when the player switches items or goes out of range.
pub struct TemptGoal {
    goal_control: Controls,
    speed: f64,
    tempt_items: &'static [u16],
    target: Option<Arc<Player>>,
    range: f64,
    cooldown: i32,
}

impl TemptGoal {
    /// Create a new `TemptGoal`.
    ///
    /// `speed` — movement speed toward the player.
    /// `tempt_items` — static slice of item IDs that trigger temptation.
    /// `range` — maximum distance to detect a tempting player.
    #[must_use]
    pub fn new(speed: f64, tempt_items: &'static [u16], range: f64) -> Box<Self> {
        Box::new(Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            speed,
            tempt_items,
            target: None,
            range,
            cooldown: 0,
        })
    }

    fn is_tempted_by(&self, item_id: u16) -> bool {
        self.tempt_items.contains(&item_id)
    }
}

impl Goal for TemptGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            if self.cooldown > 0 {
                self.cooldown -= 1;
                return false;
            }

            let mob_entity = mob.get_mob_entity();
            let mob_pos = mob_entity.living_entity.entity.pos.load();
            let world = mob_entity.living_entity.entity.world.load();

            // Find nearby players sorted by distance
            let players = world.get_nearby_players(mob_pos, self.range);

            // Find the closest player holding a tempt item
            for player in &players {
                if !player.living_entity.entity.is_alive() {
                    continue;
                }
                let held = player.inventory.held_item();
                let stack = held.lock().await;
                if self.is_tempted_by(stack.item.id) {
                    self.target = Some(player.clone());
                    return true;
                }
            }

            false
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let Some(target) = &self.target else {
                return false;
            };

            if !target.living_entity.entity.is_alive() {
                return false;
            }

            let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
            let target_pos = target.living_entity.entity.pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

            if dist_sq > self.range * self.range {
                return false;
            }

            // Check if the player is still holding a tempt item
            let held = target.inventory.held_item();
            let stack = held.lock().await;
            self.tempt_items.contains(&stack.item.id)
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if let Some(target) = &self.target {
                let mob_entity = mob.get_mob_entity();
                let current_pos = mob_entity.living_entity.entity.pos.load();
                let target_pos = target.living_entity.entity.pos.load();
                let mut navigator = mob_entity.navigator.lock().await;
                navigator.set_progress(NavigatorGoal {
                    current_progress: current_pos,
                    destination: target_pos,
                    speed: self.speed,
                });
            }
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.target = None;
            self.cooldown = self.get_tick_count(100);
            let mut navigator = mob.get_mob_entity().navigator.lock().await;
            navigator.cancel();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if let Some(target) = &self.target {
                let mob_entity = mob.get_mob_entity();
                let target_pos = target.living_entity.entity.pos.load();

                // Look at the player
                mob_entity.living_entity.entity.look_at(target_pos);

                // Update navigation toward the player
                let current_pos = mob_entity.living_entity.entity.pos.load();
                let mut navigator = mob_entity.navigator.lock().await;
                navigator.set_progress(NavigatorGoal {
                    current_progress: current_pos,
                    destination: target_pos,
                    speed: self.speed,
                });
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

// Tempt item ID lists for common mobs (vanilla-accurate).
// Use `Item::FOO.id` from pumpkin_data::item::Item.

/// Wheat — tempts cows, sheep, goats, mooshrooms.
pub static TEMPT_WHEAT: &[u16] = &[Item::WHEAT.id];

/// Seeds — tempts chickens, parrots.
pub static TEMPT_SEEDS: &[u16] = &[
    Item::WHEAT_SEEDS.id,
    Item::MELON_SEEDS.id,
    Item::PUMPKIN_SEEDS.id,
    Item::BEETROOT_SEEDS.id,
    Item::TORCHFLOWER_SEEDS.id,
];

/// Carrots/potatoes/beetroots — tempts pigs.
pub static TEMPT_PIG: &[u16] = &[Item::CARROT.id, Item::POTATO.id, Item::BEETROOT.id];

/// Carrots/golden carrots/dandelions — tempts rabbits.
pub static TEMPT_RABBIT: &[u16] = &[Item::CARROT.id, Item::GOLDEN_CARROT.id, Item::DANDELION.id];

/// Cod/salmon — tempts cats, ocelots.
pub static TEMPT_CAT: &[u16] = &[Item::COD.id, Item::SALMON.id];

/// Sweet berries/glow berries — tempts foxes.
pub static TEMPT_FOX: &[u16] = &[Item::SWEET_BERRIES.id, Item::GLOW_BERRIES.id];

/// Raw/cooked meat — tempts wolves.
pub static TEMPT_WOLF: &[u16] = &[
    Item::BEEF.id,
    Item::COOKED_BEEF.id,
    Item::PORKCHOP.id,
    Item::COOKED_PORKCHOP.id,
    Item::MUTTON.id,
    Item::COOKED_MUTTON.id,
    Item::CHICKEN.id,
    Item::COOKED_CHICKEN.id,
    Item::RABBIT.id,
    Item::COOKED_RABBIT.id,
];

/// Bamboo — tempts pandas.
pub static TEMPT_PANDA: &[u16] = &[Item::BAMBOO.id];

/// Seagrass — tempts turtles.
pub static TEMPT_TURTLE: &[u16] = &[Item::SEAGRASS.id];

/// Tropical fish bucket — tempts axolotls.
pub static TEMPT_AXOLOTL: &[u16] = &[Item::TROPICAL_FISH_BUCKET.id];

/// Golden apples/golden carrots — tempts horses.
pub static TEMPT_HORSE: &[u16] = &[Item::GOLDEN_APPLE.id, Item::GOLDEN_CARROT.id];

/// Warped fungus — tempts striders.
pub static TEMPT_STRIDER: &[u16] = &[Item::WARPED_FUNGUS.id];

/// Slime ball — tempts frogs.
pub static TEMPT_FROG: &[u16] = &[Item::SLIME_BALL.id];

/// Cactus — tempts camels.
pub static TEMPT_CAMEL: &[u16] = &[Item::CACTUS.id];

/// Spider eye — tempts armadillos.
pub static TEMPT_ARMADILLO: &[u16] = &[Item::SPIDER_EYE.id];

/// Torchflower seeds — tempts sniffers.
pub static TEMPT_SNIFFER: &[u16] = &[Item::TORCHFLOWER_SEEDS.id];

/// Hay bale — tempts llamas.
pub static TEMPT_LLAMA: &[u16] = &[Item::HAY_BLOCK.id];

/// Flowers — tempts bees.
pub static TEMPT_BEE: &[u16] = &[
    Item::DANDELION.id,
    Item::POPPY.id,
    Item::BLUE_ORCHID.id,
    Item::ALLIUM.id,
    Item::AZURE_BLUET.id,
    Item::RED_TULIP.id,
    Item::ORANGE_TULIP.id,
    Item::WHITE_TULIP.id,
    Item::PINK_TULIP.id,
    Item::OXEYE_DAISY.id,
    Item::CORNFLOWER.id,
    Item::LILY_OF_THE_VALLEY.id,
    Item::SUNFLOWER.id,
    Item::LILAC.id,
    Item::ROSE_BUSH.id,
    Item::PEONY.id,
    Item::TORCHFLOWER.id,
    Item::WITHER_ROSE.id,
];
