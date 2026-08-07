//! Anvil menu: repair, rename, enchantment merging and the prior-work-penalty cost curve.
//!
//! Mirrors `net/minecraft/world/inventory/AnvilMenu.java` (26.2 decompile, extending
//! `ItemCombinerMenu.java`). `create_result`/`may_pickup`/`on_take` are transcribed from
//! `createResult`/`mayPickup`/`onTake` there (line ranges cited on each function below).
//!
//! Two upstream mechanisms have no Pumpkin-side representation and are approximated, matching
//! the precedent already established in `grindstone_screen_handler.rs`:
//! - `RepairCostImpl` (`pumpkin-data`) carries no value, so the "prior work penalty" that vanilla
//!   reads back off `input`/`addition` (`AnvilMenu.java:129`, the `tax` accumulator) and re-writes
//!   onto the result (`AnvilMenu.java:255-264`) cannot persist across repeated anvil uses here.
//!   Every use effectively starts from a base cost of 0 for that term; the doubling formula
//!   itself (`calculate_increased_repair_cost`) is implemented and unit-tested.
//! - `RepairableImpl` (`pumpkin-data`) carries no item set, so `ItemStack.isValidRepairItem`
//!   (material repair, e.g. an iron ingot restoring an iron pickaxe) cannot be evaluated and is
//!   stubbed to `false`. This fails safe: such a pairing falls through to the same-item combine
//!   branch, which vanilla also takes for any non-matching, non-repair-material addition, so the
//!   result is "no repair happens" rather than a wrong repair amount.
//! - The 12% anvil-damage-on-use effect (`AnvilMenu.java:100-114`) needs the block position and
//!   world RNG, neither of which this crate has access to (same boundary noted in
//!   `grindstone_screen_handler.rs`'s module docs). Not implemented here.

use std::any::Any;
use std::sync::Arc;

use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    CustomNameImpl, DataComponentImpl, EnchantmentsImpl, StoredEnchantmentsImpl,
};
use pumpkin_data::item::Item;
use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use pumpkin_world::inventory::Inventory;

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture, offer_or_drop_stack,
    },
    slot::{BoxFuture, NormalSlot, Slot},
    window_property::{Anvil, WindowProperty},
};

/// `AnvilMenu.MAX_NAME_LENGTH` (AnvilMenu.java:30).
const MAX_NAME_LENGTH: usize = 50;

/// `AnvilMenu.calculateIncreasedRepairCost` (AnvilMenu.java:276-278).
#[must_use]
pub fn calculate_increased_repair_cost(base_cost: i32) -> i32 {
    i32::try_from((i64::from(base_cost) * 2 + 1).min(i64::from(i32::MAX))).unwrap_or(i32::MAX)
}

/// `EnchantmentHelper.getComponentType` (EnchantmentHelper.java:81-83): enchanted books carry
/// their enchantments in `StoredEnchantments`, every other item in `Enchantments`.
fn enchantments_for_crafting(stack: &ItemStack) -> Vec<(&'static Enchantment, i32)> {
    if stack.item == &Item::ENCHANTED_BOOK {
        stack
            .get_data_component::<StoredEnchantmentsImpl>()
            .map(|c| c.enchantment.to_vec())
            .unwrap_or_default()
    } else {
        stack
            .get_data_component::<EnchantmentsImpl>()
            .map(|c| c.enchantment.to_vec())
            .unwrap_or_default()
    }
}

