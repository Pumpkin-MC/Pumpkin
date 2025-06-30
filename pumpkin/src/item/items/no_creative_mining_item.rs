use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_util::GameMode;

pub struct NoCreativeMiningItem;

impl ItemMetadata for NoCreativeMiningItem {
    fn ids() -> Box<[u16]> {
        Item::get_tag_values("#minecraft:swords")
            .expect("This is a valid vanilla tag")
            .iter()
            .map(|key| {
                Item::from_registry_key(key)
                    .expect("We just got this key from the registry")
                    .id
            })
            .chain([Item::MACE.id, Item::TRIDENT.id])
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

#[async_trait]
impl PumpkinItem for NoCreativeMiningItem {
    fn can_mine(&self, player: &Player) -> bool {
        player.gamemode.load() != GameMode::Creative
    }
}
