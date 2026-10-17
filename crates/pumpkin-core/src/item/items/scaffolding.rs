use std::any::Any;

use pumpkin_data::item::Item;
use pumpkin_data::{Block, BlockDirection, HorizontalFacingExt};

use crate::block::BlockPlaceContext;
use crate::block::blocks::scaffolding::{STABILITY_MAX_DISTANCE, ScaffoldingBlock};
use crate::block::registry::can_replace_with_other_block;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::world::World;

pub struct ScaffoldingBlockItem;

impl ItemMetadata for ScaffoldingBlockItem {
    fn ids() -> Box<[u16]> {
        [Item::SCAFFOLDING.id].into()
    }
}

impl ItemBehaviour for ScaffoldingBlockItem {
    fn update_placement_context(
        &self,
        world: &World,
        player: &Player,
        context: BlockPlaceContext,
    ) -> Option<BlockPlaceContext> {
        let pos = context.position;
        if world.get_block(&pos) != &Block::SCAFFOLDING {
            return (ScaffoldingBlock::get_distance(world, &pos) != STABILITY_MAX_DISTANCE)
                .then_some(context);
        }

        let entity = player.get_entity();
        let direction = if entity.is_sneaking() {
            if context.inside {
                context.clicked_face.opposite()
            } else {
                context.clicked_face
            }
        } else if context.clicked_face == BlockDirection::Up {
            entity.get_horizontal_facing().to_block_direction()
        } else {
            BlockDirection::Up
        };

        let mut horizontal_distance = 0;
        let mut placement_pos = pos.offset(direction.to_offset());
        while horizontal_distance < STABILITY_MAX_DISTANCE {
            if !world.is_in_build_limit(placement_pos) {
                let max_y = world.get_top_y();
                if placement_pos.0.y > max_y {
                    player.send_build_too_high_message(max_y);
                }
                break;
            }

            let (block, state) = world.get_block_and_state(&placement_pos);
            if block != &Block::SCAFFOLDING {
                if can_replace_with_other_block(block, state) {
                    return Some(BlockPlaceContext::at(world, placement_pos, direction));
                }
                break;
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
