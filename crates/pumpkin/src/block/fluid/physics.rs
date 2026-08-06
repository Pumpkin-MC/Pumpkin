use pumpkin_data::BlockState;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockStateId, fluid::Fluid, tag};

/// Check if a specific block can be replaced by fluid (based on block properties)
#[must_use]
pub fn can_be_replaced(block_state: &BlockState, block: &Block, fluid: &Fluid) -> bool {
    // An already waterlogged block retains its host state and is not replaceable.
    if block.is_waterlogged(block_state.id) {
        return false;
    }

    // Vanilla `LiquidBlockContainer` accepts water into a dry waterloggable block. The caller
    // must change it to the waterlogged state rather than replace the block with a fluid state.
    if fluid.matches_type(&Fluid::WATER) && block.with_waterlogged(block_state.id).is_some() {
        return true;
    }

    // Fluid Logic
    if let Some(other_fluid) = Fluid::from_state_id(block_state.id) {
        if !fluid.matches_type(other_fluid) {
            return true;
        }
        // Replace current fluid if it is a falling source
        if other_fluid.is_source(block_state.id) && other_fluid.is_falling(block_state.id) {
            return true;
        }
    }

    let id = block.id;

    // Blocks that fluid should never replace
    if block.has_tag(&tag::Block::MINECRAFT_DOORS)
        || block.has_tag(&tag::Block::MINECRAFT_BEDS)
        || block.has_tag(&tag::Block::MINECRAFT_LEAVES)
        || block.has_tag(&tag::Block::MINECRAFT_PRESSURE_PLATES)
        || block.has_tag(&tag::Block::C_CLUSTERS)
        || block.has_tag(&tag::Block::MINECRAFT_WALL_CORALS)
        || block.has_tag(&tag::Block::MINECRAFT_SHULKER_BOXES)
        || block.has_tag(&tag::Block::MINECRAFT_PORTALS)
        || id == Block::BELL.id
        || id == Block::BIG_DRIPLEAF.id
        || id == Block::BIG_DRIPLEAF_STEM.id
        || id == Block::SMALL_DRIPLEAF.id
        || id == Block::CAKE.id
        || id == Block::CONDUIT.id
        || id == Block::CAMPFIRE.id
        || id == Block::DRAGON_EGG.id
        || id == Block::KELP.id
        || id == Block::KELP_PLANT.id
        || id == Block::SEAGRASS.id
        || id == Block::TALL_SEAGRASS.id
        || id == Block::LADDER.id
        || id == Block::POINTED_DRIPSTONE.id
        || id == Block::SCAFFOLDING.id
    {
        return false;
    }

    // Only replace air, explicitly replaceable blocks, or carpets
    block_state.replaceable()
        || id == Block::AIR.id
        || block.has_tag(&tag::Block::MINECRAFT_WOOL_CARPETS)
        // Only use PistonBehavior::Destroy if it didn't pass the checks above
        || block_state.piston_behavior == pumpkin_data::block_state::PistonBehavior::Destroy
}

#[must_use]
pub fn waterlogged_replacement_state(
    block_state: &BlockState,
    block: &Block,
    fluid: &Fluid,
) -> Option<BlockStateId> {
    fluid
        .matches_type(&Fluid::WATER)
        .then(|| block.with_waterlogged(block_state.id))
        .flatten()
        .map(|state| state.id)
}

#[cfg(test)]
mod tests {
    use pumpkin_data::{Block, fluid::Fluid};

    use super::{can_be_replaced, waterlogged_replacement_state};

    #[test]
    fn water_flows_into_dry_waterloggable_blocks_without_replacing_them() {
        let block = &Block::OAK_SLAB;
        let dry = block.default_state;
        let waterlogged_id = waterlogged_replacement_state(dry, block, &Fluid::WATER)
            .expect("oak slabs have a waterlogged state");

        assert!(can_be_replaced(dry, block, &Fluid::WATER));
        assert!(block.is_waterlogged(waterlogged_id));
        assert!(waterlogged_replacement_state(dry, block, &Fluid::LAVA).is_none());
    }
}
