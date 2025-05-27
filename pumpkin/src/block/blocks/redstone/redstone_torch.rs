use std::sync::Arc;

use crate::block::BlockIsReplacing;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockState;
use pumpkin_data::FacingExt;
use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::Facing;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

type RWallTorchProps = pumpkin_data::block_properties::FurnaceLikeProperties;
type RTorchProps = pumpkin_data::block_properties::RedstoneOreLikeProperties;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::server::Server;
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
                let mut torch_props = RWallTorchProps::default(&Block::REDSTONE_WALL_TORCH);
                torch_props.facing = dir
                    .opposite()
                    .to_block_direction()
                    .to_horizontal_facing()
                    .unwrap();
                return torch_props.to_state_id(&Block::REDSTONE_WALL_TORCH);
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
        _block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        let world = world.unwrap();
        let support_block = world.get_block_state(&block_pos.down()).await;
        if support_block.is_center_solid(BlockDirection::Up) {
            return true;
        }
        for dir in BlockDirection::horizontal() {
            if can_place_at(world, block_pos, dir).await {
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
        if *block == Block::REDSTONE_WALL_TORCH {
            let props = RWallTorchProps::from_state_id(state, block);
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

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        let state = world.get_block_state(block_pos).await;

        if world.is_block_tick_scheduled(block_pos, block).await {
            return;
        }

        if block == &Block::REDSTONE_WALL_TORCH {
            let props = RWallTorchProps::from_state_id(state.id, block);
            if props.lit
                != should_be_lit(
                    world,
                    block_pos,
                    props.facing.to_block_direction().opposite(),
                )
                .await
            {
                world
                    .schedule_block_tick(block, *block_pos, 2, TickPriority::Normal)
                    .await;
            }
        } else if block == &Block::REDSTONE_TORCH {
            let props = RTorchProps::from_state_id(state.id, block);
            if props.lit != should_be_lit(world, block_pos, BlockDirection::Down).await {
                world
                    .schedule_block_tick(block, *block_pos, 2, TickPriority::Normal)
                    .await;
            }
        }
    }

    async fn emits_redstone_power(
        &self,
        _block: &Block,
        _state: &BlockState,
        _direction: BlockDirection,
    ) -> bool {
        true
    }

    async fn get_weak_redstone_power(
        &self,
        block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        if block == &Block::REDSTONE_WALL_TORCH {
            let props = RWallTorchProps::from_state_id(state.id, block);
            if props.lit && direction != props.facing.to_block_direction() {
                return 15;
            }
        } else if block == &Block::REDSTONE_TORCH {
            let props = RTorchProps::from_state_id(state.id, block);
            if props.lit && direction != BlockDirection::Up {
                return 15;
            }
        }
        0
    }

    async fn get_strong_redstone_power(
        &self,
        block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        if direction == BlockDirection::Down {
            if block == &Block::REDSTONE_WALL_TORCH {
                let props = RWallTorchProps::from_state_id(state.id, block);
                if props.lit {
                    return 15;
                }
            } else if block == &Block::REDSTONE_TORCH {
                let props = RTorchProps::from_state_id(state.id, block);
                if props.lit {
                    return 15;
                }
            }
        }
        0
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, block: &Block, block_pos: &BlockPos) {
        let state = world.get_block_state(block_pos).await;
        if block == &Block::REDSTONE_WALL_TORCH {
            let mut props = RWallTorchProps::from_state_id(state.id, block);
            let should_be_lit_now = should_be_lit(
                world,
                block_pos,
                props.facing.to_block_direction().opposite(),
            )
            .await;
            if props.lit != should_be_lit_now {
                props.lit = should_be_lit_now;
                world
                    .set_block_state(block_pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                    .await;
                update_neighbors(world, block_pos).await;
            }
        } else if block == &Block::REDSTONE_TORCH {
            let mut props = RTorchProps::from_state_id(state.id, block);
            let should_be_lit_now = should_be_lit(world, block_pos, BlockDirection::Down).await;
            if props.lit != should_be_lit_now {
                props.lit = should_be_lit_now;
                world
                    .set_block_state(block_pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                    .await;
                update_neighbors(world, block_pos).await;
            }
        }
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        _state_id: BlockStateId,
        block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        update_neighbors(world, block_pos).await;
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        _block: &Block,
        location: BlockPos,
        _old_state_id: BlockStateId,
        _moved: bool,
    ) {
        update_neighbors(world, &location).await;
    }
}

pub async fn should_be_lit(world: &World, pos: &BlockPos, face: BlockDirection) -> bool {
    let other_pos = pos.offset(face.to_offset());
    let (block, state) = world.get_block_and_block_state(&other_pos).await;
    get_redstone_power(&block, &state, world, &other_pos, face).await == 0
}

pub async fn update_neighbors(world: &Arc<World>, pos: &BlockPos) {
    for dir in BlockDirection::all() {
        let other_pos = pos.offset(dir.to_offset());
        world.update_neighbors(&other_pos, None).await;
    }
}

async fn can_place_at(world: &World, block_pos: &BlockPos, facing: BlockDirection) -> bool {
    world
        .get_block_state(&block_pos.offset(facing.to_offset()))
        .await
        .is_side_solid(facing.opposite())
}
