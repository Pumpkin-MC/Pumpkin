use std::any::Any;

use pumpkin_data::item::Item;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;

use crate::block::blocks::scaffolding::{STABILITY_MAX_DISTANCE, ScaffoldingBlock};
use crate::block::registry::can_replace_with_other_block;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};

pub struct ScaffoldingItem;

impl ItemMetadata for ScaffoldingItem {
    fn ids() -> Box<[u16]> {
        [Item::SCAFFOLDING.id].into()
    }
}

impl ItemBehaviour for ScaffoldingItem {
    fn update_placement_context(
        &self,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        inside_block: bool,
    ) -> Option<(BlockPos, BlockDirection)> {
        let world = player.world();
        // Vanilla `BlockPlaceContext.getClickedPos`; scaffolding can be replaced by scaffolding.
        let (block, state) = world.get_block_and_state(&location);
        let clicked_pos =
            if block == &Block::SCAFFOLDING || can_replace_with_other_block(block, state) {
                location
            } else {
                location.offset(face.to_offset())
            };
        if world.get_block(&clicked_pos) != &Block::SCAFFOLDING {
            let distance = ScaffoldingBlock::get_distance(&*world, &clicked_pos);
            return (distance != STABILITY_MAX_DISTANCE).then_some((location, face));
        }

        let entity = player.get_entity();
        let direction = if entity.is_sneaking() {
            if inside_block { face.opposite() } else { face }
        } else if face == BlockDirection::Up {
            BlockDirection::from_cardinal_direction(entity.get_horizontal_facing())
        } else {
            BlockDirection::Up
        };

        let mut horizontal_distance = 0;
        let mut placement_pos = clicked_pos.offset(direction.to_offset());
        while horizontal_distance < STABILITY_MAX_DISTANCE {
            if !world.is_in_build_limit(placement_pos) {
                let max_y = world.get_top_y();
                if placement_pos.0.y > max_y {
                    player.send_build_too_high_message(max_y);
                }
                return None;
            }

            let (block, state) = world.get_block_and_state(&placement_pos);
            if block != &Block::SCAFFOLDING {
                // `BlockPlaceContext.at(context, placementPos, direction)`
                return can_replace_with_other_block(block, state)
                    .then_some((placement_pos, direction));
            }

            placement_pos = placement_pos.offset(direction.to_offset());
            if direction.is_horizontal() {
                horizontal_distance += 1;
            }
        }

        None
    }

    fn must_survive(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
