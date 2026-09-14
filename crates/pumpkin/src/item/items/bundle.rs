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
    fn normal_use(&self, _item: &Item, player: &Player) {
        let mut held_item = player.inventory.held_item();
        let mut matched = false;
        let mut used_slot_index = player.inventory.get_selected_slot() as usize;

        if !held_item.is_empty() && Self::ids().contains(&held_item.item.id) {
            matched = true;
            if let Some(bundle_contents) = held_item.get_data_component_mut::<BundleContentsImpl>()
                && let Some(extracted_stack) = bundle_contents.try_extract()
            {
                let position = player.position();
                player.world().play_sound(
                    Sound::ItemBundleRemoveOne,
                    pumpkin_data::sound::SoundCategory::Players,
                    &position,
                );
                let updated_bundle = held_item.clone();

                player.drop_item(extracted_stack);
                player.sync_hand_slot(used_slot_index, updated_bundle);
            }
        }

        if !matched {
            let mut off_hand_item = player.inventory.off_hand_item();
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

                    player.drop_item(extracted_stack);
                    player.sync_hand_slot(used_slot_index, updated_bundle);
                }
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
