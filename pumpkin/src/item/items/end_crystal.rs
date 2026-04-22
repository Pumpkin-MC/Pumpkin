use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::decoration::end_crystal::EndCrystalEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct EndCrystalItem;

impl ItemMetadata for EndCrystalItem {
    fn ids() -> Box<[u16]> {
        [Item::END_CRYSTAL.id].into()
    }
}

impl ItemBehaviour for EndCrystalItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let block = world.get_block(&location).await;
            if block != &Block::OBSIDIAN && block != &Block::BEDROCK {
                return;
            }

            let location = location.up();
            let location_vec = location.0.to_f64();

            if !world.get_block_state(&location).await.is_air()
                || !world
                    .get_entities_at_box(&BoundingBox::new(
                        Vector3::new(location_vec.x, location_vec.y, location_vec.z),
                        Vector3::new(location_vec.x + 1.0, location_vec.y + 2.0, location_vec.z),
                    ))
                    .is_empty()
            {
                return;
            }

            let entity = Entity::new(world.clone(), location.to_f64(), &EntityType::END_CRYSTAL);
            let end_crystal = Arc::new(EndCrystalEntity::new(entity));
            world.spawn_entity(end_crystal.clone()).await;
            end_crystal.set_show_bottom(false).await;
            item.decrement_unless_creative(player.gamemode.load(), 1);
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
