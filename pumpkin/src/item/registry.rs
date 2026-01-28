use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use std::collections::HashMap;
use std::sync::Arc;

use super::{ItemBehaviour, ItemMetadata};

#[derive(Default)]
pub struct ItemRegistry {
    items: HashMap<u16, Arc<dyn ItemBehaviour>>,
}

impl ItemRegistry {
    pub fn register<T: ItemBehaviour + ItemMetadata + 'static>(&mut self, item: T) {
        let val = Arc::new(item);
        self.items.reserve(T::ids().len());
        for i in T::ids() {
            self.items.insert(i, val.clone());
        }
    }

    pub async fn on_use(&self, item: &Item, player: &Player) {
        let pumpkin_item = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item.normal_use(item, player).await;
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub async fn use_on_block(
        &self,
        stack: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        cursor_pos: Vector3<f32>,
        block: &Block,
        server: &Server,
    ) {
        let pumpkin_item = self.get_pumpkin_item(stack.item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item
                .use_on_block(stack, player, location, face, cursor_pos, block, server)
                .await;
        }
    }

    pub async fn use_on_entity(
        &self,
        stack: &mut ItemStack,
        player: &Player,
        entity: Arc<dyn EntityBase>,
    ) {
        let pumpkin_item = self.get_pumpkin_item(stack.item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item.use_on_entity(stack, player, entity).await;
        }
    }

    pub fn can_mine(&self, item: &Item, player: &Player) -> bool {
        let pumpkin_block = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block.can_mine(player);
        }
        true
    }

    #[must_use]
    pub fn get_pumpkin_item(&self, item: u16) -> Option<&Arc<dyn ItemBehaviour>> {
        self.items.get(&item)
    }
}
