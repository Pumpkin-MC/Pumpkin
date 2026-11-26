use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::entity::decoration::leash_knot::LeashKnotEntity;
use crate::entity::player::Player;
use crate::entity::{can_have_a_leash_attached_to, leashable_in_area};
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct LeadItem;

impl LeadItem {
    pub fn bind_player_mobs(
        player: &Player,
        world: &Arc<World>,
        pos: BlockPos,
    ) -> BlockActionResult {
        let center = pos.to_centered_f64();
        let entities_to_leash = leashable_in_area(world, center, player.entity_id());
        if entities_to_leash.is_empty() {
            return BlockActionResult::Pass;
        }

        let existing_knot = LeashKnotEntity::get_knot(world, pos);
        let active_knot = existing_knot
            .clone()
            .unwrap_or_else(|| LeashKnotEntity::create_knot(world, pos));
        let mut any_leashed = false;
        for leashable in entities_to_leash {
            if can_have_a_leash_attached_to(leashable.as_ref(), active_knot.as_ref())
                && leashable
                    .get_entity()
                    .leash_to_if_held_by(player.entity_id(), &active_knot)
            {
                any_leashed = true;
            }
        }

        if any_leashed {
            LeashKnotEntity::play_placement_sound(active_knot.get_entity());
            world.emit_game_event(GameEvent::BlockAttach.name(), center);
            return BlockActionResult::SuccessServer;
        }

        if existing_knot.is_none() {
            active_knot.get_entity().remove();
        }
        BlockActionResult::Pass
    }
}

impl ItemMetadata for LeadItem {
    fn ids() -> Box<[u16]> {
        [Item::LEAD.id].into()
    }
}

impl ItemBehaviour for LeadItem {
    fn use_on_block(
        &self,
        _item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        if !block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_FENCES) {
            return BlockActionResult::Pass;
        }
        Self::bind_player_mobs(player, &player.world(), location)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
