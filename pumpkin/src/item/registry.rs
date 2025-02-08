use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;
use std::collections::HashMap;
use std::sync::Arc;

use super::pumpkin_item::{ItemMetadata, PumpkinItem};

#[derive(Default)]
pub struct ItemRegistry {
    items: HashMap<u16, Arc<dyn PumpkinItem>>,
}

impl ItemRegistry {
    pub fn register<T: PumpkinItem + ItemMetadata + 'static>(&mut self, item: T) {
        self.items.insert(T::ID, Arc::new(item));
    }

    pub async fn on_use(&self, item: &Item, player: &Player, server: &Server) {
        let pumpkin_block = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block.normal_use(item, player, server).await;
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

    #[must_use]
    pub fn get_pumpkin_item(&self, item_id: u16) -> Option<&Arc<dyn PumpkinItem>> {
        self.items.get(&item_id)
    }
}
