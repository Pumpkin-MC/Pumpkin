use std::sync::Arc;

use super::super::memory::types;
use crate::entity::EntityBase;

pub mod give_gift_to_hero;
pub mod interact_with;
pub mod look_and_follow_trading_player_sink;
pub mod play_tag_with_other_kids;
pub mod set_entity_look_target;
pub mod set_look_and_interact;
pub mod show_trades_to_player;
pub mod trade_with_villager;
pub mod villager_make_love;

pub use give_gift_to_hero::GiveGiftToHero;
pub use look_and_follow_trading_player_sink::LookAndFollowTradingPlayerSink;
pub use show_trades_to_player::ShowTradesToPlayer;
pub use trade_with_villager::TradeWithVillager;
pub use villager_make_love::VillagerMakeLove;

fn interaction_target_of(entity: &Arc<dyn EntityBase>) -> Option<Arc<dyn EntityBase>> {
    let mob = entity.get_mob()?;
    mob.get_mob_entity()
        .with_brain(mob, |brain| brain.get(types::INTERACTION_TARGET).cloned())
}
