use std::pin::Pin;
use std::sync::Arc;

use crate::entity::{Entity, item::ItemEntity, player::Player};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::BundleContentsImpl;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_util::Hand;

pub struct BundleItem;

impl ItemMetadata for BundleItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_BUNDLES.1.into()
    }
}

impl ItemBehaviour for BundleItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let held_item_ref = player.inventory.held_item();
            let held_item = held_item_ref.lock().await;
            if !held_item.is_empty() && Self::ids().contains(&held_item.item.id) {
                let stack = held_item.clone();
                drop(held_item);
                player
                    .living_entity
                    .set_active_hand(Hand::Right, stack, Self::USE_DURATION)
                    .await;
                return;
            }
            drop(held_item);

            let off_hand_item_ref = player.inventory.off_hand_item().await;
            let off_hand_item = off_hand_item_ref.lock().await;
            if !off_hand_item.is_empty() && Self::ids().contains(&off_hand_item.item.id) {
                let stack = off_hand_item.clone();
                drop(off_hand_item);
                player
                    .living_entity
                    .set_active_hand(Hand::Left, stack, Self::USE_DURATION)
                    .await;
            }
        })
    }

    fn on_use_tick<'a>(
        &'a self,
        _stack: &'a ItemStack,
        player: &'a Player,
        ticks_remaining: i32,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if Self::should_drop_content(ticks_remaining) {
                Self::drop_content(player).await;
            }
        })
    }

    fn on_destroyed<'a>(
        &'a self,
        entity: &'a ItemEntity,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let contents = {
                let mut bundle = entity.get_item_stack().lock().await;
                let Some(contents) = bundle.get_data_component_mut::<BundleContentsImpl>() else {
                    return;
                };
                contents.selected_item_index = -1;
                std::mem::take(&mut contents.items)
            };

            let base_entity = entity.get_entity();
            let world = base_entity.world.load_full();
            let position = base_entity.pos.load();
            for stack in contents {
                let entity = Entity::new(world.clone(), position, &EntityType::ITEM);
                world
                    .spawn_entity(Arc::new(ItemEntity::new(entity, stack)))
                    .await;
            }
        })
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BundleItem {
    const USE_DURATION: i32 = 200;

    const fn should_drop_content(ticks_remaining: i32) -> bool {
        ticks_remaining == Self::USE_DURATION
            || ticks_remaining < Self::USE_DURATION - 10 && ticks_remaining % 2 == 0
    }

    async fn drop_content(player: &Player) -> bool {
        let hand = *player.living_entity.active_hand.lock().await;
        let (slot_index, bundle_ref) = match hand {
            Some(Hand::Right) => (
                player.inventory.get_selected_slot() as usize,
                player.inventory.held_item(),
            ),
            Some(Hand::Left) => (
                PlayerInventory::OFF_HAND_SLOT,
                player.inventory.off_hand_item().await,
            ),
            None => return false,
        };

        let mut bundle = bundle_ref.lock().await;
        if bundle.is_empty() || !Self::ids().contains(&bundle.item.id) {
            return false;
        }
        let Some(bundle_contents) = bundle.get_data_component_mut::<BundleContentsImpl>() else {
            return false;
        };
        let Some(extracted_stack) = bundle_contents.try_extract() else {
            return false;
        };
        let updated_bundle = bundle.clone();
        drop(bundle);

        let position = player.position();
        player.world().play_sound(
            Sound::ItemBundleRemoveOne,
            SoundCategory::Players,
            &position,
        );
        player.drop_item(extracted_stack).await;
        player.world().play_sound(
            Sound::ItemBundleDropContents,
            SoundCategory::Players,
            &position,
        );
        player.sync_hand_slot(slot_index, updated_bundle).await;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::BundleItem;

    #[test]
    fn uses_vanilla_drop_timing() {
        assert!(BundleItem::should_drop_content(200));
        assert!(!BundleItem::should_drop_content(199));
        assert!(!BundleItem::should_drop_content(190));
        assert!(!BundleItem::should_drop_content(189));
        assert!(BundleItem::should_drop_content(188));
        assert!(!BundleItem::should_drop_content(187));
        assert!(BundleItem::should_drop_content(186));
    }
}
