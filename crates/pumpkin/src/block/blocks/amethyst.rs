use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockStateId, FacingExt,
    block_properties::{AmethystClusterLikeProperties, BlockProperties},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::block::{
    BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    RandomTickArgs, blocks::abstract_wall_mounting::WallMountedBlock,
};

const ALL_DIRECTIONS: [BlockDirection; 6] = [
    BlockDirection::Down,
    BlockDirection::Up,
    BlockDirection::North,
    BlockDirection::South,
    BlockDirection::West,
    BlockDirection::East,
];

pub struct AmethystBlock;

impl BlockMetadata for AmethystBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::SMALL_AMETHYST_BUD,
            BlockId::MEDIUM_AMETHYST_BUD,
            BlockId::LARGE_AMETHYST_BUD,
            BlockId::AMETHYST_CLUSTER,
        ]
        .into()
    }
}

impl BlockBehaviour for AmethystBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props =
            AmethystClusterLikeProperties::from_state_id(args.block.default_state.id, args.block);
        props.facing = args.direction.to_facing();
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        // Use the provided direction, or fallback to the current state's direction if missing
        let direction = args
            .direction
            .unwrap_or_else(|| self.get_direction(args.state.id, args.block));

        WallMountedBlock::can_place_at(self, args.block_accessor, args.position, direction)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        WallMountedBlock::get_state_for_neighbor_update(self, args)
    }
}

impl WallMountedBlock for AmethystBlock {
    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection {
        let props = AmethystClusterLikeProperties::from_state_id(state_id, block);
        props.facing.to_block_direction().opposite()
    }
}

#[pumpkin_block("minecraft:budding_amethyst")]
pub struct BuddingAmethystBlock;

impl BlockBehaviour for BuddingAmethystBlock {
    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if rand::rng().random_range(0..5) == 0 {
            let grow_direction = {
                let mut rng = rand::rng();
                ALL_DIRECTIONS[rng.random_range(0..ALL_DIRECTIONS.len())]
            };
            let grow_pos = args.position.offset(grow_direction.to_offset());
            let (relative_block, relative_state) = args.world.get_block_and_state(&grow_pos);
            let relative_state_id = relative_state.id;

            let next_stage_and_water =
                if can_cluster_grow_at_state(relative_block, relative_state_id) {
                    Some((&Block::SMALL_AMETHYST_BUD, relative_block == &Block::WATER))
                } else if relative_block == &Block::SMALL_AMETHYST_BUD {
                    let props = AmethystClusterLikeProperties::from_state_id(
                        relative_state_id,
                        &Block::SMALL_AMETHYST_BUD,
                    );
                    (props.facing == grow_direction.to_facing())
                        .then_some((&Block::MEDIUM_AMETHYST_BUD, props.waterlogged))
                } else if relative_block == &Block::MEDIUM_AMETHYST_BUD {
                    let props = AmethystClusterLikeProperties::from_state_id(
                        relative_state_id,
                        &Block::MEDIUM_AMETHYST_BUD,
                    );
                    (props.facing == grow_direction.to_facing())
                        .then_some((&Block::LARGE_AMETHYST_BUD, props.waterlogged))
                } else if relative_block == &Block::LARGE_AMETHYST_BUD {
                    let props = AmethystClusterLikeProperties::from_state_id(
                        relative_state_id,
                        &Block::LARGE_AMETHYST_BUD,
                    );
                    (props.facing == grow_direction.to_facing())
                        .then_some((&Block::AMETHYST_CLUSTER, props.waterlogged))
                } else {
                    None
                };

            if let Some((next_stage, waterlogged)) = next_stage_and_water {
                let mut target_props = AmethystClusterLikeProperties::default(next_stage);
                target_props.facing = grow_direction.to_facing();
                target_props.waterlogged = waterlogged;
                let target_state_id = target_props.to_state_id(next_stage);
                args.world
                    .set_block_state(&grow_pos, target_state_id, BlockFlags::NOTIFY_ALL);
            }
        }
    }
}

#[must_use]
pub fn can_cluster_grow_at_state(block: &Block, state_id: BlockStateId) -> bool {
    block.default_state.is_air()
        || (block == &Block::WATER && state_id == Block::WATER.default_state.id)
}
