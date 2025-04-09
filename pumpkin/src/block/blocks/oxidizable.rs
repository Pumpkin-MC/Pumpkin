use pumpkin_data::block::Block;
use std::collections::HashMap;
use std::iter::Iterator;
use std::sync::LazyLock;

static OXIDATION_INCREASE: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    HashMap::from([
        (Block::COPPER_BLOCK.id, Block::EXPOSED_COPPER.id),
        (Block::EXPOSED_COPPER.id, Block::WEATHERED_COPPER.id),
        (Block::WEATHERED_COPPER.id, Block::OXIDIZED_COPPER.id),
        (Block::CUT_COPPER.id, Block::EXPOSED_CUT_COPPER.id),
        (Block::EXPOSED_CUT_COPPER.id, Block::WEATHERED_CUT_COPPER.id),
        (
            Block::WEATHERED_CUT_COPPER.id,
            Block::OXIDIZED_CUT_COPPER.id,
        ),
        (Block::CHISELED_COPPER.id, Block::EXPOSED_CHISELED_COPPER.id),
        (
            Block::EXPOSED_CHISELED_COPPER.id,
            Block::WEATHERED_CHISELED_COPPER.id,
        ),
        (
            Block::WEATHERED_CHISELED_COPPER.id,
            Block::OXIDIZED_CHISELED_COPPER.id,
        ),
        (Block::CUT_COPPER_SLAB.id, Block::EXPOSED_CUT_COPPER_SLAB.id),
        (
            Block::EXPOSED_CUT_COPPER_SLAB.id,
            Block::WEATHERED_CUT_COPPER_SLAB.id,
        ),
        (
            Block::WEATHERED_CUT_COPPER_SLAB.id,
            Block::OXIDIZED_CUT_COPPER_SLAB.id,
        ),
        (
            Block::CUT_COPPER_STAIRS.id,
            Block::EXPOSED_CUT_COPPER_STAIRS.id,
        ),
        (
            Block::EXPOSED_CUT_COPPER_STAIRS.id,
            Block::WEATHERED_CUT_COPPER_STAIRS.id,
        ),
        (
            Block::WEATHERED_CUT_COPPER_STAIRS.id,
            Block::OXIDIZED_CUT_COPPER_STAIRS.id,
        ),
        (Block::COPPER_DOOR.id, Block::EXPOSED_COPPER_DOOR.id),
        (
            Block::EXPOSED_COPPER_DOOR.id,
            Block::WEATHERED_COPPER_DOOR.id,
        ),
        (
            Block::WEATHERED_COPPER_DOOR.id,
            Block::OXIDIZED_COPPER_DOOR.id,
        ),
        (Block::COPPER_TRAPDOOR.id, Block::EXPOSED_COPPER_TRAPDOOR.id),
        (
            Block::EXPOSED_COPPER_TRAPDOOR.id,
            Block::WEATHERED_COPPER_TRAPDOOR.id,
        ),
        (
            Block::WEATHERED_COPPER_TRAPDOOR.id,
            Block::OXIDIZED_COPPER_TRAPDOOR.id,
        ),
        (Block::COPPER_GRATE.id, Block::EXPOSED_COPPER_GRATE.id),
        (
            Block::EXPOSED_COPPER_GRATE.id,
            Block::WEATHERED_COPPER_GRATE.id,
        ),
        (
            Block::WEATHERED_COPPER_GRATE.id,
            Block::OXIDIZED_COPPER_GRATE.id,
        ),
        (Block::COPPER_BULB.id, Block::EXPOSED_COPPER_BULB.id),
        (
            Block::EXPOSED_COPPER_BULB.id,
            Block::WEATHERED_COPPER_BULB.id,
        ),
        (
            Block::WEATHERED_COPPER_BULB.id,
            Block::OXIDIZED_COPPER_BULB.id,
        ),
    ])
});
static OXIDATION_DECREASE: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    OXIDATION_INCREASE
        .iter()
        .map(|(k, v)| (*v, *k))
        .collect::<HashMap<_, _>>()
});
pub trait Oxidizable {
    fn decrease_oxidation(&self, block: &Block) -> Option<Block>;
}
pub struct Oxidation;
impl Oxidizable for Oxidation {
    fn decrease_oxidation(&self, block: &Block) -> Option<Block> {
        let replacement_block = OXIDATION_DECREASE.get(&block.id);
        // Clippy made me do this
        if let Some(item) = replacement_block {
            return Block::from_id(*item);
        }
        None
    }
}
