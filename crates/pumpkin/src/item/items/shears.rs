use std::pin::Pin;
use std::sync::Arc;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::game_event::{GameEventContext, emit_game_event};
use pumpkin_data::block_properties::{
    BlockProperties, CaveVinesLikeProperties, KelpLikeProperties,
};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct ShearsItem;

impl ItemMetadata for ShearsItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::SHEARS.id])
    }
}

/// vanilla `GrowingPlantHeadBlock.MAX_AGE`.
const MAX_AGE: u8 = 25;

/// Mirrors `GrowingPlantHeadBlock.isMaxAge` + `getMaxAgeState` for the four head blocks
/// shears interact with (kelp, twisting/weeping vines, cave vines). Returns `None` if
/// `block` isn't one of these, or if it's already at max age (vanilla falls through to
/// `super.useOn`, i.e. does nothing, in both cases).
fn max_age_state(block: &Block, state_id: BlockStateId) -> Option<BlockStateId> {
    if block == &Block::KELP || block == &Block::TWISTING_VINES || block == &Block::WEEPING_VINES {
        let props = KelpLikeProperties::from_state_id(state_id, block);
        if props.age >= MAX_AGE {
            return None;
        }
        let new_props = KelpLikeProperties { age: MAX_AGE };
        Some(new_props.to_state_id(block))
    } else if block == &Block::CAVE_VINES {
        let props = CaveVinesLikeProperties::from_state_id(state_id, block);
        if props.age >= MAX_AGE {
            return None;
        }
        let new_props = CaveVinesLikeProperties {
            age: MAX_AGE,
            berries: props.berries,
        };
        Some(new_props.to_state_id(block))
    } else {
        None
    }
}

impl ItemBehaviour for ShearsItem {
    // Mob-shearing (sheep/mooshroom/snow golem/bogged) and beehive/nest shearing are out
    // of scope here (tracked separately) - only the block-carving half of ShearsItem is
    // implemented. Pumpkin carving lives in `PumpkinBlock::use_with_item`
    // (block/blocks/pumpkin.rs) since vanilla dispatches it from
    // `PumpkinBlock#useItemOn`, which the client/server call before the item's own
    // `useOn` - Pumpkin's `use_with_item`/`use_on_block` dispatch mirrors that order.
    //
    // `ShearsItem#mineBlock` (durability loss on block break, unless the block is fire)
    // and the cobweb/leaves/vine efficient-tool mining-speed bonus are both already
    // data-driven generically for every tool via the `Tool` component
    // (`Player::apply_tool_damage_for_block_break`, `ItemStack::get_speed`) - no
    // shears-specific code is needed for either.
    fn use_on_block<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let state_id = world.get_block_state_id(&location);
            let Some(new_state_id) = max_age_state(block, state_id) else {
                return;
            };

            world
                .set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL)
                .await;

            world.play_block_sound_expect(
                player,
                Sound::BlockGrowingPlantCrop,
                SoundCategory::Blocks,
                location,
            );

            if let Some(player_arc) = world.get_player_by_id(player.get_entity().entity_id) {
                emit_game_event(
                    &world,
                    GameEvent::BlockChange,
                    location.to_f64(),
                    GameEventContext::of_entity(player_arc as Arc<dyn crate::entity::EntityBase>),
                )
                .await;
            }

            player.damage_held_item(1).await;
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::max_age_state;
    use pumpkin_data::Block;
    use pumpkin_data::block_properties::{
        BlockProperties, CaveVinesLikeProperties, KelpLikeProperties,
    };

    #[test]
    fn kelp_not_at_max_age_carves_to_max_age() {
        let props = KelpLikeProperties { age: 3 };
        let state_id = props.to_state_id(&Block::KELP);
        let new_state_id =
            max_age_state(&Block::KELP, state_id).expect("kelp below max age should carve");
        let new_props = KelpLikeProperties::from_state_id(new_state_id, &Block::KELP);
        assert_eq!(new_props.age, 25);
    }

    #[test]
    fn kelp_already_at_max_age_does_nothing() {
        let props = KelpLikeProperties { age: 25 };
        let state_id = props.to_state_id(&Block::KELP);
        assert!(max_age_state(&Block::KELP, state_id).is_none());
    }

    #[test]
    fn twisting_and_weeping_vines_carve_to_max_age() {
        for block in [&Block::TWISTING_VINES, &Block::WEEPING_VINES] {
            let props = KelpLikeProperties { age: 0 };
            let state_id = props.to_state_id(block);
            let new_state_id =
                max_age_state(block, state_id).expect("vines below max age should carve");
            let new_props = KelpLikeProperties::from_state_id(new_state_id, block);
            assert_eq!(new_props.age, 25);
        }
    }

    #[test]
    fn cave_vines_carve_to_max_age_preserving_berries() {
        for berries in [false, true] {
            let props = CaveVinesLikeProperties { age: 10, berries };
            let state_id = props.to_state_id(&Block::CAVE_VINES);
            let new_state_id = max_age_state(&Block::CAVE_VINES, state_id)
                .expect("cave vines below max age should carve");
            let new_props =
                CaveVinesLikeProperties::from_state_id(new_state_id, &Block::CAVE_VINES);
            assert_eq!(new_props.age, 25);
            assert_eq!(new_props.berries, berries);
        }
    }

    #[test]
    fn cave_vines_already_at_max_age_does_nothing() {
        let props = CaveVinesLikeProperties {
            age: 25,
            berries: true,
        };
        let state_id = props.to_state_id(&Block::CAVE_VINES);
        assert!(max_age_state(&Block::CAVE_VINES, state_id).is_none());
    }

    #[test]
    fn unrelated_block_returns_none() {
        assert!(max_age_state(&Block::STONE, Block::STONE.default_state.id).is_none());
    }
}
