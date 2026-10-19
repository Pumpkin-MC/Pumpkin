//! Bundles override what the vanilla container click path does with them.
//!
//! Vanilla hangs these overrides off `BundleItem`, reached through
//! `AbstractContainerMenu.tryItemClickBehaviourOverride`. Item behaviour lives in `pumpkin-core`,
//! which depends on this crate, so the click path cannot call into it: the bundle overrides it needs
//! live here instead.

use std::sync::Arc;

use pumpkin_data::data_component_impl::BundleContentsImpl;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;

use crate::container_click::MouseClick;
use crate::screen_handler::InventoryPlayer;
use crate::slot::Slot;

// Vanilla `AbstractContainerMenu.tryItemClickBehaviourOverride`. Returns true when an item override
// consumed the click.
pub(crate) fn try_item_click_behaviour_override(
    player: &dyn InventoryPlayer,
    click_type: &MouseClick,
    slot: &Arc<dyn Slot>,
    clicked: &ItemStack,
    carried: &mut ItemStack,
) -> bool {
    override_stacked_on_other(player, click_type, slot, clicked, carried)
        || override_other_stacked_on_me(player, click_type, slot, carried)
}

// Vanilla `BundleContents.Mutable.tryTransfer`.
fn try_transfer(
    contents: &mut BundleContentsImpl,
    slot: &Arc<dyn Slot>,
    player: &dyn InventoryPlayer,
) -> u8 {
    let other = slot.get_cloned_stack();
    if !BundleContentsImpl::can_item_be_in_bundle(&other) {
        return 0;
    }
    let max_amount = contents.get_max_amount_to_add(BundleContentsImpl::weight_of(&other));
    let mut taken = slot.safe_take(other.item_count, max_amount, player);
    let inserted = contents.try_insert(&mut taken);
    if taken.item_count > 0 {
        slot.insert_stack(taken);
    }
    inserted
}

// Vanilla `BundleItem.overrideStackedOnOther`, called with a bundle in the cursor.
fn override_stacked_on_other(
    player: &dyn InventoryPlayer,
    click_type: &MouseClick,
    slot: &Arc<dyn Slot>,
    other: &ItemStack,
    carried: &mut ItemStack,
) -> bool {
    let Some(mut contents) = carried.get_data_component::<BundleContentsImpl>().cloned() else {
        return false;
    };
    let pitch: f32 = 0.8 + rand::random_range(0.0..1.0) * 0.4;

    if *click_type == MouseClick::Left && !other.is_empty() {
        if try_transfer(&mut contents, slot, player) > 0 {
            player.play_item_sound(Sound::ItemBundleInsert, 0.8, pitch);
        } else {
            player.play_item_sound(Sound::ItemBundleInsertFail, 1.0, 1.0);
        }
    } else if *click_type == MouseClick::Right && other.is_empty() {
        if let Some(item) = contents.remove_one() {
            let mut leftover = slot.insert_stack(item);
            if leftover.item_count > 0 {
                contents.try_insert(&mut leftover);
            } else {
                player.play_item_sound(Sound::ItemBundleRemoveOne, 0.8, pitch);
            }
        }
    } else {
        return false;
    }

    carried.set_data_component(contents);
    true
}

// Vanilla `BundleItem.overrideOtherStackedOnMe`, called with a bundle in the clicked slot.
fn override_other_stacked_on_me(
    player: &dyn InventoryPlayer,
    click_type: &MouseClick,
    slot: &Arc<dyn Slot>,
    other: &mut ItemStack,
) -> bool {
    if *click_type == MouseClick::Left && other.is_empty() {
        // Vanilla `toggleSelectedItem(self, -1)`, which needs the bundle selection Pumpkin has no
        // state for yet, and leaves the click to the default behaviour.
        return false;
    }

    let mut bundle_stack = slot.get_cloned_stack();
    let Some(mut contents) = bundle_stack
        .get_data_component::<BundleContentsImpl>()
        .cloned()
    else {
        return false;
    };
    let pitch: f32 = 0.8 + rand::random_range(0.0..1.0) * 0.4;

    if *click_type == MouseClick::Left && !other.is_empty() {
        if slot.allow_modification(player) && contents.try_insert(other) > 0 {
            player.play_item_sound(Sound::ItemBundleInsert, 0.8, pitch);
        } else {
            player.play_item_sound(Sound::ItemBundleInsertFail, 1.0, 1.0);
        }
    } else if *click_type == MouseClick::Right && other.is_empty() {
        if slot.allow_modification(player)
            && let Some(item) = contents.remove_one()
        {
            player.play_item_sound(Sound::ItemBundleRemoveOne, 0.8, pitch);
            *other = item;
        }
    } else {
        return false;
    }

    bundle_stack.set_data_component(contents);
    slot.set_stack(bundle_stack);
    true
}
