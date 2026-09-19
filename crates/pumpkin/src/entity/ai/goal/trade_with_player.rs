use std::sync::atomic::Ordering;

use super::{Controls, Goal};
use crate::entity::mob::Mob;

const MAX_TRADE_DISTANCE_SQ: f64 = 16.0;

/// While a player is trading the merchant stands still; looking at them is a separate goal.
#[derive(Default)]
pub struct TradeWithPlayerGoal;

impl TradeWithPlayerGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Goal for TradeWithPlayerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = &mob.get_mob_entity().living_entity.entity;
        if !entity.is_alive()
            || entity.touching_water.load(Ordering::Relaxed)
            || !entity.on_ground.load(Ordering::Relaxed)
            || entity.velocity_dirty.load(Ordering::Relaxed)
        {
            return false;
        }

        let Some(player) = mob.get_trading_player() else {
            return false;
        };
        entity
            .pos
            .load()
            .squared_distance_to_vec(&player.living_entity.entity.pos.load())
            <= MAX_TRADE_DISTANCE_SQ
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn stop(&mut self, mob: &dyn Mob) {
        // The merchant screen's validity check then closes the trade on its own.
        mob.clear_trading_player();
    }

    fn controls(&self) -> Controls {
        Controls::JUMP | Controls::MOVE
    }
}
