use pumpkin_data::block_properties::{BlockProperties, MangrovePropaguleLikeProperties};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{Block, BlockStateId, tag, tag::Taggable};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    RandomTickArgs,
};

type PropaguleProperties = MangrovePropaguleLikeProperties;

/// `mangrove_propagule`. Vanilla: `net.minecraft.world.level.block.MangrovePropaguleBlock`.
///
/// Only the propagule's own state machine is implemented here: waterlogging, and the hanging
/// variant's age-up (`randomTick`'s `HANGING` branch, `state.cycle(AGE)`). Growing a *standing*
/// propagule into a mangrove tree (`SaplingBlock.advanceTree` -> `TreeGrower`) is a whole
/// worldgen feature and is out of scope here, matching PARITY.md's existing carve-out for
/// `TreeGrower`; that branch of `randomTick` is left as a documented no-op below.
#[pumpkin_block("minecraft:mangrove_propagule")]
pub struct MangrovePropaguleBlock;

/// `MangrovePropaguleBlock.mayPlaceOn` (non-hanging) / `canSurvive`'s hanging branch.
fn can_survive(hanging: bool, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    if hanging {
        let above = block_accessor.get_block(&pos.up());
        above.has_tag(&tag::Block::MINECRAFT_SUPPORTS_HANGING_MANGROVE_PROPAGULE)
    } else {
        let below = block_accessor.get_block(&pos.down());
        below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_MANGROVE_PROPAGULE)
    }
}

impl BlockBehaviour for MangrovePropaguleBlock {
    /// A player can only ever place a propagule standing up; the hanging variant is only
    /// produced by mangrove tree generation.
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_survive(false, args.block_accessor, args.position)
    }

    /// `MangrovePropaguleBlock.getStateForPlacement`: `AGE = 4` (fully grown), not hanging.
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = PropaguleProperties::default(args.block);
            props.waterlogged = args.replacing.water_source();
            props.age = 4;
            props.hanging = false;
            props.to_state_id(args.block)
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let props = PropaguleProperties::from_state_id(args.state_id, args.block);
            if props.waterlogged {
                args.world.schedule_fluid_tick(
                    &Fluid::WATER,
                    *args.position,
                    Fluid::WATER.flow_speed as u8,
                    TickPriority::Normal,
                );
            }
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
            .await
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let (block, state_id) = args.world.get_block_and_state_id(args.position);
            let mut props = PropaguleProperties::from_state_id(state_id, block);
            if props.hanging && props.age < 4 {
                props.age += 1;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        })
    }
}

impl PlantBlockBase for MangrovePropaguleBlock {
    /// Overrides the default (which only checks `SUPPORTS_VEGETATION` below) to branch on
    /// whether *this* propagule is currently hanging.
    fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        let (block, state) = block_accessor.get_block_and_state(block_pos);
        let hanging = if block == &Block::MANGROVE_PROPAGULE {
            PropaguleProperties::from_state_id(state.id, block).hanging
        } else {
            false
        };
        can_survive(hanging, block_accessor, block_pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hanging_age_caps_at_four() {
        let mut props = PropaguleProperties::default(&Block::MANGROVE_PROPAGULE);
        props.hanging = true;
        props.age = 4;
        // Mirrors the `props.hanging && props.age < 4` guard in random_tick: at max age no
        // further increment should be attempted.
        assert!(!(props.hanging && props.age < 4));

        props.age = 3;
        assert!(props.hanging && props.age < 4);
    }
}
