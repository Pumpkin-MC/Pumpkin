use std::pin::Pin;

use pumpkin_data::item::Item;
use pumpkin_util::GameMode;

use crate::{
    entity::player::Player,
    item::{ItemBehaviour, ItemMetadata},
};

pub struct TridentItem;

impl ItemMetadata for TridentItem {
    fn ids() -> Box<[u16]> {
        [Item::TRIDENT.id].into()
    }
}

impl ItemBehaviour for TridentItem {
    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }

    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // TODO: Implement trident throwing (spawn entity, apply durability, handle Riptide).
            let _ = player;
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
