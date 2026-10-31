use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};

/// Mirrors vanilla `FlowingFluid.canHoldFluid` for flowing fluid.
///
/// A `LiquidBlockContainer` passes `canHoldAnyFluid`, but its `canPlaceLiquid` rejects flowing
/// fluid, so it never holds it. Vanilla only waterlogs one when source water reaches it through
/// source conversion, which Pumpkin's flow doesn't do yet.
#[must_use]
pub fn can_hold_fluid(block: &Block) -> bool {
    block.has_tag(&tag::Block::MINECRAFT_WASHED_AWAY_BY_FLUIDS) && !is_liquid_block_container(block)
}

/// Blocks implementing vanilla's `LiquidBlockContainer`: every `SimpleWaterloggedBlock`, plus
/// kelp and seagrass.
fn is_liquid_block_container(block: &Block) -> bool {
    block.is_waterloggable()
        || block == &Block::KELP
        || block == &Block::KELP_PLANT
        || block == &Block::SEAGRASS
        || block == &Block::TALL_SEAGRASS
}
