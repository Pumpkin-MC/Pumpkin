use std::sync::Arc;

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
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct FireworkRocketItem;

impl ItemMetadata for FireworkRocketItem {
    fn ids() -> Box<[u16]> {
        [Item::FIREWORK_ROCKET.id].into()
    }
}

impl ItemBehaviour for FireworkRocketItem {
    // A firework rocket always launches, whether or not the player is gliding - it's a
    // functional item in its own right (decorative in the open, a boost while gliding), not
    // an elytra-only action. The boost itself is gated separately, inside the spawned
    // entity's own tick logic (`FireworkRocketEntity::tick`), based on the owner's
    // fall-flying state at the time - so spawning unconditionally here is what lets both
    // cases (plain launch, and launch-while-gliding boost) share one code path.
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        _location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) {
        let world = player.world();
        let entity = Entity::new(
            world.clone(),
            player.get_entity().pos.load(),
            &EntityType::FIREWORK_ROCKET,
        );
        let entity =
            FireworkRocketEntity::new_shot_with_item(entity, player.get_entity(), item.clone());
        world.spawn_entity(Arc::new(entity));
        item.decrement_unless_creative(player.gamemode.load(), 1);
    }

    fn normal_use(&self, _item: &Item, player: &Player) {
        let mut held = player.inventory().held_item();
        let mut is_main = true;
        if held.is_empty() || held.item.id != Item::FIREWORK_ROCKET.id {
            held = player.inventory().off_hand_item();
            is_main = false;
            if held.is_empty() || held.item.id != Item::FIREWORK_ROCKET.id {
                return;
            }
        }

        let world = player.world();
        let entity = Entity::new(
            world.clone(),
            player.get_entity().pos.load(),
            &EntityType::FIREWORK_ROCKET,
        );
        let entity =
            FireworkRocketEntity::new_shot_with_item(entity, player.get_entity(), held.clone());
        world.spawn_entity(Arc::new(entity));

        held.decrement_unless_creative(player.gamemode.load(), 1);
        if is_main {
            player.inventory().set_held_item(held);
        } else {
            player
                .inventory()
                .set_stack_in_hand(pumpkin_util::Hand::Left, held);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
