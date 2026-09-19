use std::sync::Arc;

use crate::block::UseWithItemArgs;
use crate::block::entities::BlockEntity;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{BlockDirection, BlockId, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct HoneyCombItem;

impl ItemMetadata for HoneyCombItem {
    fn ids() -> Box<[u16]> {
        [Item::HONEYCOMB.id].into()
    }
}

impl ItemBehaviour for HoneyCombItem {
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        let world = player.world();
        if try_wax_block(&world, location, block) {
            item.decrement_unless_creative(player.gamemode.load(), 1);
            BlockActionResult::Success
        } else {
            BlockActionResult::Pass
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// The state `block` should become once waxed into `waxed`.
///
/// Vanilla waxes with `Block.withPropertiesOf`, which carries every shared property over
/// rather than special-casing each family: trapdoors keep their facing and open state,
/// stairs their shape, slabs their half, bulbs their lit state, grates their waterlogging.
fn waxed_state_id(block: &Block, state_id: BlockStateId, waxed: &'static Block) -> BlockStateId {
    if waxed.states.len() <= 1 {
        return waxed.default_state.id;
    }
    block
        .properties(state_id)
        .map_or(waxed.default_state.id, |props| {
            waxed.from_properties(&props.to_props()).to_state_id(waxed)
        })
}

/// Waxes the block at `location` if it has a waxed equivalent, emitting the wax
/// particles and sound on success.
pub(crate) fn try_wax_block(world: &Arc<World>, location: BlockPos, block: &Block) -> bool {
    let Some(replacement) = get_waxed_equivalent(block.id) else {
        return false;
    };
    let new_block = replacement.to_block();

    let new_state_id = waxed_state_id(block, world.get_block_state_id(&location), new_block);

    world.set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL);
    world.sync_world_event(WorldEvent::ParticlesWaxOn, location, 0);
    true
}

impl HoneyCombItem {
    pub fn apply_to_sign(
        &self,
        args: &UseWithItemArgs<'_>,
        block_entity: &Arc<dyn BlockEntity>,
        sign_entity: &crate::block::entities::sign::SignEntityRef<'_>,
    ) -> BlockActionResult {
        if sign_entity.is_waxed() {
            return BlockActionResult::PassToDefaultBlockAction;
        }
        sign_entity.set_waxed(true);

        args.world.update_block_entity(block_entity);
        args.world
            .sync_world_event(WorldEvent::ParticlesWaxOn, *args.position, 0);

        BlockActionResult::Success
    }
}

const fn get_waxed_equivalent(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::OXIDIZED_COPPER => Some(BlockId::WAXED_OXIDIZED_COPPER),
        BlockId::WEATHERED_COPPER => Some(BlockId::WAXED_WEATHERED_COPPER),
        BlockId::EXPOSED_COPPER => Some(BlockId::WAXED_EXPOSED_COPPER),
        BlockId::COPPER_BLOCK => Some(BlockId::WAXED_COPPER_BLOCK),
        BlockId::OXIDIZED_CHISELED_COPPER => Some(BlockId::WAXED_OXIDIZED_CHISELED_COPPER),
        BlockId::WEATHERED_CHISELED_COPPER => Some(BlockId::WAXED_WEATHERED_CHISELED_COPPER),
        BlockId::EXPOSED_CHISELED_COPPER => Some(BlockId::WAXED_EXPOSED_CHISELED_COPPER),
        BlockId::CHISELED_COPPER => Some(BlockId::WAXED_CHISELED_COPPER),
        BlockId::OXIDIZED_COPPER_GRATE => Some(BlockId::WAXED_OXIDIZED_COPPER_GRATE),
        BlockId::WEATHERED_COPPER_GRATE => Some(BlockId::WAXED_WEATHERED_COPPER_GRATE),
        BlockId::EXPOSED_COPPER_GRATE => Some(BlockId::WAXED_EXPOSED_COPPER_GRATE),
        BlockId::COPPER_GRATE => Some(BlockId::WAXED_COPPER_GRATE),
        BlockId::OXIDIZED_CUT_COPPER => Some(BlockId::WAXED_OXIDIZED_CUT_COPPER),
        BlockId::WEATHERED_CUT_COPPER => Some(BlockId::WAXED_WEATHERED_CUT_COPPER),
        BlockId::EXPOSED_CUT_COPPER => Some(BlockId::WAXED_EXPOSED_CUT_COPPER),
        BlockId::CUT_COPPER => Some(BlockId::WAXED_CUT_COPPER),
        BlockId::OXIDIZED_CUT_COPPER_STAIRS => Some(BlockId::WAXED_OXIDIZED_CUT_COPPER_STAIRS),
        BlockId::WEATHERED_CUT_COPPER_STAIRS => Some(BlockId::WAXED_WEATHERED_CUT_COPPER_STAIRS),
        BlockId::EXPOSED_CUT_COPPER_STAIRS => Some(BlockId::WAXED_EXPOSED_CUT_COPPER_STAIRS),
        BlockId::CUT_COPPER_STAIRS => Some(BlockId::WAXED_CUT_COPPER_STAIRS),
        BlockId::OXIDIZED_CUT_COPPER_SLAB => Some(BlockId::WAXED_OXIDIZED_CUT_COPPER_SLAB),
        BlockId::WEATHERED_CUT_COPPER_SLAB => Some(BlockId::WAXED_WEATHERED_CUT_COPPER_SLAB),
        BlockId::EXPOSED_CUT_COPPER_SLAB => Some(BlockId::WAXED_EXPOSED_CUT_COPPER_SLAB),
        BlockId::CUT_COPPER_SLAB => Some(BlockId::WAXED_CUT_COPPER_SLAB),
        BlockId::OXIDIZED_COPPER_BULB => Some(BlockId::WAXED_OXIDIZED_COPPER_BULB),
        BlockId::WEATHERED_COPPER_BULB => Some(BlockId::WAXED_WEATHERED_COPPER_BULB),
        BlockId::EXPOSED_COPPER_BULB => Some(BlockId::WAXED_EXPOSED_COPPER_BULB),
        BlockId::COPPER_BULB => Some(BlockId::WAXED_COPPER_BULB),
        BlockId::OXIDIZED_COPPER_DOOR => Some(BlockId::WAXED_OXIDIZED_COPPER_DOOR),
        BlockId::WEATHERED_COPPER_DOOR => Some(BlockId::WAXED_WEATHERED_COPPER_DOOR),
        BlockId::EXPOSED_COPPER_DOOR => Some(BlockId::WAXED_EXPOSED_COPPER_DOOR),
        BlockId::COPPER_DOOR => Some(BlockId::WAXED_COPPER_DOOR),
        BlockId::OXIDIZED_COPPER_TRAPDOOR => Some(BlockId::WAXED_OXIDIZED_COPPER_TRAPDOOR),
        BlockId::WEATHERED_COPPER_TRAPDOOR => Some(BlockId::WAXED_WEATHERED_COPPER_TRAPDOOR),
        BlockId::EXPOSED_COPPER_TRAPDOOR => Some(BlockId::WAXED_EXPOSED_COPPER_TRAPDOOR),
        BlockId::COPPER_TRAPDOOR => Some(BlockId::WAXED_COPPER_TRAPDOOR),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{get_waxed_equivalent, waxed_state_id};
    use pumpkin_data::BlockId;

    /// Waxing changes the copper variant and nothing else, so every property the unwaxed
    /// block carries has to survive the swap: a trapdoor's facing and open state, a stair's
    /// shape, a slab's half, a bulb's lit state, a grate's waterlogging.
    #[test]
    fn waxing_preserves_every_block_property() {
        let mut checked = 0;
        for raw in 0..BlockId::COUNT {
            let Some(id) = BlockId::new(raw) else {
                continue;
            };
            let Some(waxed) = get_waxed_equivalent(id) else {
                continue;
            };
            let block = id.to_block();
            let waxed_block = waxed.to_block();

            for state in block.states {
                let Some(props) = block.properties(state.id) else {
                    continue;
                };
                let before = props.to_props();
                let after_id = waxed_state_id(block, state.id, waxed_block);
                let after = waxed_block
                    .properties(after_id)
                    .expect("the waxed block has the same properties")
                    .to_props();
                assert_eq!(before, after, "{} -> {}", block.name, waxed_block.name);
                checked += 1;
            }
        }
        assert!(checked > 0, "no waxable block states were checked");
    }
}
