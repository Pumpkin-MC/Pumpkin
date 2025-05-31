use crate::block::BlockIsReplacing;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::{Block, FacingExt, HorizontalFacingExt};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockAccessor;

type WallTorchProps = pumpkin_data::block_properties::WallTorchLikeProperties;
// Normal tourches don't have properties

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::server::Server;
use crate::world::World;

pub struct TorchBlock;

impl BlockMetadata for TorchBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[
            Block::TORCH.name,
            Block::SOUL_TORCH.name,
            Block::WALL_TORCH.name,
            Block::SOUL_WALL_TORCH.name,
        ]
    }
}

#[async_trait]
impl PumpkinBlock for TorchBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        player: &Player,
        block: &Block,
        block_pos: &BlockPos,
        face: BlockDirection,
        replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        if face == BlockDirection::Down {
            let support_block = world.get_block_state(&block_pos.down()).await;
            if support_block.is_center_solid(BlockDirection::Up) {
                return block.default_state_id;
            }
        }
        let mut directions = player.get_entity().get_entity_facing_order();

        if replacing == BlockIsReplacing::None {
            let face = face.to_facing();
            let mut i = 0;
            while i < directions.len() && directions[i] != face {
                i += 1;
            }

            if i > 0 {
                directions.copy_within(0..i, 1);
                directions[0] = face;
            }
        } else if directions[0] == Facing::Down {
            let support_block = world.get_block_state(&block_pos.down()).await;
            if support_block.is_center_solid(BlockDirection::Up) {
                return block.default_state_id;
            }
        }

        for dir in directions {
            if dir != Facing::Up
                && dir != Facing::Down
                && can_place_at(world, block_pos, dir.to_block_direction()).await
            {
                let wall_block = if *block == Block::TORCH {
                    Block::WALL_TORCH
                } else {
                    Block::SOUL_WALL_TORCH
                };
                let mut torch_props = WallTorchProps::default(&wall_block);
                torch_props.facing = dir
                    .opposite()
                    .to_block_direction()
                    .to_horizontal_facing()
                    .unwrap();
                return torch_props.to_state_id(&wall_block);
            }
        }

        let support_block = world.get_block_state(&block_pos.down()).await;
        if support_block.is_center_solid(BlockDirection::Up) {
            block.default_state_id
        } else {
            0
        }
    }

    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        let support_block = block_accessor.get_block_state(&block_pos.down()).await;
        if support_block.is_center_solid(BlockDirection::Up) {
            return true;
        }
        for dir in BlockDirection::horizontal() {
            if can_place_at(world.unwrap(), block_pos, dir).await {
                return true;
            }
        }
        false
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state: u16,
        block_pos: &BlockPos,
        direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: u16,
    ) -> u16 {
        if *block == Block::WALL_TORCH || *block == Block::SOUL_WALL_TORCH {
            let props = WallTorchProps::from_state_id(state, block);
            if props.facing.to_block_direction().opposite() == direction
                && !can_place_at(world, block_pos, props.facing.to_block_direction()).await
            {
                return 0;
            }
        } else if direction == BlockDirection::Down {
            let support_block = world.get_block_state(&block_pos.down()).await;
            if !support_block.is_center_solid(BlockDirection::Up) {
                return 0;
            }
        }
        state
    }
}

async fn can_place_at(world: &World, block_pos: &BlockPos, facing: BlockDirection) -> bool {
    world
        .get_block_state(&block_pos.offset(facing.to_offset()))
        .await
        .is_side_solid(facing.opposite())
}
