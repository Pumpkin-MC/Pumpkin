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
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct FireworkRocketItem;

const fn should_consume_rocket(is_creative: bool) -> bool {
    !is_creative
}

impl ItemMetadata for FireworkRocketItem {
    fn ids() -> Box<[u16]> {
        [Item::FIREWORK_ROCKET.id].into()
    }
}

impl ItemBehaviour for FireworkRocketItem {
    fn use_on_block<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        cursor_pos: Vector3<f32>,
        _block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let entity = Entity::new(
                world.clone(),
                Vector3::new(
                    f64::from(location.0.x) + f64::from(cursor_pos.x),
                    f64::from(location.0.y) + f64::from(cursor_pos.y),
                    f64::from(location.0.z) + f64::from(cursor_pos.z),
                ),
                &EntityType::FIREWORK_ROCKET,
            );
            let entity = FireworkRocketEntity::new(entity);
            world.spawn_entity(Arc::new(entity)).await;
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
                let entity = FireworkRocketEntity::new_shot(entity, player.get_entity());
                world.spawn_entity(Arc::new(entity)).await;

                // Vanilla `FireworkRocketItem::use` consumes the hand that launched
                // the attached rocket, except in Creative mode.
                if should_consume_rocket(player.is_creative()) {
                    let held_item = player.inventory.held_item();
                    let mut held_item = held_item.lock().await;
                    if held_item.item == &Item::FIREWORK_ROCKET {
                        held_item.decrement(1);
                    } else {
                        drop(held_item);
                        let off_hand_item = player.inventory.off_hand_item().await;
                        let mut off_hand_item = off_hand_item.lock().await;
                        if off_hand_item.item == &Item::FIREWORK_ROCKET {
                            off_hand_item.decrement(1);
                        }
                    }
                }
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::should_consume_rocket;

    #[test]
    fn rockets_are_consumed_outside_creative() {
        assert!(should_consume_rocket(false));
        assert!(!should_consume_rocket(true));
    }
}
