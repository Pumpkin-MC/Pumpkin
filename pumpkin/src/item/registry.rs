use crate::entity::Entity;
use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_data::item::Item;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::{block::Block, sound::Sound};
use pumpkin_inventory::player::PlayerInventory;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use std::collections::HashMap;
use std::sync::Arc;

use super::pumpkin_item::{ItemMetadata, PumpkinItem};

#[derive(Default)]
pub struct ItemRegistry {
    items: HashMap<Box<[u16]>, Arc<dyn PumpkinItem>>,
}

impl ItemRegistry {
    pub fn register<T: PumpkinItem + ItemMetadata + 'static>(&mut self, item: T) {
        self.items.insert(T::ids(), Arc::new(item));
    }

    pub async fn on_use(&self, item: &Item, player: &Player) {
        let pumpkin_block = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block.normal_use(item, player).await;

            let mut inventory = player.inventory.lock().await;

            self.handle_possible_broken_item(&mut inventory, player)
                .await;
        }
    }

    pub async fn on_attack(&self, entity: &Entity, item: &Item, player: &Player) {
        let pumpkin_block = self.get_pumpkin_item(item.id);

        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block.on_attack(item, entity).await;

            let mut inventory = player.inventory.lock().await;

            self.handle_possible_broken_item(&mut inventory, player)
                .await;
        }
    }

    // TODO: we probably need a method to take care of items that can break when they're
    // not in the main/secondary hand (e.g Elytra)
    //
    // Note: this method assume that get_selected_slot will the item used, which might
    // be in the main or secondary hand
    async fn handle_possible_broken_item(&self, inventory: &mut PlayerInventory, player: &Player) {
        let slot_id = inventory.get_selected_slot();

        if let Some(item_stack) = inventory.held_item().cloned() {
            if !item_stack.is_broken() {
                player
                    .update_single_slot(inventory, slot_id, item_stack)
                    .await;

                return;
            }

            inventory.decrease_current_stack(1);

            player.empty_slot(inventory, slot_id).await;

            let item: Item = item_stack.item;

            if let Some(sound_name) = item.components.break_sound {
                if let Some(sound) = Sound::from_name(sound_name) {
                    player
                        .world()
                        .await
                        .play_sound(sound, SoundCategory::Players, &player.position())
                        .await;
                }
            }
        }
    }

    pub async fn use_on_block(
        &self,
        item: &Item,
        player: &Player,
        location: BlockPos,
        face: &BlockDirection,
        block: &Block,
        server: &Server,
    ) {
        let pumpkin_item = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item
                .use_on_block(item, player, location, face, block, server)
                .await;

            let mut inventory = player.inventory.lock().await;

            self.handle_possible_broken_item(&mut inventory, player)
                .await;
        }
    }

    pub async fn on_after_dig(
        &self,
        item: &Item,
        player: &Player,
        location: BlockPos,
        block: &Block,
        server: &Server,
    ) {
        let pumpkin_item = self.get_pumpkin_item(item.id);
        if let Some(pumpkin_item) = pumpkin_item {
            pumpkin_item
                .on_after_dig(item, player, location, block, server)
                .await;

            let mut inventory = player.inventory.lock().await;

            self.handle_possible_broken_item(&mut inventory, player)
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
    pub fn get_pumpkin_item(&self, item_id: u16) -> Option<&Arc<dyn PumpkinItem>> {
        //TODO: We really want to use a lookup table for this
        self.items
            .iter()
            .find_map(|(ids, item)| ids.contains(&item_id).then_some(item))
    }
}
