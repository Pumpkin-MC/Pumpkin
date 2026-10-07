use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::data_component_impl::BundleContentsImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::tag;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_util::Hand;

pub struct BundleItem;

// Vanilla `BundleItem.TICKS_MAX_THROW_DURATION`.
const TICKS_MAX_THROW_DURATION: i32 = 200;
// Vanilla `BundleItem.TICKS_AFTER_FIRST_THROW`.
const TICKS_AFTER_FIRST_THROW: i32 = 10;
// Vanilla `BundleItem.TICKS_BETWEEN_THROWS`.
const TICKS_BETWEEN_THROWS: i32 = 2;

impl ItemMetadata for BundleItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_BUNDLES.1.into()
    }
}

impl ItemBehaviour for BundleItem {
    // Vanilla `BundleItem.use`, which only starts the use: the contents come out while the button
    // is held, in [`Self::on_use_tick`].
    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        let stack = player.inventory.get_stack_in_hand(hand);
        player
            .living_entity
            .set_active_hand(hand, stack, TICKS_MAX_THROW_DURATION);
    }

    // Vanilla `BundleItem.onUseTick`.
    fn on_use_tick(&self, _stack: &ItemStack, player: &Player, remaining_use_ticks: i32) {
        let is_first_tick = remaining_use_ticks == TICKS_MAX_THROW_DURATION;
        if is_first_tick
            || (remaining_use_ticks < TICKS_MAX_THROW_DURATION - TICKS_AFTER_FIRST_THROW
                && remaining_use_ticks % TICKS_BETWEEN_THROWS == 0)
        {
            Self::drop_content(player);
        }
    }

    fn get_use_duration(&self) -> i32 {
        TICKS_MAX_THROW_DURATION
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BundleItem {
    // Vanilla `BundleItem.dropContent` through `removeOneItemFromBundle`: the stack in the hand is
    // the one written back, so the dropped item actually leaves the bundle.
    fn drop_content(player: &Player) {
        let hand = player
            .living_entity
            .active_hand
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .unwrap_or(Hand::Right);
        let slot_index = if hand == Hand::Left {
            PlayerInventory::OFF_HAND_SLOT
        } else {
            player.inventory.get_selected_slot() as usize
        };

        let mut bundle = player.inventory.get_stack_in_hand(hand);
        let Some(contents) = bundle.get_data_component_mut::<BundleContentsImpl>() else {
            return;
        };
        let Some(item) = contents.remove_one() else {
            return;
        };

        let updated_bundle = bundle.clone();
        player
            .inventory
            .set_stack_in_hand(hand, updated_bundle.clone());
        player.sync_hand_slot(slot_index, updated_bundle);

        Self::play_sound(player, Sound::ItemBundleRemoveOne);
        player.drop_item(item);
        Self::play_sound(player, Sound::ItemBundleDropContents);
        player.increment_stat(StatisticCategory::Used, bundle.item.id as i32, 1);
    }

    // Vanilla `BundleItem.playRemoveOneSound` and the sounds around it.
    fn play_sound(player: &Player, sound: Sound) {
        player.play_item_sound(sound, 0.8, 0.8 + rand::random_range(0.0..1.0) * 0.4);
    }
}
