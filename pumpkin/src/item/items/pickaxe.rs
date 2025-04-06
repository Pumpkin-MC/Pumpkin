use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
pub struct PickaxeItem;

impl ItemMetadata for PickaxeItem {
    fn ids() -> Box<[u16]> {
        Item::get_tag_values("#minecraft:pickaxes")
            .expect("This is a valid vanilla tag")
            .iter()
            .map(|key| {
                Item::from_registry_key(key)
                    .expect("We just got this key from the registry")
                    .id
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

#[async_trait]
impl PumpkinItem for PickaxeItem {
    async fn on_after_dig(
        &self,
        _item: &Item,
        player: &Player,
        _location: BlockPos,
        _block: &Block,
        _server: &Server,
    ) {
        if player.gamemode.load() != GameMode::Creative {
            self.increment_damage(player).await;
        }
    }
}
