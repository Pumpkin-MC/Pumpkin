use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_util::GameMode;

pub struct MaceItem;

impl ItemMetadata for MaceItem {
    fn ids() -> Box<[u16]> {
        [Item::MACE.id].into()
    }
}
#[async_trait]
impl PumpkinItem for MaceItem {
    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }
}
