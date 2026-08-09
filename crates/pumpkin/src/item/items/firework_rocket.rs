use std::pin::Pin;
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
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct FireworkRocketItem;

impl FireworkRocketItem {
    const PLACEMENT_OFFSET: f64 = 0.15;

    fn placement_position(
        location: BlockPos,
        cursor_pos: Vector3<f32>,
        face: BlockDirection,
    ) -> Vector3<f64> {
        let face = face.to_offset();
        Vector3::new(
            f64::from(location.0.x)
                + f64::from(cursor_pos.x)
                + f64::from(face.x) * Self::PLACEMENT_OFFSET,
            f64::from(location.0.y)
                + f64::from(cursor_pos.y)
                + f64::from(face.y) * Self::PLACEMENT_OFFSET,
            f64::from(location.0.z)
                + f64::from(cursor_pos.z)
                + f64::from(face.z) * Self::PLACEMENT_OFFSET,
        )
    }

    async fn consume_held_rocket(player: &Player) {
        if player.gamemode.load() == pumpkin_util::GameMode::Creative {
            return;
        }

        let mut main_hand = player.inventory.held_item().await;
        let consumed_stack =
            (!main_hand.is_empty() && main_hand.item.id == Item::FIREWORK_ROCKET.id).then(|| {
                main_hand.decrement_unless_creative(player.gamemode.load(), 1);
                main_hand
            });

        if let Some(stack) = consumed_stack {
            player.inventory.set_held_item(stack.clone()).await;
            player
                .sync_hand_slot(player.inventory.get_selected_slot() as usize, stack)
                .await;
            return;
        }

        let mut updated_stack = player.inventory.off_hand_item().await;
        if updated_stack.is_empty() || updated_stack.item.id != Item::FIREWORK_ROCKET.id {
            return;
        }
        updated_stack.decrement_unless_creative(player.gamemode.load(), 1);
        player
            .inventory
            .set_stack_in_hand(pumpkin_util::Hand::Left, updated_stack.clone())
            .await;
        player
            .sync_hand_slot(PlayerInventory::OFF_HAND_SLOT, updated_stack)
            .await;
    }
}

impl ItemMetadata for FireworkRocketItem {
    fn ids() -> Box<[u16]> {
        [Item::FIREWORK_ROCKET.id].into()
    }
}

impl ItemBehaviour for FireworkRocketItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        face: BlockDirection,
        cursor_pos: Vector3<f32>,
        _block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let entity = Entity::new(
                world.clone(),
                Self::placement_position(location, cursor_pos, face),
                &EntityType::FIREWORK_ROCKET,
            );
            let entity = FireworkRocketEntity::new(entity, item.clone());
            world.spawn_entity(Arc::new(entity)).await;
            item.decrement_unless_creative(player.gamemode.load(), 1);
        })
    }

    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            if player.get_entity().is_fall_flying() {
                let world = player.world();
                let entity = Entity::new(
                    world.clone(),
                    player.get_entity().pos.load(),
                    &EntityType::FIREWORK_ROCKET,
                );
                let item_stack = player.inventory.held_item().await;
                let entity =
                    FireworkRocketEntity::new_shot(entity, player.get_entity(), item_stack);
                world.spawn_entity(Arc::new(entity)).await;
                Self::consume_held_rocket(player).await;
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placed_rocket_is_offset_outside_clicked_face() {
        let position = FireworkRocketItem::placement_position(
            BlockPos(Vector3::new(10, 20, 30)),
            Vector3::new(0.25, 1.0, 0.75),
            BlockDirection::Up,
        );

        assert_eq!(position, Vector3::new(10.25, 21.15, 30.75));
    }
}
