use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_data::block::Block;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;
use std::collections::HashMap;
use std::sync::Arc;

use super::pumpkin_item::{ItemMetadata, PumpkinItem};

#[derive(Default)]
pub struct ItemRegistry {
    items: HashMap<&'static [u16], Arc<dyn PumpkinItem>>,
}

impl ItemRegistry {
    pub fn register<T: PumpkinItem + ItemMetadata + 'static>(&mut self, item: T) {
        self.items.insert(T::IDS, Arc::new(item));
    }

    pub async fn on_use(&self, item: &Item, player: &Player) {
        let pumpkin_block = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block.normal_use(item, player).await;
        }
    }

    pub async fn use_on_block(
        &self,
        item: &Item,
        player: &Player,
        location: BlockPos,
        block: &Block,
        server: &Server,
    ) {
        let pumpkin_item = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            return pumpkin_item
                .use_on_block(item, player, location, block, server)
                .await;
        }
    }

    pub fn can_mine(&self, item: &Item, player: &Player) -> bool {
        let pumpkin_block = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block.can_mine(player);
        }
        true
    }

    #[must_use]
    pub fn get_pumpkin_item(&self, item_id: u16) -> Option<&Arc<dyn PumpkinItem>> {
        self.items
            .iter()
            .find_map(|(ids, item)| ids.contains(&item_id).then_some(item))
    }
}
