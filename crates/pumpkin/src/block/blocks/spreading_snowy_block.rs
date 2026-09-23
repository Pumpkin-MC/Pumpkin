use std::sync::Arc;

use pumpkin_data::block_properties::{
    GrassBlockLikeProperties, SnowLikeProperties, WaterLikeProperties,
};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection, BlockId, BlockState, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::lighting::LightEngine;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::block::{
    BlockBehaviour, BlockMetadata, GetStateForNeighborUpdateArgs, OnPlaceArgs, RandomTickArgs,
};
use crate::world::World;

/// Base logic for snowy blocks, matching vanilla `net.minecraft.world.level.block.SnowyBlock`.
pub struct SnowyBlock;

impl SnowyBlock {
    #[must_use]
    pub fn is_snowy_setting(above_state: &BlockState) -> bool {
        above_state
            .id
            .to_block()
            .has_tag(&tag::Block::MINECRAFT_SNOW)
    }

    #[must_use]
    pub fn on_place(block: &Block, world: &World, position: &BlockPos) -> BlockStateId {
        let block_above = world.get_block(&position.up());
        let mut props = GrassBlockLikeProperties::from_state_id(block.default_state.id);
        props.snowy = block_above.has_tag(&tag::Block::MINECRAFT_SNOW);
        props.to_state_id(block)
    }

    #[must_use]
    pub fn get_state_for_neighbor_update(args: &GetStateForNeighborUpdateArgs<'_>) -> BlockStateId {
        if args.direction == BlockDirection::Up {
            let block_above = args.world.get_block(args.neighbor_position);
            let mut props = GrassBlockLikeProperties::from_state_id(args.state_id);
            let should_be_snowy = block_above.has_tag(&tag::Block::MINECRAFT_SNOW);
            if props.snowy == should_be_snowy {
                return args.state_id;
            }
            props.snowy = should_be_snowy;
            return props.to_state_id(args.block);
        }
        args.state_id
    }
}

/// Abstract base logic for spreading snowy blocks (e.g. Grass Block, Mycelium),
/// matching vanilla `net.minecraft.world.level.block.SpreadingSnowyBlock`.
pub struct SpreadingSnowyBlock;

impl SpreadingSnowyBlock {
    #[must_use]
    pub fn is_full_fluid(world: &World, pos: &BlockPos) -> bool {
        let state = world.get_block_state(pos);
        let block = state.id.to_block();
        if block == &Block::WATER || block == &Block::LAVA {
            return WaterLikeProperties::from_state_id(state.id).level == 0;
        }
        state.id.is_waterlogged()
    }

    #[must_use]
    pub fn is_water_fluid(world: &World, pos: &BlockPos) -> bool {
        let state = world.get_block_state(pos);
        let block = state.id.to_block();
        block == &Block::WATER || state.id.is_waterlogged()
    }

    #[must_use]
    pub fn can_stay_alive_with_above(
        state: &BlockState,
        above_state: &BlockState,
        is_full_fluid: bool,
    ) -> bool {
        if above_state.id.to_block() == &Block::SNOW {
            let props = SnowLikeProperties::from_state_id(above_state.id);
            if props.layers == 1 {
                return true;
            }
        }

        if is_full_fluid {
            return false;
        }

        let light_dampening_top_face = LightEngine::get_light_dampening_into(
            state,
            above_state,
            BlockDirection::Up,
            above_state.opacity,
        );
        light_dampening_top_face < 15
    }

    #[must_use]
    pub fn can_stay_alive(state: &BlockState, world: &World, pos: &BlockPos) -> bool {
        let above = pos.up();
        if !world.is_loaded(&above) {
            return true;
        }
        let above_state = world.get_block_state(&above);
        let is_full_fluid = Self::is_full_fluid(world, &above);
        Self::can_stay_alive_with_above(state, above_state, is_full_fluid)
    }

    #[must_use]
    pub fn can_propagate(state: &BlockState, world: &World, pos: &BlockPos) -> bool {
        let above = pos.up();
        if !world.is_loaded(&above) {
            return false;
        }
        Self::can_stay_alive(state, world, pos) && !Self::is_water_fluid(world, &above)
    }

    pub fn random_tick(
        state: &BlockState,
        world: &Arc<World>,
        pos: &BlockPos,
        base_block: &'static Block,
        default_block_state: &'static BlockState,
    ) {
        if !Self::can_stay_alive(state, world, pos) {
            world.set_block_state(pos, base_block.default_state.id, BlockFlags::NOTIFY_ALL);
        } else if world.get_max_local_raw_brightness(&pos.up()) >= 9 {
            let mut rng = rand::rng();
            for _ in 0..4 {
                let dx = rng.random_range(0..3) - 1;
                let dy = rng.random_range(0..5) - 3;
                let dz = rng.random_range(0..3) - 1;
                let test_pos = pos.add(dx, dy, dz);
                if !world.is_loaded(&test_pos) {
                    continue;
                }
                if world.get_block(&test_pos) == base_block
                    && Self::can_propagate(default_block_state, world, &test_pos)
                {
                    let above_test = test_pos.up();
                    let is_snowy = if world.is_loaded(&above_test) {
                        SnowyBlock::is_snowy_setting(world.get_block_state(&above_test))
                    } else {
                        false
                    };
                    let mut props = GrassBlockLikeProperties::from_state_id(default_block_state.id);
                    props.snowy = is_snowy;
                    let new_state_id = props.to_state_id(default_block_state.id.to_block());
                    world.set_block_state(&test_pos, new_state_id, BlockFlags::NOTIFY_ALL);
                }
            }
        }
    }
}

/// Podzol block, matching vanilla `net.minecraft.world.level.block.SnowyBlock` with `BlockItemIds.PODZOL`.
pub struct PodzolBlock;

impl BlockMetadata for PodzolBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::PODZOL].into()
    }
}

impl BlockBehaviour for PodzolBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        SnowyBlock::on_place(args.block, args.world, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        SnowyBlock::get_state_for_neighbor_update(&args)
    }
}

/// Mycelium block, matching vanilla `net.minecraft.world.level.block.MyceliumBlock`.
pub struct MyceliumBlock;

impl BlockMetadata for MyceliumBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::MYCELIUM].into()
    }
}

impl BlockBehaviour for MyceliumBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        SnowyBlock::on_place(args.block, args.world, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        SnowyBlock::get_state_for_neighbor_update(&args)
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        SpreadingSnowyBlock::random_tick(
            state,
            args.world,
            args.position,
            &Block::DIRT,
            Block::MYCELIUM.default_state,
        );
    }
}
