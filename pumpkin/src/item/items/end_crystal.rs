use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::decoration::end_crystal::EndCrystalEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use uuid::Uuid;

pub struct EndCrystalItem;

impl ItemMetadata for EndCrystalItem {
    fn ids() -> Box<[u16]> {
        [Item::END_CRYSTAL.id].into()
    }
}

#[async_trait]
impl ItemBehaviour for EndCrystalItem {
    async fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        _block: &Block,
        _server: &Server,
    ) {
        let world = player.world();
        let block = world.get_block(&location).await;
        if block != &Block::OBSIDIAN && block != &Block::BEDROCK {
            return;
        }

        let location = location.up();
        let entity = Entity::new(
            Uuid::new_v4(),
            world.clone(),
            location.to_f64(),
            &EntityType::END_CRYSTAL,
            false,
        );
        let end_crystal = Arc::new(EndCrystalEntity::new(entity));
        world.spawn_entity(end_crystal.clone()).await;
        end_crystal.set_show_bottom(false).await;
        item.decrement_unless_creative(player.gamemode.load(), 1);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
