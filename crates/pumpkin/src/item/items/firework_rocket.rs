use crate::block::registry::BlockActionResult;

use crate::entity::player::Player;
use crate::entity::projectile::firework_rocket::FireworkRocketEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

const ROCKET_PLACEMENT_OFFSET: f64 = 0.15;

pub struct FireworkRocketItem;

impl ItemMetadata for FireworkRocketItem {
    fn ids() -> Box<[u16]> {
        [Item::FIREWORK_ROCKET.id].into()
    }
}

impl ItemBehaviour for FireworkRocketItem {
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        if player.get_entity().is_fall_flying() {
            return BlockActionResult::Pass;
        }

        let world = player.world();
        let offset = face.to_offset();
        let entity = Entity::new(
            world.clone(),
            Vector3::new(
                f64::from(location.0.x)
                    + f64::from(cursor_pos.x)
                    + f64::from(offset.x) * ROCKET_PLACEMENT_OFFSET,
                f64::from(location.0.y)
                    + f64::from(cursor_pos.y)
                    + f64::from(offset.y) * ROCKET_PLACEMENT_OFFSET,
                f64::from(location.0.z)
                    + f64::from(cursor_pos.z)
                    + f64::from(offset.z) * ROCKET_PLACEMENT_OFFSET,
            ),
            &EntityType::FIREWORK_ROCKET,
        );
        let rocket = FireworkRocketEntity::new(entity, Some(player.get_entity()), item.clone());
        rocket.spawn(&world);
        item.decrement_unless_creative(player.gamemode.load(), 1);
        BlockActionResult::Success
    }

    fn normal_use_in_hand(
        &self,
        _item: &Item,
        player: &Player,
        hand: Hand,
        _yaw: f32,
        _pitch: f32,
    ) {
        if !player.get_entity().is_fall_flying() {
            return;
        }

        let inventory = player.inventory();
        let mut held = inventory.get_stack_in_hand(hand);
        if held.is_empty() || held.item.id != Item::FIREWORK_ROCKET.id {
            return;
        }

        let world = player.world();
        let entity = Entity::new(
            world.clone(),
            player.get_entity().pos.load(),
            &EntityType::FIREWORK_ROCKET,
        );
        if let Some(player) = world.get_player_by_id(player.get_entity().entity_id)
            && let Some(server) = world.server.upgrade()
        {
            let mut event =
                crate::plugin::api::events::player::player_elytra_boost::PlayerElytraBoostEvent {
                    player,
                    firework_id: entity.entity_id,
                    cancelled: false,
                };
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return;
            }
        }
        if player.get_entity().drop_all_leash_connections() {
            world.play_sound_fine(
                Sound::ItemLeadBreak,
                SoundCategory::Neutral,
                &player.get_entity().pos.load(),
                1.0,
                1.0,
            );
        }
        let rocket = FireworkRocketEntity::new_attached(entity, held.clone(), player.get_entity());
        rocket.spawn(&world);

        held.decrement_unless_creative(player.gamemode.load(), 1);
        inventory.set_stack_in_hand(hand, held);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
