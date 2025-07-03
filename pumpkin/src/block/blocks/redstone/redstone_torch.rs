use std::sync::Arc;

use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::CanPlaceAtArgs;
use crate::block::pumpkin_block::EmitsRedstonePowerArgs;
use crate::block::pumpkin_block::GetRedstonePowerArgs;
use crate::block::pumpkin_block::GetStateForNeighborUpdateArgs;
use crate::block::pumpkin_block::OnNeighborUpdateArgs;
use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::OnScheduledTickArgs;
use crate::block::pumpkin_block::OnStateReplacedArgs;
use crate::block::pumpkin_block::PlacedArgs;
use crate::entity::EntityBase;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::FacingExt;
use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::Facing;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

type RWallTorchProps = pumpkin_data::block_properties::FurnaceLikeProperties;
type RTorchProps = pumpkin_data::block_properties::RedstoneOreLikeProperties;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::world::World;

use super::get_redstone_power;

pub struct RedstoneTorchBlock;

impl BlockMetadata for RedstoneTorchBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::REDSTONE_TORCH.name, Block::REDSTONE_WALL_TORCH.name]
    }
}

#[async_trait]
impl PumpkinBlock for RedstoneTorchBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let world = args.world;
        let block = args.block;
        let location = args.location;

        if args.direction == BlockDirection::Down {
            let support_block = world.get_block_state(&location.down()).await;
            if support_block.is_center_solid(BlockDirection::Up) {
                return block.default_state.id;
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
            let support_block = world.get_block_state(&location.down()).await;
            if support_block.is_center_solid(BlockDirection::Up) {
                return block.default_state.id;
            }
        }

        for dir in directions {
            if dir != Facing::Up
                && dir != Facing::Down
                && can_place_at(world, location, dir.to_block_direction()).await
            {
                let mut torch_props = RWallTorchProps::default(&Block::REDSTONE_WALL_TORCH);
                torch_props.facing = dir
                    .opposite()
                    .to_block_direction()
                    .to_horizontal_facing()
                    .unwrap();
                return torch_props.to_state_id(&Block::REDSTONE_WALL_TORCH);
            }
        }

        let support_block = world.get_block_state(&location.down()).await;
        if support_block.is_center_solid(BlockDirection::Up) {
            block.default_state.id
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
        if args.block == &Block::REDSTONE_WALL_TORCH {
            let props = RWallTorchProps::from_state_id(args.state_id, args.block);
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

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let state = args.world.get_block_state(args.location).await;

        if args
            .world
            .is_block_tick_scheduled(args.location, args.block)
            .await
        {
            return;
        }

        if args.block == &Block::REDSTONE_WALL_TORCH {
            let props = RWallTorchProps::from_state_id(state.id, args.block);
            if props.lit
                != should_be_lit(
                    args.world,
                    args.location,
                    props.facing.to_block_direction().opposite(),
                )
                .await
            {
                args.world
                    .schedule_block_tick(args.block, *args.location, 2, TickPriority::Normal)
                    .await;
            }
        } else if args.block == &Block::REDSTONE_TORCH {
            let props = RTorchProps::from_state_id(state.id, args.block);
            if props.lit != should_be_lit(args.world, args.location, BlockDirection::Down).await {
                args.world
                    .schedule_block_tick(args.block, *args.location, 2, TickPriority::Normal)
                    .await;
            }
        }
    }

    async fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    async fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if args.block == &Block::REDSTONE_WALL_TORCH {
            let props = RWallTorchProps::from_state_id(args.state.id, args.block);
            if props.lit && args.direction != props.facing.to_block_direction() {
                return 15;
            }
        } else if args.block == &Block::REDSTONE_TORCH {
            let props = RTorchProps::from_state_id(args.state.id, args.block);
            if props.lit && args.direction != BlockDirection::Up {
                return 15;
            }
        }
        0
    }

    async fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if args.direction == BlockDirection::Down {
            if args.block == &Block::REDSTONE_WALL_TORCH {
                let props = RWallTorchProps::from_state_id(args.state.id, args.block);
                if props.lit {
                    return 15;
                }
            } else if args.block == &Block::REDSTONE_TORCH {
                let props = RTorchProps::from_state_id(args.state.id, args.block);
                if props.lit {
                    return 15;
                }
            }
        }
        0
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.location).await;
        if args.block == &Block::REDSTONE_WALL_TORCH {
            let mut props = RWallTorchProps::from_state_id(state.id, args.block);
            let should_be_lit_now = should_be_lit(
                args.world,
                args.location,
                props.facing.to_block_direction().opposite(),
            )
            .await;
            if props.lit != should_be_lit_now {
                props.lit = should_be_lit_now;
                args.world
                    .set_block_state(
                        args.location,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                update_neighbors(args.world, args.location).await;
            }
        } else if args.block == &Block::REDSTONE_TORCH {
            let mut props = RTorchProps::from_state_id(state.id, args.block);
            let should_be_lit_now =
                should_be_lit(args.world, args.location, BlockDirection::Down).await;
            if props.lit != should_be_lit_now {
                props.lit = should_be_lit_now;
                args.world
                    .set_block_state(
                        args.location,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                update_neighbors(args.world, args.location).await;
            }
        }
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        update_neighbors(args.world, args.location).await;
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        update_neighbors(args.world, args.location).await;
    }
}

pub async fn should_be_lit(world: &World, pos: &BlockPos, face: BlockDirection) -> bool {
    let other_pos = pos.offset(face.to_offset());
    let (block, state) = world.get_block_and_block_state(&other_pos).await;
    get_redstone_power(block, state, world, &other_pos, face).await == 0
}

pub async fn update_neighbors(world: &Arc<World>, pos: &BlockPos) {
    for dir in BlockDirection::all() {
        let other_pos = pos.offset(dir.to_offset());
        world.update_neighbors(&other_pos, None).await;
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
