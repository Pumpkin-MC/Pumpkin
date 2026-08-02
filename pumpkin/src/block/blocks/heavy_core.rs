use crate::block::{BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs, OnPlaceArgs};
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{BlockProperties, MangroveRootsLikeProperties};
use pumpkin_data::fluid::Fluid;
use pumpkin_macros::pumpkin_block;
use pumpkin_world::tick::TickPriority;

/// `heavy_core`: a simple waterloggable block used as the crafting/anvil-like core of the mace.
///
/// Vanilla: `net.minecraft.world.level.block.HeavyCoreBlock`. That class only implements
/// `SimpleWaterloggedBlock` plumbing (waterlogged state, a fixed 8x?x8 column shape, and
/// `isPathfindable` returning false) — it contains no mace-related logic whatsoever. The
/// "mace breaks heavy core" interaction lives in the mace item / attack logic elsewhere in
/// vanilla, not in this block class, and there is no such event plumbing in Pumpkin yet
/// (searched for "mace" across the block/entity code, found nothing). That piece is out of
/// scope for this block file and is left undone; only the waterlogged block shell below is
/// implemented, matching everything `HeavyCoreBlock.java` itself actually does.
type HeavyCoreProperties = MangroveRootsLikeProperties;

#[pumpkin_block("minecraft:heavy_core")]
pub struct HeavyCoreBlock;

impl BlockBehaviour for HeavyCoreBlock {
    /// `HeavyCoreBlock.getStateForPlacement`: waterlogged iff placed into a water source.
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = HeavyCoreProperties::default(args.block);
            props.waterlogged = args.replacing.water_source();
            props.to_state_id(args.block)
        })
    }

    /// `HeavyCoreBlock.updateShape`: keep re-scheduling the water fluid tick while waterlogged.
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let props = HeavyCoreProperties::from_state_id(args.state_id, args.block);
            if props.waterlogged {
                args.world.schedule_fluid_tick(
                    &Fluid::WATER,
                    *args.position,
                    Fluid::WATER.flow_speed as u8,
                    TickPriority::Normal,
                );
            }
            args.state_id
        })
    }
}
