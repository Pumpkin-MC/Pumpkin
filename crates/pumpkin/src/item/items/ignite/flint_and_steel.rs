use crate::entity::player::Player;
use crate::item::ItemBehaviour;
use crate::item::ItemMetadata;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use crate::item::items::ignite::ignition::Ignition;

use crate::plugin::api::events::world::portal_create::{PortalCreateEvent, PortalType};

pub struct FlintAndSteelItem;

impl ItemMetadata for FlintAndSteelItem {
    fn ids() -> Box<[u16]> {
        [Item::FLINT_AND_STEEL.id].into()
    }
}

impl ItemBehaviour for FlintAndSteelItem {
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &Block,
        _server: &Server,
    ) {
        let world = player.world();
        let server_ref = world.server.upgrade();
        if let Some(server_ref) = server_ref {
            let player_arc = world.get_player_by_uuid(player.gameprofile.id);
            let mut event = crate::plugin::api::events::block::block_ignite::BlockIgniteEvent::new(
                location,
                &pumpkin_data::Block::FIRE,
                player_arc,
            );
            server_ref
                .plugin_manager
                .fire_blocking(&server_ref, &mut event);
            let mut portal_event = PortalCreateEvent::new(location, PortalType::Nether);
            server_ref
                .plugin_manager
                .fire_blocking(&server_ref, &mut portal_event);
            if event.cancelled || portal_event.cancelled {
                return;
            }
        }

        let ignited = Ignition::ignite_block(
            |world: Arc<World>, pos: BlockPos, new_state_id: BlockStateId| {
                world.set_block_state(&pos, new_state_id, BlockFlags::NOTIFY_ALL);
            },
            &world,
            location,
            location.offset(face.to_offset()),
            block,
        );

        if ignited && player.gamemode.load() != pumpkin_util::GameMode::Creative {
            // TODO: Handle DamageResult::Broken to broadcast item break and update player slot.
            let _ = item.damage_item(1);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
