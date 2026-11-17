use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockState, BlockStateId, Mirror, Rotation,
    block_properties::BrownMushroomBlockLikeProperties,
};

use crate::block::{BlockBehaviour, BlockMetadata, GetStateForNeighborUpdateArgs, OnPlaceArgs};

pub struct HugeMushroomBlock;

impl BlockMetadata for HugeMushroomBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::BROWN_MUSHROOM_BLOCK,
            BlockId::RED_MUSHROOM_BLOCK,
            BlockId::MUSHROOM_STEM,
        ]
        .into()
    }
}

impl BlockBehaviour for HugeMushroomBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = BrownMushroomBlockLikeProperties::default(args.block);
        props.down = args.world.get_block(&args.position.down()) != args.block;
        props.up = args.world.get_block(&args.position.up()) != args.block;
        props.north = args
            .world
            .get_block(&args.position.offset(BlockDirection::North.to_offset()))
            != args.block;
        props.east = args
            .world
            .get_block(&args.position.offset(BlockDirection::East.to_offset()))
            != args.block;
        props.south = args
            .world
            .get_block(&args.position.offset(BlockDirection::South.to_offset()))
            != args.block;
        props.west = args
            .world
            .get_block(&args.position.offset(BlockDirection::West.to_offset()))
            != args.block;
        props.to_state_id(args.block)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if Block::from_state_id(args.neighbor_state_id) == args.block {
            let mut props = BrownMushroomBlockLikeProperties::from_state_id(args.state_id);
            match args.direction {
                BlockDirection::Down => props.down = false,
                BlockDirection::Up => props.up = false,
                BlockDirection::North => props.north = false,
                BlockDirection::East => props.east = false,
                BlockDirection::South => props.south = false,
                BlockDirection::West => props.west = false,
            }
            props.to_state_id(args.block)
        } else {
            args.state_id
        }
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        let mut props = BrownMushroomBlockLikeProperties::from_state_id(state_id);
        let (north, east, south, west) = (props.north, props.east, props.south, props.west);
        match rotation {
            Rotation::None => {}
            Rotation::Clockwise90 => {
                props.north = west;
                props.east = north;
                props.south = east;
                props.west = south;
            }
            Rotation::Rotate180 => {
                props.north = south;
                props.east = west;
                props.south = north;
                props.west = east;
            }
            Rotation::CounterClockwise90 => {
                props.north = east;
                props.east = south;
                props.south = west;
                props.west = north;
            }
        }
        BlockState::from_id(props.to_state_id(block))
    }

    fn mirror(&self, block: &Block, state_id: BlockStateId, mirror: Mirror) -> &'static BlockState {
        let mut props = BrownMushroomBlockLikeProperties::from_state_id(state_id);
        match mirror {
            Mirror::None => {}
            Mirror::LeftRight => {
                std::mem::swap(&mut props.north, &mut props.south);
            }
            Mirror::FrontBack => {
                std::mem::swap(&mut props.east, &mut props.west);
            }
        }
        BlockState::from_id(props.to_state_id(block))
    }
}
