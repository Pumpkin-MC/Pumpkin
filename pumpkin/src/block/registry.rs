use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::item::Item;
use pumpkin_inventory::OpenContainer;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::registry::Block;
use std::collections::HashMap;
use std::sync::Arc;

use super::properties::Direction;

pub enum BlockActionResult {
    /// Allow other actions to be executed
    Continue,
    /// Block other actions
    Consume,
}

#[derive(Default)]
pub struct BlockRegistry {
    blocks: HashMap<String, Arc<dyn PumpkinBlock>>,
}

impl BlockRegistry {
    pub fn register<T: PumpkinBlock + BlockMetadata + 'static>(&mut self, block: T) {
        self.blocks.insert(block.name(), Arc::new(block));
    }

    pub async fn on_use(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .normal_use(block, player, location, server)
                .await;
        }
    }

    pub async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        item: &Item,
        server: &Server,
    ) -> BlockActionResult {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .use_with_item(block, player, location, item, server)
                .await;
        }
        BlockActionResult::Continue
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn on_place(
        &self,
        server: &Server,
        world: &World,
        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        player_direction: &Direction,
        other: bool,
    ) -> u16 {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .on_place(
                    server,
                    world,
                    block,
                    face,
                    block_pos,
                    use_item_on,
                    player_direction,
                    other,
                )
                .await;
        }
        server
            .block_properties_manager
            .on_place_state(
                world,
                block,
                face,
                block_pos,
                use_item_on,
                player_direction,
                other,
            )
            .await
    }

    pub async fn on_placed(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block.placed(block, player, location, server).await;
        }
    }

    pub async fn broken(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block.broken(block, player, location, server).await;
        }
    }

    pub async fn close(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
        container: &mut OpenContainer,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .close(block, player, location, server, container)
                .await;
        }
    }

    #[must_use]
    pub fn get_pumpkin_block(&self, block: &Block) -> Option<&Arc<dyn PumpkinBlock>> {
        self.blocks
            .get(format!("minecraft:{}", block.name).as_str())
    }
}
