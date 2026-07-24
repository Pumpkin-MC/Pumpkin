//! Vanilla torch (26.2 CFR):
//! - `BaseTorchBlock.updateShape`: if neighbour is DOWN and !canSurvive → AIR (**immediate**, no tick)
//! - `WallTorchBlock.updateShape`: if neighbour is opposite of FACING and !canSurvive → AIR
//! - `canSurvive`: floor torch needs center-solid below; wall needs face-sturdy on attach side
//! - Water destroys via fluid replace (`PistonBehavior::Destroy` / canBeReplaced), not random ticks

use crate::block::{BlockFuture, BlockIsReplacing};
use crate::entity::EntityBase;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BlockProperties, Facing};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{Block, FacingExt, HorizontalFacingExt};
use pumpkin_data::{BlockDirection, BlockId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

type WallTorchProps = pumpkin_data::block_properties::WallTorchLikeProperties;

use crate::block::{
    BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnNeighborUpdateArgs, OnPlaceArgs,
};

pub struct TorchBlock;

impl BlockMetadata for TorchBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::TORCH,
            BlockId::SOUL_TORCH,
            BlockId::WALL_TORCH,
            BlockId::SOUL_WALL_TORCH,
            BlockId::COPPER_TORCH,
            BlockId::COPPER_WALL_TORCH,
        ]
        .into()
    }
}

impl TorchBlock {
    fn is_wall(block: &Block) -> bool {
        *block == Block::WALL_TORCH
            || *block == Block::SOUL_WALL_TORCH
            || *block == Block::COPPER_WALL_TORCH
    }

    /// Vanilla `BaseTorchBlock.canSurvive` / wall variant.
    fn can_survive(
        world: &dyn BlockAccessor,
        block: &Block,
        pos: &BlockPos,
        state_id: BlockStateId,
    ) -> bool {
        // Fluid at this cell: torch cannot exist (water washes it out immediately via shape/neighbor).
        if is_fluid_blocking_torch(world, pos) {
            return false;
        }

        if Self::is_wall(block) {
            let props = WallTorchProps::from_state_id(state_id, block);
            let attach = props.facing.to_block_direction().opposite();
            return wall_can_place_at(world, pos, attach);
        }

        let support = world.get_block_state(&pos.down());
        support.is_center_solid(BlockDirection::Up)
    }
}

impl BlockBehaviour for TorchBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            // Note: do not reject fluid here — placement may *replace* water (vanilla).
            // Fluid wash-out is handled after place via updateShape / on_neighbor_update.

            if args.direction == BlockDirection::Down {
                let support_block = args.world.get_block_state(&args.position.down());
                if support_block.is_center_solid(BlockDirection::Up) {
                    return floor_torch_block(args.block).default_state.id;
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
                let support_block = args.world.get_block_state(&args.position.down());
                if support_block.is_center_solid(BlockDirection::Up) {
                    return floor_torch_block(args.block).default_state.id;
                }
            }

            for dir in directions {
                if dir != Facing::Up
                    && dir != Facing::Down
                    && wall_can_place_at(args.world, args.position, dir.to_block_direction())
                {
                    let wall_block = wall_torch_block(args.block);
                    let mut torch_props = WallTorchProps::default(&wall_block);
                    torch_props.facing = dir.opposite().to_horizontal_facing().unwrap();
                    return torch_props.to_state_id(&wall_block);
                }
            }

            let support_block = args.world.get_block_state(&args.position.down());
            if support_block.is_center_solid(BlockDirection::Up) {
                floor_torch_block(args.block).default_state.id
            } else {
                BlockStateId::AIR
            }
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        // Support only — water at the target cell is replaced by place_block, not a soft deny.
        let support_block = args.block_accessor.get_block_state(&args.position.down());
        if support_block.is_center_solid(BlockDirection::Up) {
            return true;
        }
        for dir in BlockDirection::horizontal() {
            if wall_can_place_at(args.block_accessor, args.position, dir.to_block_direction()) {
                return true;
            }
        }
        false
    }

    /// Vanilla `updateShape` — **immediate** AIR when support is gone (no scheduled/random tick).
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            // Fluid pushed into this cell.
            if is_fluid_blocking_torch(args.world, args.position) {
                return BlockStateId::AIR;
            }

            if Self::is_wall(args.block) {
                let props = WallTorchProps::from_state_id(args.state_id, args.block);
                // Vanilla WallTorchBlock.updateShape:
                // directionToNeighbour.getOpposite() == FACING && !canSurvive → AIR
                let facing = props.facing.to_block_direction();
                if args.direction.opposite() == facing
                    && !Self::can_survive(args.world, args.block, args.position, args.state_id)
                {
                    return BlockStateId::AIR;
                }
            } else if args.direction == BlockDirection::Down
                && !Self::can_survive(args.world, args.block, args.position, args.state_id)
            {
                // Vanilla BaseTorchBlock: directionToNeighbour == DOWN && !canSurvive
                return BlockStateId::AIR;
            }
            args.state_id
        })
    }

    /// Backup path: neighborChanged also re-validates canSurvive (Pumpkin dual pipeline).
    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position);
            if args.world.get_block(args.position).id != args.block.id {
                return;
            }
            if !Self::can_survive(args.world.as_ref(), args.block, args.position, state_id) {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        })
    }
}

fn floor_torch_block(item_or_block: &Block) -> &'static Block {
    if *item_or_block == Block::SOUL_TORCH || *item_or_block == Block::SOUL_WALL_TORCH {
        &Block::SOUL_TORCH
    } else if *item_or_block == Block::COPPER_TORCH || *item_or_block == Block::COPPER_WALL_TORCH {
        &Block::COPPER_TORCH
    } else {
        &Block::TORCH
    }
}

fn wall_torch_block(item_or_block: &Block) -> &'static Block {
    if *item_or_block == Block::SOUL_TORCH || *item_or_block == Block::SOUL_WALL_TORCH {
        &Block::SOUL_WALL_TORCH
    } else if *item_or_block == Block::COPPER_TORCH || *item_or_block == Block::COPPER_WALL_TORCH {
        &Block::COPPER_WALL_TORCH
    } else {
        &Block::WALL_TORCH
    }
}

/// Vanilla `WallTorchBlock.canSurvive` — face-sturdy on the attachment side.
fn wall_can_place_at(
    world: &dyn BlockAccessor,
    block_pos: &BlockPos,
    facing: BlockDirection,
) -> bool {
    world
        .get_block_state(&block_pos.offset(facing.to_offset()))
        .is_side_solid(facing.opposite())
}

/// True if this cell is occupied by water/lava (torch cannot survive).
fn is_fluid_blocking_torch(world: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    let state = world.get_block_state(pos);
    if state.is_liquid() {
        return true;
    }
    // Waterlogged / fluid state id
    let id = state.id;
    Fluid::from_state_id(id).is_some_and(|f| f.id != Fluid::EMPTY.id)
}
