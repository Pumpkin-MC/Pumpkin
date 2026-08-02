use crate::block::BlockBehaviour;
use crate::block::BlockFuture;
use crate::block::BlockMetadata;
use crate::block::CanPlaceAtArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::RandomTickArgs;
use crate::block::blocks::abstract_wall_mounting::WallMountedBlock;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockId;
use pumpkin_data::BlockStateId;
use pumpkin_data::FacingExt;
use pumpkin_data::block_properties::AmethystClusterLikeProperties;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::WaterLikeProperties;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

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
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = AmethystClusterLikeProperties::from_state_id(
                args.block.default_state.id,
                args.block,
            );
            props.facing = args.direction.to_facing().opposite();
            props.waterlogged = args.replacing.water_source();
            props.to_state_id(args.block)
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        // Use the provided direction, or fallback to the current state's direction if missing
        let direction = args
            .direction
            .unwrap_or_else(|| self.get_direction(args.state.id, args.block));

        WallMountedBlock::can_place_at(self, args.block_accessor, args.position, direction)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move { WallMountedBlock::get_state_for_neighbor_update(self, args).await })
    }
}

impl WallMountedBlock for AmethystBlock {
    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection {
        let props = AmethystClusterLikeProperties::from_state_id(state_id, block);
        props.facing.to_block_direction()
    }
}

/// `budding_amethyst`: randomly spawns/grows amethyst buds on an adjacent face.
///
/// Vanilla: `net.minecraft.world.level.block.BuddingAmethystBlock`.
pub struct BuddingAmethystBlock;

impl BlockMetadata for BuddingAmethystBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::BUDDING_AMETHYST].into()
    }
}

/// `BuddingAmethystBlock.canClusterGrowAtState`: air, or a full (source) water block.
fn can_cluster_grow_at_state(block: &Block, state_id: BlockStateId) -> bool {
    if block == &Block::AIR {
        return true;
    }
    if block == &Block::WATER {
        return WaterLikeProperties::from_state_id(state_id, block).level == 0;
    }
    false
}

/// Whether `relative_state`'s fluid is water: either the block itself is a water source/flow,
/// or (for an existing amethyst bud/cluster) its own `waterlogged` property is set. Mirrors
/// `relativeState.getFluidState().is(Fluids.WATER)` used when building the grown block's state.
fn relative_state_is_water(block: &Block, state_id: BlockStateId) -> bool {
    if block == &Block::WATER {
        return true;
    }
    if AmethystClusterLikeProperties::handles_block_id(block.id) {
        return AmethystClusterLikeProperties::from_state_id(state_id, block).waterlogged;
    }
    false
}

/// Returns the amethyst bud/cluster stage `relative_state` would grow into when a budding
/// amethyst block extends toward it from `grow_direction`, or `None` if it can't grow there.
/// Mirrors the if/else-if chain in `BuddingAmethystBlock.randomTick`.
fn next_growth_stage(
    relative_block: &'static Block,
    relative_state_id: BlockStateId,
    grow_direction: BlockDirection,
) -> Option<&'static Block> {
    let facing_matches = |props_block: &Block| -> bool {
        AmethystClusterLikeProperties::from_state_id(relative_state_id, props_block).facing
            == grow_direction.to_facing()
    };
    if can_cluster_grow_at_state(relative_block, relative_state_id) {
        Some(&Block::SMALL_AMETHYST_BUD)
    } else if relative_block == &Block::SMALL_AMETHYST_BUD && facing_matches(relative_block) {
        Some(&Block::MEDIUM_AMETHYST_BUD)
    } else if relative_block == &Block::MEDIUM_AMETHYST_BUD && facing_matches(relative_block) {
        Some(&Block::LARGE_AMETHYST_BUD)
    } else if relative_block == &Block::LARGE_AMETHYST_BUD && facing_matches(relative_block) {
        Some(&Block::AMETHYST_CLUSTER)
    } else {
        None
    }
}

impl BlockBehaviour for BuddingAmethystBlock {
    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // GROWTH_CHANCE = 5 -> `random.nextInt(5) == 0`
            if rand::rng().random_range(0..5) != 0 {
                return;
            }
            let directions = BlockDirection::all();
            let grow_direction = directions[rand::rng().random_range(0..directions.len())];
            let grow_pos = args.position.offset(grow_direction.to_offset());
            let (relative_block, relative_state_id) = args.world.get_block_and_state_id(&grow_pos);

            if let Some(next_stage) =
                next_growth_stage(relative_block, relative_state_id, grow_direction)
            {
                let mut props = AmethystClusterLikeProperties::default(next_stage);
                props.facing = grow_direction.to_facing();
                props.waterlogged = relative_state_is_water(relative_block, relative_state_id);
                args.world
                    .set_block_state(
                        &grow_pos,
                        props.to_state_id(next_stage),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn air_can_grow_small_bud() {
        assert!(can_cluster_grow_at_state(
            &Block::AIR,
            Block::AIR.default_state.id
        ));
    }

    #[test]
    fn non_full_water_and_solid_blocks_cannot_grow() {
        assert!(!can_cluster_grow_at_state(
            &Block::STONE,
            Block::STONE.default_state.id
        ));
    }

    #[test]
    fn next_stage_chain_follows_facing() {
        // A small bud facing Up, approached from Up (grow_direction = Up), advances to medium.
        let mut props = AmethystClusterLikeProperties::default(&Block::SMALL_AMETHYST_BUD);
        props.facing = BlockDirection::Up.to_facing();
        let state_id = props.to_state_id(&Block::SMALL_AMETHYST_BUD);

        assert_eq!(
            next_growth_stage(&Block::SMALL_AMETHYST_BUD, state_id, BlockDirection::Up),
            Some(&Block::MEDIUM_AMETHYST_BUD)
        );
        // Wrong facing direction blocks growth.
        assert_eq!(
            next_growth_stage(&Block::SMALL_AMETHYST_BUD, state_id, BlockDirection::Down),
            None
        );
    }
}
