use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockState, BlockStateId,
    block_properties::BlockProperties,
    tag::{self},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::{
        BlockBehaviour, BlockMetadata, CanPlaceAtArgs, EmitsRedstonePowerArgs,
        GetRedstonePowerArgs, OnEntityCollisionArgs, OnNeighborUpdateArgs, OnScheduledTickArgs,
        OnStateReplacedArgs,
    },
    world::World,
};

use super::{PressurePlate, detection_box_at};

/// This is for Normal Pressure plates, so not Gold or Iron
pub struct PressurePlateBlock;

type PressurePlateProps = pumpkin_data::block_properties::StonePressurePlateLikeProperties;

impl BlockMetadata for PressurePlateBlock {
    fn ids() -> Box<[BlockId]> {
        let mut combined = Vec::new();
        combined.extend_from_slice(tag::Block::MINECRAFT_WOODEN_PRESSURE_PLATES.1);
        combined.extend_from_slice(tag::Block::MINECRAFT_STONE_PRESSURE_PLATES.1);
        combined.iter().map(|v| BlockId::new_or_air(*v)).collect()
    }
}

impl BlockBehaviour for PressurePlateBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        self.on_entity_collision_pp(args);
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        let output = self.get_redstone_output(args.block, state.id);
        if output > 0 {
            let (block, state) = args.world.get_block_and_state(args.position);
            Self.update_plate_state(args.world, args.position, block, state, output);
        }
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        self.on_state_replaced_pp(args);
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        self.get_redstone_output(args.block, args.state.id)
    }

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if args.direction == BlockDirection::Up {
            return self.get_redstone_output(args.block, args.state.id);
        }
        0
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if !Self::can_pressure_plate_place_at(args.world, args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        args.world
            .is_some_and(|world| Self::can_pressure_plate_place_at(world, args.position))
    }
}

impl PressurePlate for PressurePlateBlock {
    fn get_redstone_output(&self, block: &Block, state: BlockStateId) -> u8 {
        let props = PressurePlateProps::from_state_id(state, block);
        if props.powered { 15 } else { 0 }
    }

    fn calculate_redstone_output(&self, world: &World, _block: &Block, pos: &BlockPos) -> u8 {
        let aabb = detection_box_at(pos);
        if !world.get_entities_at_box(&aabb).is_empty()
            || !world.get_players_at_box(&aabb).is_empty()
        {
            return 15;
        }
        0
    }

    fn set_redstone_output(&self, block: &Block, state: &BlockState, output: u8) -> BlockStateId {
        let mut props = PressurePlateProps::from_state_id(state.id, block);
        props.powered = output > 0;
        props.to_state_id(block)
    }
}
