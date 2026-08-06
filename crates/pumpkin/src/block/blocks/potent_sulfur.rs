use pumpkin_data::block_properties::{
    BlockProperties, PotentSulfurLikeProperties, PotentSulfurState,
};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockId, BlockStateId};
use pumpkin_util::math::position::BlockPos;

use crate::block::entities::potent_sulfur::PotentSulfurBlockEntity;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, GetStateForNeighborUpdateArgs, OnPlaceArgs,
};
use crate::world::World;

/// `net.minecraft.world.level.block.PotentSulfurBlock`.
pub struct PotentSulfurBlock;

/// `PotentSulfurBlock.ALLOWED_WATER_BLOCKS_ABOVE`.
pub const ALLOWED_WATER_BLOCKS_ABOVE: i32 = 4;

impl BlockMetadata for PotentSulfurBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::POTENT_SULFUR].into()
    }
}

/// `FluidState.isSourceOfType(Fluids.WATER)`.
pub fn is_water_source(world: &World, pos: &BlockPos) -> bool {
    let state_id = world.get_block_state_id(pos);
    Fluid::from_state_id(state_id).is_some_and(|fluid| {
        fluid.name == "minecraft:water"
            && fluid.is_source(state_id)
            && fluid
                .states
                .iter()
                .any(|s| s.block_state_id == state_id && s.is_still && s.is_source)
    })
}

/// `PotentSulfurBlockEntity.isGeyserPassableBlock`: air and water are always passable,
/// anything else only if its collision shape is empty.
pub fn is_geyser_passable(world: &World, pos: &BlockPos) -> bool {
    let (block, state) = world.get_block_and_state(pos);
    if state.is_air() || block.id == Block::WATER.id {
        return true;
    }
    state.collision_shapes.is_empty()
}

/// `PotentSulfurBlockEntity.findNoxiousGasSourceBlock`.
///
/// Walks up through the water column above the block and returns the first non-water,
/// passable position, or `None` if the column is blocked or extends past
/// `ALLOWED_WATER_BLOCKS_ABOVE`.
pub fn find_noxious_gas_source_block(world: &World, origin: &BlockPos) -> Option<BlockPos> {
    let max_y = origin.0.y + ALLOWED_WATER_BLOCKS_ABOVE + 1;
    let mut pos = origin.up();
    while pos.0.y <= max_y {
        let is_water_logged = is_water_source(world, &pos);
        let (block, state) = world.get_block_and_state(&pos);
        if !is_water_logged || (block.id != Block::WATER.id && !is_geyser_passable(world, &pos)) {
            if state.is_air() || is_geyser_passable(world, &pos) {
                return Some(pos);
            }
            break;
        }
        pos = pos.up();
    }
    None
}

/// `PotentSulfurBlock.isSourceIfFluid`.
fn is_source_if_fluid(world: &World, pos: &BlockPos) -> bool {
    let state_id = world.get_block_state_id(pos);
    Fluid::from_state_id(state_id).is_none_or(|fluid| fluid.is_source(state_id))
}

/// `PotentSulfurBlock.validBlockState`.
///
/// Picks the state from the water above and the block below. An ERUPTING geyser keeps
/// its state; any transition into the geyser pair from a non-geyser state resets the
/// block entity's countdown.
pub fn valid_block_state(
    world: &World,
    pos: &BlockPos,
    block: &Block,
    state_id: BlockStateId,
) -> BlockStateId {
    let mut props = PotentSulfurLikeProperties::from_state_id(state_id, block);

    if !is_water_source(world, &pos.up()) {
        props.potent_sulfur_state = PotentSulfurState::Dry;
        return props.to_state_id(block);
    }

    let below = pos.down();
    let below_block = world.get_block(&below);

    if below_block.has_tag(&tag::Block::MINECRAFT_CAUSES_CONTINUOUS_GEYSER_ERUPTIONS)
        && is_source_if_fluid(world, &below)
    {
        props.potent_sulfur_state = PotentSulfurState::Continuous;
        return props.to_state_id(block);
    }

    if below_block.has_tag(&tag::Block::MINECRAFT_CAUSES_PERIODIC_GEYSER_ERUPTIONS)
        && is_source_if_fluid(world, &below)
    {
        let is_geyser = matches!(
            props.potent_sulfur_state,
            PotentSulfurState::Erupting | PotentSulfurState::Dormant
        );
        if !is_geyser
            && let Some(entity) = world.get_block_entity(pos)
            && let Some(sulfur) = entity.as_any().downcast_ref::<PotentSulfurBlockEntity>()
        {
            sulfur.reset_countdown();
        }
        if props.potent_sulfur_state != PotentSulfurState::Erupting {
            props.potent_sulfur_state = PotentSulfurState::Dormant;
        }
        return props.to_state_id(block);
    }

    props.potent_sulfur_state = PotentSulfurState::Wet;
    props.to_state_id(block)
}

impl BlockBehaviour for PotentSulfurBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            valid_block_state(
                args.world,
                args.position,
                args.block,
                args.block.default_state.id,
            )
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(
            async move { valid_block_state(args.world, args.position, args.block, args.state_id) },
        )
    }
}
