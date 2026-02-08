use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::player::Player;
use crate::entity::projectile::firework_rocket::FireworkRocketEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;

pub struct FireworkRocketItem;

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
            let entity = FireworkRocketEntity::new(entity).await;
            world.spawn_entity(Arc::new(entity)).await;
        })
    }

    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
        _hand: Hand,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            if player.get_entity().fall_flying.load(Ordering::Relaxed) {
                let world = player.world();
                let entity = Entity::new(
                    world.clone(),
                    player.get_entity().pos.load(),
                    &EntityType::FIREWORK_ROCKET,
                );
                let entity = FireworkRocketEntity::new_shot(entity, player.get_entity()).await;
                world.spawn_entity(Arc::new(entity)).await;
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
