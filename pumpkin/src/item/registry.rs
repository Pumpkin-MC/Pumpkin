use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rustc_hash::FxHashMap;
use std::sync::Arc;

use super::{ItemBehaviour, ItemMetadata};

#[derive(Default)]
pub struct ItemRegistry {
    items: FxHashMap<u16, Arc<dyn ItemBehaviour>>,
}

impl ItemRegistry {
    pub fn register<T: ItemBehaviour + ItemMetadata + 'static>(&mut self, item: T) {
        let val = Arc::new(item);
        self.items.reserve(T::ids().len());
        for i in T::ids() {
            self.items.insert(i, val.clone());
        }
    }

    pub async fn on_use(&self, stack: &ItemStack, player: &Player) {
        let item = stack.item;
        let cooldown = stack.get_use_cooldown();
        let cooldown_group = cooldown
            .and_then(|c| c.cooldown_group.clone())
            .unwrap_or_else(|| item.registry_key.to_string());

        if player.is_on_cooldown(&cooldown_group).await {
            return;
        }

        let pumpkin_item = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item.normal_use(item, player).await;
        }

        if let Some(cooldown) = cooldown {
            player
                .start_cooldown(cooldown_group, (cooldown.seconds * 20.0) as i32)
                .await;
        }
    }

    pub async fn on_stopped_using(&self, stack: &ItemStack, player: &Player) {
        if let Some(behaviour) = self.get_pumpkin_item(stack.item.id) {
            behaviour.on_stopped_using(stack, player).await;
        }
    }

    /// Returns the item's use duration in ticks, as defined by its registered behaviour.
    /// Returns `None` if the item has no registered behaviour or its duration is 0.
    #[must_use]
    pub fn get_use_duration(&self, item_id: u16) -> Option<i32> {
        self.get_pumpkin_item(item_id)
            .map(|b| b.get_use_duration())
            .filter(|&d| d > 0)
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
        let cooldown = stack.get_use_cooldown().cloned();
        let cooldown_group = cooldown
            .as_ref()
            .and_then(|c| c.cooldown_group.clone())
            .unwrap_or_else(|| stack.item.registry_key.to_string());

        if player.is_on_cooldown(&cooldown_group).await {
            return;
        }

        let pumpkin_item = self.get_pumpkin_item(stack.item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item
                .use_on_block(stack, player, location, face, cursor_pos, block, server)
                .await;
        }

        if let Some(cooldown) = cooldown {
            player
                .start_cooldown(cooldown_group, (cooldown.seconds * 20.0) as i32)
                .await;
        }
    }

    pub async fn use_on_entity(
        &self,
        stack: &mut ItemStack,
        player: &Player,
        entity: Arc<dyn EntityBase>,
    ) {
        let cooldown = stack.get_use_cooldown().cloned();
        let cooldown_group = cooldown
            .as_ref()
            .and_then(|c| c.cooldown_group.clone())
            .unwrap_or_else(|| stack.item.registry_key.to_string());

        if player.is_on_cooldown(&cooldown_group).await {
            return;
        }

        let pumpkin_item = self.get_pumpkin_item(stack.item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item.use_on_entity(stack, player, entity).await;
        }

        if let Some(cooldown) = cooldown {
            player
                .start_cooldown(cooldown_group, (cooldown.seconds * 20.0) as i32)
                .await;
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
