use std::sync::Arc;

use crate::block::blocks::redstone::rails::rails_tag_shape;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::entity::vehicle::minecart::{AbstractMinecart, MinecartType, RAIL_HEIGHT, is_minecart};
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::game_event::GameEventContext;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

/// Vanilla `MinecartItem`.
pub struct MinecartItem;

/// Extra spawn height on a slope.
const SLOPE_HEIGHT: f64 = 0.5;

impl ItemMetadata for MinecartItem {
    fn ids() -> Box<[u16]> {
        MinecartType::ALL
            .into_iter()
            .filter_map(MinecartType::item)
            .map(|item| item.id)
            .collect()
    }
}

impl ItemBehaviour for MinecartItem {
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
        let Some(shape) = rails_tag_shape(block, world.get_block_state_id(&location)) else {
            return BlockActionResult::Fail;
        };
        let Some(kind) = MinecartType::from_item(item.item) else {
            return BlockActionResult::Fail;
        };
        let offset = if shape.is_ascending() {
            SLOPE_HEIGHT
        } else {
            0.0
        };
        // Vanilla `pos.getX() + 0.5`: the rail block's corner plus half a block.
        let corner = location.0.to_f64();
        let minecart = Arc::new(AbstractMinecart::create(
            world.clone(),
            Vector3::new(
                corner.x + 0.5,
                corner.y + RAIL_HEIGHT + offset,
                corner.z + 0.5,
            ),
            kind,
            item,
            Some(player),
        ));
        // Experimental carts don't stack on one spot.
        if minecart.uses_experimental_movement()
            && !world
                .get_entities_of_type_at_box(
                    &minecart.vehicle.entity.bounding_box.load(),
                    is_minecart,
                )
                .is_empty()
        {
            return BlockActionResult::Fail;
        }
        world.spawn_entity(minecart);
        // TODO: vanilla's context also carries the block state below the rail.
        world.emit_game_event_with_context(
            GameEvent::EntityPlace.name(),
            location.to_centered_f64(),
            &GameEventContext {
                source_entity: Some(player),
            },
        );
        item.decrement_unless_creative(player.gamemode.load(), 1);
        BlockActionResult::Success
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
