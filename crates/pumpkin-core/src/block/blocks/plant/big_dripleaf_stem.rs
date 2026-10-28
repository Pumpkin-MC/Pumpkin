use std::sync::Arc;

use crate::block::blocks::plant::big_dripleaf::{BigDripleafBlock, is_water_source};
use crate::block::{
    BlockBehaviour, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnScheduledTickArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::{HorizontalFacing, LadderLikeProperties};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockStateId, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

#[pumpkin_block("minecraft:big_dripleaf_stem")]
pub struct BigDripleafStemBlock;

pub type BigDripleafStemLikeProperties = LadderLikeProperties;

impl BigDripleafStemBlock {
    #[must_use]
    pub fn can_survive(block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let below_block = block_accessor.get_block(&pos.down());
        let above_block = block_accessor.get_block(&pos.up());
        (below_block == &Block::BIG_DRIPLEAF_STEM
            || below_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_BIG_DRIPLEAF))
            && (above_block == &Block::BIG_DRIPLEAF_STEM || above_block == &Block::BIG_DRIPLEAF)
    }

    pub fn place(
        world: &Arc<World>,
        pos: &BlockPos,
        waterlogged: bool,
        facing: HorizontalFacing,
    ) -> bool {
        let mut props = BigDripleafStemLikeProperties::default(&Block::BIG_DRIPLEAF_STEM);
        props.waterlogged = waterlogged;
        props.facing = facing;
        world.set_block_state(
            pos,
            props.to_state_id(&Block::BIG_DRIPLEAF_STEM),
            BlockFlags::NOTIFY_ALL,
        );
        true
    }

    fn get_top_connected_block(world: &World, pos: &BlockPos) -> Option<BlockPos> {
        let mut cursor = pos.up();
        while world.is_in_build_limit(cursor) {
            let block = world.get_block(&cursor);
            if block == &Block::BIG_DRIPLEAF_STEM {
                cursor = cursor.up();
            } else if block == &Block::BIG_DRIPLEAF {
                return Some(cursor);
            } else {
                return None;
            }
        }
        None
    }
}

impl BlockBehaviour for BigDripleafStemBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::can_survive(args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if (args.direction == BlockDirection::Down || args.direction == BlockDirection::Up)
            && !Self::can_survive(args.world, args.position)
        {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }
        let props = BigDripleafStemLikeProperties::from_state_id(args.state_id);
        if props.waterlogged {
            args.world.schedule_fluid_tick(
                &Fluid::WATER,
                *args.position,
                Fluid::WATER.flow_speed as u8,
                TickPriority::Normal,
            );
        }
        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !Self::can_survive(args.world.as_ref(), args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        Self::get_top_connected_block(args.world, args.position)
            .is_some_and(|head_pos| BigDripleafBlock::can_grow_into(args.world, &head_pos.up()))
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        if let Some(head_pos) = Self::get_top_connected_block(args.world, args.position) {
            let place_head_pos = head_pos.up();
            let props = BigDripleafStemLikeProperties::from_state_id(args.state_id);
            let head_waterlogged = is_water_source(args.world.as_ref(), &head_pos);
            let above_waterlogged = is_water_source(args.world.as_ref(), &place_head_pos);
            Self::place(args.world, &head_pos, head_waterlogged, props.facing);
            BigDripleafBlock::place(args.world, &place_head_pos, above_waterlogged, props.facing);
        }
    }
}