/// `EnchantmentHelper.setEnchantments` (EnchantmentHelper.java:73-75).
fn set_enchantments_for_crafting(
    stack: &mut ItemStack,
    enchantments: Vec<(&'static Enchantment, i32)>,
) {
    let component = if stack.item == &Item::ENCHANTED_BOOK {
        DataComponent::StoredEnchantments
    } else {
        DataComponent::Enchantments
    };
    stack.patch.retain(|(id, _)| *id != component);
    let boxed: Box<dyn DataComponentImpl> = if component == DataComponent::StoredEnchantments {
        Box::new(StoredEnchantmentsImpl {
            enchantment: enchantments.into(),
        })
    } else {
        Box::new(EnchantmentsImpl {
            enchantment: enchantments.into(),
        })
    };
    stack.patch.push((component, Some(boxed)));
}

/// `EnchantmentHelper.canStoreEnchantments` (EnchantmentHelper.java:69-71): whether the item's
/// default components include an (Stored)Enchantments slot at all, not whether it currently has
/// any levels in it.
fn can_store_enchantments(stack: &ItemStack) -> bool {
    if stack.item == &Item::ENCHANTED_BOOK {
        stack
            .get_data_component::<StoredEnchantmentsImpl>()
            .is_some()
    } else {
        stack.get_data_component::<EnchantmentsImpl>().is_some()
    }
}

/// `ItemStack.isValidRepairItem` (ItemStack.java:1117-1120). See module docs: `RepairableImpl`
/// carries no item set in `pumpkin-data`, so this cannot be evaluated yet.
const fn is_valid_repair_item(_input: &ItemStack, _addition: &ItemStack) -> bool {
    false
}

/// Merges the addition's enchantments into `enchantments`, accumulating the level cost into
/// `price`. Returns `(any_compatible, any_incompatible)`.
///
/// Ports the enchantment-merge half of `AnvilMenu.createResult` (AnvilMenu.java:196-262);
/// split out of `create_result` to keep that function under the workspace line limit.
fn merge_enchantments(
    enchantments: &mut Vec<(&'static Enchantment, i32)>,
    additional: Vec<(&'static Enchantment, i32)>,
    input: &ItemStack,
    using_book: bool,
    infinite_materials: bool,
    price: &mut i32,
) -> (bool, bool) {
    let (mut any_compatible, mut any_incompatible) = (false, false);

    for (enchantment, level) in additional {
        let current = enchantments
            .iter()
            .find(|(e, _)| *e == enchantment)
            .map_or(0, |(_, l)| *l);
        let mut new_level = if current == level {
            level + 1
        } else {
            level.max(current)
        };

        let mut compatible = infinite_materials
            || input.item == &Item::ENCHANTED_BOOK
            || enchantment.can_enchant(input.item);

        for (other, _) in &*enchantments {
            if *other != enchantment && !enchantment.are_compatible(other) {
                compatible = false;
                *price += 1;
            }
        }

        if compatible {
            any_compatible = true;
            new_level = new_level.min(enchantment.max_level);

            if let Some(entry) = enchantments.iter_mut().find(|(e, _)| *e == enchantment) {
                entry.1 = new_level;
            } else {
                enchantments.push((enchantment, new_level));
            }

            let mut fee = i32::try_from(enchantment.anvil_cost).unwrap_or(i32::MAX);
            if using_book {
                fee = (fee / 2).max(1);
            }
            *price += fee * new_level;
            if input.item_count > 1 {
                *price = 40;
            }
        } else {
            any_incompatible = true;
        }
    }

    (any_compatible, any_incompatible)
}

/// Applies the rename half of `AnvilMenu.createResult` (AnvilMenu.java:264-274) to `result`
/// and returns the naming cost (1 when the name actually changed, else 0).
/// Split out of `create_result` to keep it under the workspace line limit.
fn apply_rename(rename_text: &str, input: &ItemStack, result: &mut ItemStack) -> i32 {
    if rename_text.is_empty() {
        if get_custom_name(input).is_some() {
            remove_custom_name(result);
            return 1;
        }
    } else if Some(rename_text) != get_custom_name(input).as_deref() {
        result.set_custom_name(rename_text.to_string());
        return 1;
    }
    0
}

fn get_custom_name(stack: &ItemStack) -> Option<String> {
    stack
        .get_data_component::<CustomNameImpl>()
        .map(|c| c.name.clone().get_text())
}

fn remove_custom_name(stack: &mut ItemStack) {
    stack
        .patch
        .retain(|(id, _)| *id != DataComponent::CustomName);
}

/// The output slot (`AnvilMenu.java:66-68`, inherited from `ItemCombinerMenu`): `mayPlace`
/// is always `false`, regardless of the held item. Without this, the generic pickup logic in
/// `screen_handler.rs` treats a mismatched cursor item as insertable and swaps it into the
/// slot instead of declining the click, handing the player the result for free.
struct AnvilResultSlot {
    inventory: Arc<dyn Inventory>,
    index: usize,
    id: std::sync::atomic::AtomicU8,
}

impl AnvilResultSlot {
    fn new(inventory: Arc<dyn Inventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: std::sync::atomic::AtomicU8::new(0),
        }
    }
}

impl Slot for AnvilResultSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id
            .store(id as u8, std::sync::atomic::Ordering::Relaxed);
    }

    fn can_insert(&self, _stack: &ItemStack) -> BoxFuture<'_, bool> {
        Box::pin(async move { false })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}

