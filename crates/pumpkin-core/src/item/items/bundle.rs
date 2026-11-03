use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::BundleContentsImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::tag;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_util::Hand;

pub struct BundleItem;

impl BundleItem {
    pub const USE_DURATION: i32 = 200;
}

impl ItemMetadata for BundleItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_BUNDLES.1.into()
    }
}

impl ItemBehaviour for BundleItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        let stack = player.inventory.get_stack_in_hand(hand);
        if stack.is_empty() || !Self::ids().contains(&stack.item.id) {
            return;
        }
        player
            .living_entity
            .set_active_hand(hand, stack, Self::USE_DURATION);
    }

    fn on_use_tick(&self, stack: &ItemStack, player: &Player, remaining_use_ticks: i32) {
        let is_first_tick = remaining_use_ticks == Self::USE_DURATION;
        if !is_first_tick
            && (remaining_use_ticks >= Self::USE_DURATION - 10 || remaining_use_ticks % 2 != 0)
        {
            return;
        }

        let active_hand = *player
            .living_entity
            .active_hand
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let hand = active_hand.unwrap_or(Hand::Right);
        let mut held = player.inventory.get_stack_in_hand(hand);
        if held.is_empty() || held.item.id != stack.item.id {
            return;
        }

        if let Some(bundle_contents) = held.get_data_component_mut::<BundleContentsImpl>()
            && let Some(extracted_stack) = bundle_contents.try_extract()
        {
            let position = player.position();
            player.world().play_sound_fine(
                Sound::ItemBundleDropContents,
                SoundCategory::Players,
                &position,
                0.8,
                0.8 + rand::random::<f32>() * 0.4,
            );
            let updated_bundle = held.clone();

            player.drop_item(extracted_stack);
            player
                .inventory
                .set_stack_in_hand(hand, updated_bundle.clone());
            let slot_index = match hand {
                Hand::Right => player.inventory.get_selected_slot() as usize,
                Hand::Left => PlayerInventory::OFF_HAND_SLOT,
            };
            player.sync_hand_slot(slot_index, updated_bundle.clone());
            *player
                .living_entity
                .item_in_use
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(updated_bundle);
            player.increment_stat(StatisticCategory::Used, i32::from(stack.item.id), 1);
        }
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
