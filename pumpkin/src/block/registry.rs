use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::block::{Block, BlockState, HorizontalFacing};
use pumpkin_data::item::Item;
use pumpkin_inventory::OpenContainer;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use std::collections::HashMap;
use std::sync::Arc;

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
        world: &World,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .normal_use(block, player, location, server, world)
                .await;
        }
    }

    pub async fn explode(&self, block: &Block, world: &Arc<World>, location: BlockPos) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block.explode(block, world, location).await;
        }
    }

    pub async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        item: &Item,
        server: &Server,
        world: &World,
    ) -> BlockActionResult {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .use_with_item(block, player, location, item, server, world)
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
        player_direction: &HorizontalFacing,
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
        block.default_state_id
    }

    pub async fn can_place(
        &self,
        server: &Server,
        world: &World,
        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        player_direction: &HorizontalFacing,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .can_place(server, world, block, face, block_pos, player_direction)
                .await;
        }
        true
    }

    pub async fn on_placed(
        &self,
        world: &World,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .placed(block, player, location, server, world)
                .await;
        }
        world.update_neighbors(server, &location, None).await;
    }

    pub async fn broken(
        &self,
        world: Arc<World>,
        block: &Block,
        player: &Player,
        location: BlockPos,
        server: &Server,
        state: BlockState,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .broken(block, player, location, server, world.clone(), state)
                .await;
        }
        world.update_neighbors(server, &location, None).await;
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