pub struct AnvilScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    behaviour: ScreenHandlerBehaviour,
    pub rename_text: String,
    pub repair_cost: i16,
    /// `AnvilMenu.repairItemCountCost` (AnvilMenu.java:31): how many of the addition stack a
    /// material repair consumes. Always 0 while [`is_valid_repair_item`] is stubbed.
    repair_item_count_cost: i32,
    /// `AnvilMenu.onlyRenaming` (AnvilMenu.java:34).
    only_renaming: bool,
}

impl AnvilScreenHandler {
    #[expect(clippy::needless_pass_by_value)]
    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
    ) -> Self {
        let mut handler = Self {
            inventory: inventory.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Anvil)),
            rename_text: String::new(),
            repair_cost: 0,
            repair_item_count_cost: 0,
            only_renaming: false,
        };

        // Anvil specific slots: 2 input, 1 output
        for i in 0..2 {
            handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), i)));
        }
        handler.add_slot(Arc::new(AnvilResultSlot::new(inventory, 2)));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    /// `AnvilMenu.setItemName` (AnvilMenu.java:280-298); `validateName` (AnvilMenu.java:300-303)
    /// only implements the length cap, not the full `StringUtil.filterText` control-character
    /// strip.
    pub async fn update_item_name(&mut self, name: String, player: &dyn InventoryPlayer) {
        if name.chars().count() > MAX_NAME_LENGTH {
            return;
        }
        if name == self.rename_text {
            return;
        }
        self.rename_text = name;
        self.create_result(player).await;
        self.send_content_updates().await;
    }

    /// `AnvilMenu.createResult` (AnvilMenu.java:117-274).
    pub async fn create_result(&mut self, player: &dyn InventoryPlayer) {
        let input = {
            let lock = self.inventory.get_stack(0).await;
            lock.lock().await.clone()
        };

        self.only_renaming = false;
        self.set_repair_cost(1).await;

        if input.is_empty() || !can_store_enchantments(&input) {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
            self.set_repair_cost(0).await;
            return;
        }

        let mut result = input.clone();
        let addition = {
            let lock = self.inventory.get_stack(1).await;
            lock.lock().await.clone()
        };
        let mut enchantments = enchantments_for_crafting(&result);
        // Prior-work-penalty tax: see module docs, always 0 given the RepairCostImpl gap.
        let tax: i64 = 0;
        self.repair_item_count_cost = 0;
        let mut price: i32 = 0;

        if !addition.is_empty()
            && self.apply_addition_item(
                &input,
                &addition,
                &mut result,
                &mut enchantments,
                &mut price,
                player,
            )
        {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
            self.set_repair_cost(0).await;
            return;
        }

        let naming_cost = apply_rename(&self.rename_text, &input, &mut result);
        price += naming_cost;

        let final_price = if price <= 0 {
            0
        } else {
            (i64::from(price) + tax).clamp(0, i64::from(i32::MAX))
        };
        self.set_repair_cost(
            i16::try_from(final_price.min(i64::from(i16::MAX))).unwrap_or(i16::MAX),
        )
        .await;
        if price <= 0 {
            result = ItemStack::EMPTY.clone();
        }

        if naming_cost == price && naming_cost > 0 {
            if self.repair_cost >= 40 {
                self.set_repair_cost(39).await;
            }
            self.only_renaming = true;
        }

        if self.repair_cost >= 40 && !player.has_infinite_materials() {
            result = ItemStack::EMPTY.clone();
        }

        if !result.is_empty() {
            set_enchantments_for_crafting(&mut result, enchantments);
        }

        self.inventory.set_stack(2, result).await;
    }

    /// Applies the second-slot item to `result`: either material repair or enchantment merge,
    /// matching `AnvilMenu.createResult`'s addition-item branch (AnvilMenu.java:130-236).
    /// Returns `true` if the combination is invalid and the caller should reset to empty.
    fn apply_addition_item(
        &mut self,
        input: &ItemStack,
        addition: &ItemStack,
        result: &mut ItemStack,
        enchantments: &mut Vec<(&'static Enchantment, i32)>,
        price: &mut i32,
        player: &dyn InventoryPlayer,
    ) -> bool {
        let using_book = addition
            .get_data_component::<StoredEnchantmentsImpl>()
            .is_some();

        if result.is_damageable() && is_valid_repair_item(input, addition) {
            let mut repair_amount = result
                .get_damage()
                .min(result.get_max_damage().unwrap_or(0) / 4);
            if repair_amount <= 0 {
                return true;
            }

            let mut count = 0;
            while repair_amount > 0 && count < addition.item_count {
                let result_damage = result.get_damage() - repair_amount;
                result.set_damage(result_damage);
                *price += 1;
                repair_amount = result
                    .get_damage()
                    .min(result.get_max_damage().unwrap_or(0) / 4);
                count += 1;
            }
            self.repair_item_count_cost = i32::from(count);
        } else {
            if !using_book && (result.item != addition.item || !result.is_damageable()) {
                return true;
            }

            if result.is_damageable() && !using_book {
                let remaining1 = input.get_max_damage().unwrap_or(0) - input.get_damage();
                let remaining2 = addition.get_max_damage().unwrap_or(0) - addition.get_damage();
                let additional = remaining2 + result.get_max_damage().unwrap_or(0) * 12 / 100;
                let remaining = remaining1 + additional;
                let result_damage = (result.get_max_damage().unwrap_or(0) - remaining).max(0);
                if result_damage < result.get_damage() {
                    result.set_damage(result_damage);
                    *price += 2;
                }
            }

            let (any_compatible, any_incompatible) = merge_enchantments(
                enchantments,
                enchantments_for_crafting(addition),
                input,
                using_book,
                player.has_infinite_materials(),
                price,
            );

            if any_incompatible && !any_compatible {
                return true;
            }
        }
        false
    }

    pub async fn set_repair_cost(&mut self, cost: i16) {
        self.repair_cost = cost;
        if let Some(sync_handler) = self.behaviour.sync_handler.as_ref() {
            let (property_id, property_value) =
                WindowProperty::new(Anvil::RepairCost, cost).into_tuple();
            sync_handler
                .update_property(&self.behaviour, property_id as i32, property_value as i32)
                .await;
        }
    }

    /// `ItemCombinerMenu.mayPickup` (`AnvilMenu.mayPickup`, AnvilMenu.java:70-72).
    fn may_pickup(&self, player: &dyn InventoryPlayer) -> bool {
        (player.has_infinite_materials()
            || player.experience_level() >= i32::from(self.repair_cost))
            && self.repair_cost > 0
    }

    /// `AnvilMenu.onTake` (AnvilMenu.java:74-115), minus the block-damage effect (see module
    /// docs).
    async fn on_take(&mut self, player: &dyn InventoryPlayer) {
        if !player.has_infinite_materials() {
            player
                .add_experience_levels(-i32::from(self.repair_cost))
                .await;
        }

        if self.repair_item_count_cost > 0 {
            let addition = {
                let lock = self.inventory.get_stack(1).await;
                lock.lock().await.clone()
            };
            if !addition.is_empty() && i32::from(addition.item_count) > self.repair_item_count_cost
            {
                let mut shrunk = addition;
                shrunk.decrement(u8::try_from(self.repair_item_count_cost).unwrap_or(u8::MAX));
                self.inventory.set_stack(1, shrunk).await;
            } else {
                self.inventory.set_stack(1, ItemStack::EMPTY.clone()).await;
            }
        } else if !self.only_renaming {
            self.inventory.set_stack(1, ItemStack::EMPTY.clone()).await;
        }

        self.set_repair_cost(0).await;
        self.inventory.set_stack(0, ItemStack::EMPTY.clone()).await;
    }
}

