use crate::block::BlockIsReplacing;
use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::Tagable;
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;

type IronBarsProperties = pumpkin_data::block_properties::OakFenceLikeProperties;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::server::Server;
use crate::world::World;

#[pumpkin_block("minecraft:iron_bars")]
pub struct IronBarsBlock;

#[async_trait]
impl PumpkinBlock for IronBarsBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        block: &Block,
        _face: BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player: &Player,
        replacing: BlockIsReplacing,
    ) -> u16 {
        let mut bars_props = IronBarsProperties::default(block);
        bars_props.waterlogged = replacing.water_source();

        compute_bars_state(bars_props, world, block, block_pos).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        _direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        let bars_props = IronBarsProperties::from_state_id(state_id, block);
        compute_bars_state(bars_props, world, block, block_pos).await
    }
}

pub async fn compute_bars_state(
    mut bars_props: IronBarsProperties,
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
) -> u16 {
    for direction in BlockDirection::horizontal() {
        let other_block_pos = block_pos.offset(direction.to_offset());
        let Ok((other_block, other_block_state)) =
            world.get_block_and_block_state(&other_block_pos).await
        else {
            continue;
        };

        let connected = other_block == *block
            || (other_block_state.is_solid() && other_block_state.is_full_cube())
            || other_block.is_tagged_with("c:glass_panes").unwrap()
            || other_block.is_tagged_with("minecraft:walls").unwrap();

        match direction {
            BlockDirection::North => bars_props.north = connected,
            BlockDirection::South => bars_props.south = connected,
            BlockDirection::West => bars_props.west = connected,
            BlockDirection::East => bars_props.east = connected,
            _ => {}
        }
    }

    bars_props.to_state_id(block)
}
