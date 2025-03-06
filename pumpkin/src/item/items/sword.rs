use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_util::GameMode;

pub struct SwordItem;

impl ItemMetadata for SwordItem {
    const IDS: &'static [u16] = &[
        Item::WOODEN_SWORD.id,
        Item::STONE_SWORD.id,
        Item::GOLDEN_SWORD.id,
        Item::DIAMOND_SWORD.id,
        Item::NETHERITE_SWORD.id,
    ];
}

#[async_trait]
impl PumpkinItem for SwordItem {
    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }
}