impl ScreenHandler for AnvilScreenHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            self.inventory.on_close().await;
            // Drop inputs from anvil
            for i in 0..2 {
                let stack = self.inventory.remove_stack(i).await;
                if !stack.is_empty() {
                    offer_or_drop_stack(player, stack).await;
                }
            }
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
        })
    }

    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let mut stack_left = ItemStack::EMPTY.clone();
            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if slot.has_stack().await {
                let slot_stack_lock = slot.get_stack().await;
                let slot_stack_guard = slot_stack_lock.lock().await;
                stack_left = slot_stack_guard.clone();
                drop(slot_stack_guard);

                let mut slot_stack_mut = slot_stack_lock.lock().await;
                let prev_count = slot_stack_mut.item_count;

                if slot_index < 3 {
                    // From anvil to player
                    if !self
                        .insert_item(
                            &mut slot_stack_mut,
                            3,
                            self.get_behaviour().slots.len() as i32,
                            true,
                        )
                        .await
                    {
                        return ItemStack::EMPTY.clone();
                    }
                } else {
                    // From player to anvil input 0 and 1
                    if !self.insert_item(&mut slot_stack_mut, 0, 2, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                }

                let took = slot_index == 2 && slot_stack_mut.item_count != prev_count;

                if slot_stack_mut.is_empty() {
                    drop(slot_stack_mut);
                    slot.set_stack(ItemStack::EMPTY.clone()).await;
                } else {
                    drop(slot_stack_mut);
                    slot.mark_dirty().await;
                }

                // `ItemCombinerMenu.quickMoveStack` calls `slot.onTake` unconditionally once the
                // result slot's count changed, with no `mayPickup` gate (ItemCombinerMenu.java
                // quickMoveStack, `slotIndex == this.getResultSlot()` branch).
                if took {
                    self.on_take(player).await;
                    self.create_result(player).await;
                    self.send_content_updates().await;
                }
            }

            stack_left
        })
    }

    fn on_slot_click<'a>(
        &'a mut self,
        slot_index: i32,
        button: i32,
        action_type: pumpkin_protocol::java::server::play::SlotActionType,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            // `AbstractContainerMenu.doClick` only calls `slot.onTake` after `slot.tryRemove`
            // actually removed the stack (AbstractContainerMenu.java:420-464), reached only via
            // the `slot.mayPickup(player)` branch. `may_pickup` mirrors that gate; the actual
            // removal is left to `internal_on_slot_click` (now that `AnvilResultSlot::can_insert`
            // matches `mayPlace == false`, that call only removes the slot's stack, never a
            // mismatched-item swap), and `on_take` fires afterward only if the count dropped.
            let mut prev_result_count = 0;
            if slot_index == 2 {
                let result_slot = self.get_behaviour().slots[2].clone();
                if result_slot.has_stack().await {
                    if !self.may_pickup(player) {
                        self.send_content_updates().await;
                        return;
                    }
                    prev_result_count = result_slot.get_cloned_stack().await.item_count;
                }
            }

            let was_quick_move =
                action_type == pumpkin_protocol::java::server::play::SlotActionType::QuickMove;
            self.internal_on_slot_click(slot_index, button, action_type, player)
                .await;

            // `QuickMove` is excluded: `internal_on_slot_click` routes it into our own
            // `quick_move` override, which already calls `on_take` itself on a successful
            // take. Calling it again here would charge XP and wipe the inputs a second time.
            if slot_index == 2 && prev_result_count > 0 && !was_quick_move {
                let new_count = self.get_behaviour().slots[2]
                    .get_cloned_stack()
                    .await
                    .item_count;
                if new_count < prev_result_count {
                    self.on_take(player).await;
                }
            }

            if slot_index == 0 || slot_index == 1 || slot_index == 2 {
                self.create_result(player).await;
                self.send_content_updates().await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_equipment_slots, entity_equipment::EntityEquipment};
    use pumpkin_data::item::Item;
    use pumpkin_world::inventory::SimpleInventory;
    use tokio::sync::Mutex as TokioMutex;

    fn handler() -> AnvilScreenHandler {
        let player_inventory = Arc::new(PlayerInventory::new(
            Arc::new(TokioMutex::new(EntityEquipment::new())),
            Arc::new(build_equipment_slots()),
        ));
        let inventory: Arc<dyn Inventory> = Arc::new(SimpleInventory::new(3));
        AnvilScreenHandler::new(0, &player_inventory, inventory)
    }

    #[tokio::test]
    async fn result_slot_never_accepts_items() {
        // `AnvilMenu.java:66-68` (`ItemCombinerMenu`'s result slot): `mayPlace` is always
        // `false`, matched by `AnvilResultSlot::can_insert`.
        let handler = handler();
        let result_slot = handler.get_behaviour().slots[2].clone();
        assert!(
            !result_slot
                .can_insert(&ItemStack::new(1, &Item::DIRT))
                .await
        );
    }

    #[test]
    fn repair_cost_matches_vanilla_formula() {
        assert_eq!(calculate_increased_repair_cost(0), 1);
        assert_eq!(calculate_increased_repair_cost(1), 3);
        assert_eq!(calculate_increased_repair_cost(3), 7);
        assert_eq!(calculate_increased_repair_cost(i32::MAX), i32::MAX);
        assert_eq!(calculate_increased_repair_cost(i32::MAX / 2), i32::MAX);
    }

    #[test]
    fn can_store_enchantments_matches_default_component_presence() {
        assert!(can_store_enchantments(&ItemStack::new(
            1,
            &Item::IRON_PICKAXE
        )));
        assert!(can_store_enchantments(&ItemStack::new(
            1,
            &Item::ENCHANTED_BOOK
        )));
    }

    #[test]
    fn enchantments_round_trip_through_the_crafting_component() {
        let mut item = ItemStack::new(1, &Item::IRON_PICKAXE);
        set_enchantments_for_crafting(&mut item, vec![(&Enchantment::EFFICIENCY, 3)]);
        let round_tripped = enchantments_for_crafting(&item);
        // Enchantment has no Debug impl, so compare by identity and level rather than
        // asserting on the Vec directly.
        assert_eq!(round_tripped.len(), 1);
        assert!(std::ptr::eq(round_tripped[0].0, &Enchantment::EFFICIENCY));
        assert_eq!(round_tripped[0].1, 3);
    }

    #[test]
    fn custom_name_get_and_remove_round_trip() {
        let mut item = ItemStack::new(1, &Item::IRON_PICKAXE);
        assert_eq!(get_custom_name(&item), None);
        item.set_custom_name("Excalibur".to_string());
        assert_eq!(get_custom_name(&item), Some("Excalibur".to_string()));
        remove_custom_name(&mut item);
        assert_eq!(get_custom_name(&item), None);
    }
}
