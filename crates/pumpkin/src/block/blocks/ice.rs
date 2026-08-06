use std::sync::Arc;

use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockState, dimension::Dimension};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::{BlockBehaviour, BlockFuture, BrokenArgs, RandomTickArgs};
use crate::world::World;

#[pumpkin_block("minecraft:ice")]
pub struct IceBlock;

impl BlockBehaviour for IceBlock {
    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // `IceBlock.randomTick` melts once the block light exceeds
            // `11 - state.getLightBlock()`. Java's `getLightBlock()` is `BlockState::opacity`
            // here, and ice has an opacity of 1, so the effective threshold is a block light
            // level above 10 (one lower than the flat `> 11` used for snow layers).
            let opacity = args.world.get_block_state(args.position).opacity;
            if args.world.get_block_light_level(args.position).unwrap_or(0)
                <= 11u8.saturating_sub(opacity)
            {
                return;
            }

            melt(args.world, args.position).await;
        })
    }

    /// `IceBlock#playerDestroy`: without Silk Touch, breaking ice never drops an ice item and
    /// instead behaves like a melt - evaporating in the Nether, or turning into a water source
    /// if the block below can support it. Silk Touch (tagged `prevents_ice_melting`) suppresses
    /// this entirely, leaving the normal item drop from the loot table intact.
    ///
    /// `ServerPlayerGameMode#destroyBlock` never calls `playerDestroy` for creative players
    /// (`Player#preventsBlockDrops` returns true for `instabuild`), so this hook must reproduce
    /// that guard itself, since Pumpkin's `broken` callback fires on the creative path too.
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if args.player.gamemode.load() == GameMode::Creative {
                return;
            }

            let tool = args.player.inventory.held_item();
            let silk_touched = {
                let tool = tool.lock().await;
                tool.get_data_component::<pumpkin_data::data_component_impl::EnchantmentsImpl>()
                    .is_some_and(|enchantments| {
                        enchantments.enchantment.iter().any(|(enchantment, _)| {
                            enchantment.has_tag(&tag::Enchantment::MINECRAFT_PREVENTS_ICE_MELTING)
                        })
                    })
            };
            if silk_touched {
                return;
            }

            if args.world.dimension == Dimension::THE_NETHER {
                // The block was already replaced with air by `break_block`; nothing further to do.
                return;
            }

            let below_pos = args.position.down();
            let below_state = args.world.get_block_state(&below_pos);
            let below_block = args.world.get_block(&below_pos);
            if supports_water(below_block, below_state) {
                // `level.setBlockAndUpdate(pos, meltsInto())`: unlike `melt`, `playerDestroy`
                // does not also call `neighborChanged` on the new water.
                args.world
                    .set_block_state(
                        args.position,
                        Block::WATER.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }
}

/// `IceBlock#playerDestroy`'s below-block check: `belowState.blocksMotion() || belowState.liquid()`.
/// `blocksMotion()` is legacy-solid minus cobweb/bamboo sapling (`BlockBehaviour.java:542`).
fn supports_water(below_block: &Block, below_state: &BlockState) -> bool {
    below_state.is_liquid()
        || (below_state.is_solid()
            && below_block != &Block::COBWEB
            && below_block != &Block::BAMBOO_SAPLING)
}

/// `IceBlock#melt`: ultrawarm dimensions evaporate the ice, everything else leaves a water
/// source behind. Melting never drops an ice item, so the state is overwritten instead of
/// broken. Shared with `FrostedIceBlock`, which inherits this method unchanged.
pub(super) async fn melt(world: &Arc<World>, position: &BlockPos) {
    if world.dimension == Dimension::THE_NETHER {
        world
            .set_block_state(
                position,
                Block::AIR.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        return;
    }

    world
        .set_block_state(
            position,
            Block::WATER.default_state.id,
            BlockFlags::NOTIFY_ALL,
        )
        .await;
    // `level.neighborChanged(pos, Blocks.WATER, null)`: tell the fresh water source about its
    // own surroundings so that it starts flowing right away.
    world.update_neighbor(position, &Block::WATER).await;
}

#[cfg(test)]
mod tests {
    use super::supports_water;
    use pumpkin_data::Block;

    #[test]
    fn stone_supports_water() {
        assert!(supports_water(&Block::STONE, Block::STONE.default_state));
    }

    #[test]
    fn water_supports_water() {
        assert!(supports_water(&Block::WATER, Block::WATER.default_state));
    }

    #[test]
    fn air_does_not_support_water() {
        assert!(!supports_water(&Block::AIR, Block::AIR.default_state));
    }

    #[test]
    fn cobweb_does_not_support_water() {
        assert!(!supports_water(&Block::COBWEB, Block::COBWEB.default_state));
    }

    #[test]
    fn bamboo_sapling_does_not_support_water() {
        assert!(!supports_water(
            &Block::BAMBOO_SAPLING,
            Block::BAMBOO_SAPLING.default_state
        ));
    }
}
