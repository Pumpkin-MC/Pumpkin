use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;
use pumpkin_world::item::registry::Item;
use std::collections::HashMap;
use std::sync::Arc;

use super::pumpkin_item::{ItemMetadata, PumpkinItem};

#[derive(Default)]
pub struct ItemRegistry {
    blocks: HashMap<String, Arc<dyn PumpkinItem>>,
}

impl ItemRegistry {
    pub fn register<T: PumpkinItem + ItemMetadata + 'static>(&mut self, block: T) {
        self.blocks.insert(block.name(), Arc::new(block));
    }

    pub async fn on_use(&self, item_name: &str, item: &Item, player: &Player, server: &Server) {
        let pumpkin_block = self.get_pumpkin_item(item_name);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block.normal_use(item, player, server).await;
        }
    }

    pub async fn use_on_block(
        &self,
        item_name: &str,
        item: &Item,
        player: &Player,
        location: BlockPos,
        block: &Block,
        server: &Server,
    ) {
        let pumpkin_block = self.get_pumpkin_item(item_name);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .use_on_block(item, player, location, block, server)
                .await;
        }
    }

    #[must_use]
    pub fn get_pumpkin_item(&self, item_name: &str) -> Option<&Arc<dyn PumpkinItem>> {
        self.blocks.get(format!("minecraft:{item_name}").as_str())
    }
}
