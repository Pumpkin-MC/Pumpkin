use pumpkin_data::{Block, BlockDirection, BlockId, BlockState, BlockStateId};
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

/// This is for Gold and Iron Pressure Plate
pub struct WeightedPressurePlateBlock;

type PressurePlateProps = pumpkin_data::block_properties::LightWeightedPressurePlateLikeProperties;

impl BlockMetadata for WeightedPressurePlateBlock {
    fn ids() -> Box<[BlockId]> {
        // light = Gold
        // heavy = Iron
        [
            BlockId::LIGHT_WEIGHTED_PRESSURE_PLATE,
            BlockId::HEAVY_WEIGHTED_PRESSURE_PLATE,
        ]
        .into()
    }
}

impl BlockBehaviour for WeightedPressurePlateBlock {
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

impl PressurePlate for WeightedPressurePlateBlock {
    fn get_redstone_output(&self, _block: &Block, state: BlockStateId) -> u8 {
        let props = PressurePlateProps::from_state_id(state);
        props.power
    }

    fn calculate_redstone_output(&self, world: &World, block: &Block, pos: &BlockPos) -> u8 {
        // light = Gold
        // heavy = Iron
        let weight = if block == &Block::LIGHT_WEIGHTED_PRESSURE_PLATE {
            // Gold
            15
        } else {
            // Iron
            150
        };
        let aabb = detection_box_at(pos);
        let len = world.get_entities_at_box(&aabb).len() + world.get_players_at_box(&aabb).len();
        let len = len.min(weight);
        if len > 0 {
            let f = (weight.min(len) / weight) as f32;
            return (f * 15.0).ceil() as u8;
        }
        0
    }

    fn set_redstone_output(&self, block: &Block, state: &BlockState, output: u8) -> BlockStateId {
        let mut props = PressurePlateProps::from_state_id(state.id);
        props.power = output;
        props.to_state_id(block)
    }

    fn tick_rate(&self) -> u8 {
        10
    }
}
