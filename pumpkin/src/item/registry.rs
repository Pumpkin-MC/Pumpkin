use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;
use std::collections::HashMap;
use std::sync::Arc;

use super::pumpkin_item::{ItemMetadata, PumpkinItem};

#[derive(Default)]
pub struct ItemRegistry {
    items: HashMap<&'static Item, Arc<dyn PumpkinItem>>,
}

impl ItemRegistry {
    pub fn register<T: PumpkinItem + ItemMetadata + 'static>(&mut self, item: T) {
        let val = Arc::new(item);
        self.items.reserve(T::ids().len());
        for i in T::ids() {
            self.items.insert(Item::from_id(i).unwrap(), val.clone());
        }
    }

    pub async fn on_use(&self, item: &Item, player: &Player) {
        let pumpkin_item = self.get_pumpkin_item(item);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item.normal_use(item, player).await;
        }
    }

    pub async fn use_on_block(
        &self,
        item: &Item,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        block: &Block,
        server: &Server,
    ) {
        let pumpkin_item = self.get_pumpkin_item(item);
        if let Some(pumpkin_item) = pumpkin_item {
            return pumpkin_item
                .use_on_block(item, player, location, face, block, server)
                .await;
        }
    }

    pub fn can_mine(&self, item: &Item, player: &Player) -> bool {
        let pumpkin_block = self.get_pumpkin_item(item);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block.can_mine(player);
        }
        true
    }

    #[must_use]
    pub fn get_pumpkin_item(&self, item: &Item) -> Option<&Arc<dyn PumpkinItem>> {
        self.items.get(item)
    }
}
