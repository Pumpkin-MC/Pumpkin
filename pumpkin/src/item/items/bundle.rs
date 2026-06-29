use std::pin::Pin;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::BundleContentsImpl;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag;

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
            let mut held_item = held_item_ref.lock().await;
            let mut matched = false;
            let mut used_slot_index = player.inventory.get_selected_slot() as usize;

            if !held_item.is_empty() && Self::ids().contains(&held_item.item.id) {
                matched = true;
                if let Some(bundle_contents) =
                    held_item.get_data_component_mut::<BundleContentsImpl>()
                    && let Some(extracted_stack) = bundle_contents.try_extract()
                {
                    let position = player.position();
                    player.world().play_sound(
                        Sound::ItemBundleRemoveOne,
                        pumpkin_data::sound::SoundCategory::Players,
                        &position,
                    );
                    let updated_bundle = held_item.clone();
                    drop(held_item);

                    player.drop_item(extracted_stack).await;
                    player.sync_hand_slot(used_slot_index, updated_bundle).await;
                }
            }

            if !matched {
                let off_hand_item_ref = player.inventory.off_hand_item().await;
                let mut off_hand_item = off_hand_item_ref.lock().await;
                if !off_hand_item.is_empty() && Self::ids().contains(&off_hand_item.item.id) {
                    used_slot_index = 40; // OFF_HAND_SLOT
                    if let Some(bundle_contents) =
                        off_hand_item.get_data_component_mut::<BundleContentsImpl>()
                        && let Some(extracted_stack) = bundle_contents.try_extract()
                    {
                        let position = player.position();
                        player.world().play_sound(
                            Sound::ItemBundleRemoveOne,
                            pumpkin_data::sound::SoundCategory::Players,
                            &position,
                        );
                        let updated_bundle = off_hand_item.clone();
                        drop(off_hand_item);

                        player.drop_item(extracted_stack).await;
                        player.sync_hand_slot(used_slot_index, updated_bundle).await;
                    }
                }
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
