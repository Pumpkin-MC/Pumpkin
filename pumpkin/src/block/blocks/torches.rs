use crate::block::BlockIsReplacing;
use crate::entity::EntityBase;
use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::{Block, FacingExt, HorizontalFacingExt};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockAccessor;

type WallTorchProps = pumpkin_data::block_properties::WallTorchLikeProperties;
// Normal tourches don't have properties

use crate::block::pumpkin_block::{
    BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs, PumpkinBlock,
};

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
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.direction == BlockDirection::Down {
            let support_block = args.world.get_block_state(&args.location.down()).await;
            if support_block.is_center_solid(BlockDirection::Up) {
                return args.block.default_state.id;
            }
        }
        let mut directions = args.player.get_entity().get_entity_facing_order();

        if args.replacing == BlockIsReplacing::None {
            let face = args.direction.to_facing();
            let mut i = 0;
            while i < directions.len() && directions[i] != face {
                i += 1;
            }

            if i > 0 {
                directions.copy_within(0..i, 1);
                directions[0] = face;
            }
        } else if directions[0] == Facing::Down {
            let support_block = args.world.get_block_state(&args.location.down()).await;
            if support_block.is_center_solid(BlockDirection::Up) {
                return args.block.default_state.id;
            }
        }

        for dir in directions {
            if dir != Facing::Up
                && dir != Facing::Down
                && can_place_at(args.world, args.location, dir.to_block_direction()).await
            {
                let wall_block = if args.block == &Block::TORCH {
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

        let support_block = args.world.get_block_state(&args.location.down()).await;
        if support_block.is_center_solid(BlockDirection::Up) {
            args.block.default_state.id
        } else {
            0
        }
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let support_block = args
            .block_accessor
            .get_block_state(&args.location.down())
            .await;
        if support_block.is_center_solid(BlockDirection::Up) {
            return true;
        }
        for dir in BlockDirection::horizontal() {
            if can_place_at(args.block_accessor, args.location, dir).await {
                return true;
            }
        }
        false
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.block == &Block::WALL_TORCH || args.block == &Block::SOUL_WALL_TORCH {
            let props = WallTorchProps::from_state_id(args.state_id, args.block);
            if props.facing.to_block_direction().opposite() == args.direction
                && !can_place_at(args.world, args.location, props.facing.to_block_direction()).await
            {
                return 0;
            }
        } else if args.direction == BlockDirection::Down {
            let support_block = args.world.get_block_state(&args.location.down()).await;
            if !support_block.is_center_solid(BlockDirection::Up) {
                return 0;
            }
        }
        args.state_id
    }
}

async fn can_place_at(
    world: &dyn BlockAccessor,
    block_pos: &BlockPos,
    facing: BlockDirection,
) -> bool {
    world
        .get_block_state(&block_pos.offset(facing.to_offset()))
        .await
        .is_side_solid(facing.opposite())
}
