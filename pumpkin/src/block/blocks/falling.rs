use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockIsReplacing, BlockMetadata,
        GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
    },
    entity::falling::FallingEntity,
};
use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockState, BlockStateId,
    tag::{self, Taggable},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;

pub struct FallingBlock;

impl FallingBlock {
    #[must_use]
    pub fn can_fall_through(state: &BlockState, block: &Block) -> bool {
        state.is_air()
            || block.has_tag(&tag::Block::MINECRAFT_FIRE)
            || state.is_liquid()
            || state.replaceable()
    }
}

impl BlockMetadata for FallingBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::GRAVEL, BlockId::SAND, BlockId::RED_SAND].into()
    }
}

impl BlockBehaviour for FallingBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // TODO: make delay configurable
            args.world
                .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal);
        })
    }
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            // TODO: make delay configurable
            args.world
                .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal);
            args.state_id
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let (block, state) = args.world.get_block_and_state(&args.position.down());
            if !Self::can_fall_through(state, block) || args.position.0.y < args.world.min_y {
                return;
            }
            let state = args.world.get_block_state(args.position);
            FallingEntity::replace_spawn(args.world, *args.position, state.id).await;
        })
    }
}

/// `concrete_powder`: falls exactly like sand/gravel, but solidifies into its matching solid
/// concrete block on contact with water.
///
/// Vanilla: `net.minecraft.world.level.block.ConcretePowderBlock`.
pub struct ConcretePowderBlock;

impl BlockMetadata for ConcretePowderBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::WHITE_CONCRETE_POWDER,
            BlockId::ORANGE_CONCRETE_POWDER,
            BlockId::MAGENTA_CONCRETE_POWDER,
            BlockId::LIGHT_BLUE_CONCRETE_POWDER,
            BlockId::YELLOW_CONCRETE_POWDER,
            BlockId::LIME_CONCRETE_POWDER,
            BlockId::PINK_CONCRETE_POWDER,
            BlockId::GRAY_CONCRETE_POWDER,
            BlockId::LIGHT_GRAY_CONCRETE_POWDER,
            BlockId::CYAN_CONCRETE_POWDER,
            BlockId::PURPLE_CONCRETE_POWDER,
            BlockId::BLUE_CONCRETE_POWDER,
            BlockId::BROWN_CONCRETE_POWDER,
            BlockId::GREEN_CONCRETE_POWDER,
            BlockId::RED_CONCRETE_POWDER,
            BlockId::BLACK_CONCRETE_POWDER,
        ]
        .into()
    }
}

/// Maps a `..._concrete_powder` block to its solid `..._concrete` counterpart by name, mirroring
/// the `concrete` field vanilla's `ConcretePowderBlock` is constructed with per color.
#[must_use]
pub fn concrete_for_powder(block: &Block) -> Option<&'static Block> {
    block
        .name
        .strip_suffix("_powder")
        .and_then(Block::from_name)
}

/// `ConcretePowderBlock.canSolidify`: `state.getFluidState().is(FluidTags.WATER)`.
fn can_solidify(state: &BlockState) -> bool {
    state.is_waterlogged() || Block::from_state_id(state.id) == &Block::WATER
}

/// `ConcretePowderBlock.touchesLiquid`. Note the quirk faithfully ported from vanilla: due to how
/// the Java loop reuses a single mutable cursor starting at `pos`, the UP/NORTH/SOUTH/WEST/EAST
/// neighbors are always examined, but the DOWN neighbor is only examined when `pos` itself
/// (the block being replaced) currently contains water.
fn touches_liquid(accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    for direction in [
        BlockDirection::Up,
        BlockDirection::North,
        BlockDirection::South,
        BlockDirection::West,
        BlockDirection::East,
    ] {
        let neighbor_pos = pos.offset(direction.to_offset());
        let neighbor_state = accessor.get_block_state(&neighbor_pos);
        if can_solidify(neighbor_state) && !neighbor_state.is_side_solid(direction.opposite()) {
            return true;
        }
    }

    if can_solidify(accessor.get_block_state(pos)) {
        let down_pos = pos.offset(BlockDirection::Down.to_offset());
        let down_state = accessor.get_block_state(&down_pos);
        if can_solidify(down_state) && !down_state.is_side_solid(BlockDirection::Up) {
            return true;
        }
    }

    false
}

/// `ConcretePowderBlock.shouldSolidify`.
fn should_solidify(accessor: &dyn BlockAccessor, pos: &BlockPos, replaced_is_water: bool) -> bool {
    replaced_is_water || touches_liquid(accessor, pos)
}

/// Called when a falling block entity lands.
///
/// Mirrors `FallingBlock.onLand` (a no-op for plain sand/gravel) and its override in
/// `ConcretePowderBlock.onLand`. `replaced_state` is whatever block currently occupies the
/// landing position (before it gets overwritten).
#[must_use]
pub fn on_land_state(
    accessor: &dyn BlockAccessor,
    pos: &BlockPos,
    falling_state_id: BlockStateId,
    replaced_state: &BlockState,
) -> BlockStateId {
    let falling_block = Block::from_state_id(falling_state_id);
    let Some(concrete) = concrete_for_powder(falling_block) else {
        return falling_state_id;
    };
    if should_solidify(accessor, pos, can_solidify(replaced_state)) {
        concrete.default_state.id
    } else {
        falling_state_id
    }
}

impl BlockBehaviour for ConcretePowderBlock {
    /// `ConcretePowderBlock.getStateForPlacement`.
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let replaced_is_water = matches!(args.replacing, BlockIsReplacing::Water(_));
            if should_solidify(args.world, args.position, replaced_is_water)
                && let Some(concrete) = concrete_for_powder(args.block)
            {
                return concrete.default_state.id;
            }
            args.block.default_state.id
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world
                .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal);
        })
    }

    /// `ConcretePowderBlock.updateShape`.
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if touches_liquid(args.world, args.position)
                && let Some(concrete) = concrete_for_powder(args.block)
            {
                return concrete.default_state.id;
            }
            args.world
                .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal);
            args.state_id
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let (block, state) = args.world.get_block_and_state(&args.position.down());
            if !FallingBlock::can_fall_through(state, block) || args.position.0.y < args.world.min_y
            {
                return;
            }
            let state = args.world.get_block_state(args.position);
            FallingEntity::replace_spawn(args.world, *args.position, state.id).await;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concrete_for_powder_maps_every_color() {
        assert_eq!(
            concrete_for_powder(&Block::WHITE_CONCRETE_POWDER),
            Some(&Block::WHITE_CONCRETE)
        );
        assert_eq!(
            concrete_for_powder(&Block::BLACK_CONCRETE_POWDER),
            Some(&Block::BLACK_CONCRETE)
        );
    }

    #[test]
    fn concrete_for_powder_rejects_unrelated_blocks() {
        assert_eq!(concrete_for_powder(&Block::SAND), None);
        assert_eq!(concrete_for_powder(&Block::WHITE_CONCRETE), None);
    }

    #[test]
    fn can_solidify_matches_water_block() {
        assert!(can_solidify(Block::WATER.default_state));
        assert!(!can_solidify(Block::STONE.default_state));
    }
}
