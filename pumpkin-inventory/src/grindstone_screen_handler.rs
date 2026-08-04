//! Grindstone menu: repairs/combines two damageable items, or strips non-curse enchantments
//! from a single item, awarding XP based on the enchantments removed.
//!
//! Mirrors `net/minecraft/world/inventory/GrindstoneMenu.java` (26.2 decompile). Slot layout,
//! `quickMoveStack` ranges (`INV_SLOT_START`/`END` = 3/30, `USE_ROW_SLOT_START`/`END` = 30/39,
//! result at 2) and `computeResult`/`mergeItems`/`removeNonCursesFrom` are transcribed from that
//! file (exact line ranges cited on each function below).
//!
//! Two upstream mechanisms have no Pumpkin-side representation and are approximated:
//! - `RepairCostImpl` is a unit struct with no carried value, so the vanilla
//!   `item.set(DataComponents.REPAIR_COST, repairCost)` write (GrindstoneMenu.java:196) cannot be
//!   persisted onto the output item. The formula itself is implemented and unit-tested.
//! - Vanilla spawns a physical XP orb entity in the world on take (GrindstoneMenu.java:68-74).
//!   Pumpkin's furnace screen handler (see `furnace_like_slot.rs`) already established the
//!   precedent of awarding XP directly to the player instead of spawning a visible orb; this
//!   follows that precedent rather than inventing a new cross-crate world-access path (the
//!   screen handler, in `pumpkin-inventory`, has no access to the concrete `World` type that
//!   lives in the `pumpkin` crate, which depends on `pumpkin-inventory` and not vice versa).

use std::any::Any;
use std::sync::Arc;

use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    DataComponentImpl, EnchantmentsImpl, MaxDamageImpl, StoredEnchantmentsImpl,
};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::screen::WindowType;
use pumpkin_data::tag::{Enchantment as EnchantmentTag, Taggable};
use pumpkin_protocol::java::server::play::SlotActionType;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::inventory::SimpleInventory;

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
    offer_or_drop_stack,
};
use crate::slot::{BoxFuture, Slot};

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

/// `EnchantmentHelper.hasAnyEnchantments` (EnchantmentHelper.java:85-88): checks both components,
/// unlike `ItemStack::has_enchantments`, which only checks `Enchantments`.
fn has_any_enchantments(stack: &ItemStack) -> bool {
    stack.has_enchantments()
        || stack
            .get_data_component::<StoredEnchantmentsImpl>()
            .is_some_and(|c| !c.enchantment.is_empty())
}

/// Replaces the crafting-relevant enchantment component entirely (the counterpart to
/// [`enchantments_for_crafting`]).
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
    if !enchantments.is_empty() {
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
}

/// Sets the `MaxDamage` component. No public setter exists on `ItemStack` for it (only
/// `set_damage`, for `Damage`); this mirrors that function's shape.
fn set_max_damage(stack: &mut ItemStack, max_damage: i32) {
    stack
        .patch
        .retain(|(id, _)| *id != DataComponent::MaxDamage);
    stack.patch.push((
        DataComponent::MaxDamage,
        Some(Box::new(MaxDamageImpl { max_damage })),
    ));
}

fn is_curse(enchantment: &'static Enchantment) -> bool {
    enchantment.has_tag(&EnchantmentTag::MINECRAFT_CURSE)
}

/// `GrindstoneMenu.removeNonCursesFrom` (GrindstoneMenu.java:182-198), minus the unrepresentable
/// `REPAIR_COST` write (see module docs).
fn remove_non_curses_from(mut item: ItemStack) -> ItemStack {
    let kept: Vec<_> = enchantments_for_crafting(&item)
        .into_iter()
        .filter(|(e, _)| is_curse(e))
        .collect();
    set_enchantments_for_crafting(&mut item, kept.clone());

    if item.item == &Item::ENCHANTED_BOOK && kept.is_empty() {
        item = ItemStack::new_with_component(item.item_count, &Item::BOOK, item.patch.clone());
    }

    item
}

/// `GrindstoneMenu.mergeEnchantsFrom` (GrindstoneMenu.java:169-180): each of `source`'s
/// enchantments is merged into `target` via vanilla's max-upgrade rule. Curses are skipped only
/// when `target` *already* carries that exact curse (a no-op in practice, since curses are all
/// single-level) — otherwise a curse transfers onto `target` exactly like any other enchantment,
/// same as a non-curse would.
fn merge_enchants_from(target: &mut ItemStack, source: &ItemStack) {
    let mut merged = enchantments_for_crafting(target);
    for (enchant, level) in enchantments_for_crafting(source) {
        let existing_level = merged
            .iter()
            .find(|(e, _)| *e == enchant)
            .map_or(0, |(_, l)| *l);
        if !is_curse(enchant) || existing_level == 0 {
            let new_level = existing_level.max(level).min(255);
            if let Some(entry) = merged.iter_mut().find(|(e, _)| *e == enchant) {
                entry.1 = new_level;
            } else {
                merged.push((enchant, new_level));
            }
        }
    }
    set_enchantments_for_crafting(target, merged);
}

/// `GrindstoneMenu.mergeItems` (GrindstoneMenu.java:141-167).
fn merge_items(input: &ItemStack, additional: &ItemStack) -> ItemStack {
    if input.item != additional.item {
        return ItemStack::EMPTY.clone();
    }

    let durability = input
        .get_max_damage()
        .unwrap_or(0)
        .max(additional.get_max_damage().unwrap_or(0));
    let remaining1 = input.get_max_damage().unwrap_or(0) - input.get_damage();
    let remaining2 = additional.get_max_damage().unwrap_or(0) - additional.get_damage();
    let remaining = remaining1 + remaining2 + durability * 5 / 100;

    let count = if input.is_damageable() {
        1
    } else {
        if input.get_max_stack_size() < 2 || !input.are_items_and_components_equal(additional) {
            return ItemStack::EMPTY.clone();
        }
        2
    };

    let mut new_item = input.copy_with_count(count);
    if new_item.is_damageable() {
        set_max_damage(&mut new_item, durability);
        new_item.set_damage((durability - remaining).max(0));
    }

    merge_enchants_from(&mut new_item, additional);
    remove_non_curses_from(new_item)
}

/// `GrindstoneMenu.computeResult` (GrindstoneMenu.java:122-139).
fn compute_result(input: &ItemStack, additional: &ItemStack) -> ItemStack {
    let has_an_item = !input.is_empty() || !additional.is_empty();
    if !has_an_item {
        return ItemStack::EMPTY.clone();
    }

    if input.item_count <= 1 && additional.item_count <= 1 {
        let has_both_items = !input.is_empty() && !additional.is_empty();
        if has_both_items {
            merge_items(input, additional)
        } else {
            let item = if input.is_empty() { additional } else { input };
            if has_any_enchantments(item) {
                remove_non_curses_from(item.clone())
            } else {
                ItemStack::EMPTY.clone()
            }
        }
    } else {
        ItemStack::EMPTY.clone()
    }
}

/// `GrindstoneMenu$2.getExperienceFromItem` (GrindstoneMenu.java:91-104): sum of
/// `Enchantment.getMinCost(level)` over every non-curse enchantment on `item`.
fn experience_from_item(item: &ItemStack) -> i32 {
    enchantments_for_crafting(item)
        .into_iter()
        .filter(|(e, _)| !is_curse(e))
        .map(|(e, level)| e.min_cost.calculate(level))
        .sum()
}

/// `GrindstoneMenu$2.getExperienceAmount` (GrindstoneMenu.java:79-89).
fn experience_amount(input: &ItemStack, additional: &ItemStack) -> i32 {
    let amount = experience_from_item(input) + experience_from_item(additional);
    if amount > 0 {
        // `ceil(amount / 2.0)` for positive `amount`, without relying on unstable signed div_ceil.
        let half_amount = (amount + 1) / 2;
        half_amount + rand::random_range(0..half_amount)
    } else {
        0
    }
}

pub struct GrindstoneScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    pub repair_inventory: Arc<SimpleInventory>,
    pub result_inventory: Arc<SimpleInventory>,
}

impl GrindstoneScreenHandler {
    pub fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let behaviour = ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Grindstone));
        let repair_inventory = Arc::new(SimpleInventory::new(2));
        let result_inventory = Arc::new(SimpleInventory::new(1));

        let mut handler = Self {
            behaviour,
            repair_inventory: repair_inventory.clone(),
            result_inventory: result_inventory.clone(),
        };

        handler.add_slot(Arc::new(GrindstoneRepairSlot::new(
            repair_inventory.clone() as Arc<dyn Inventory>,
            0,
        )));
        handler.add_slot(Arc::new(GrindstoneRepairSlot::new(
            repair_inventory.clone() as Arc<dyn Inventory>,
            1,
        )));
        handler.add_slot(Arc::new(GrindstoneResultSlot::new(
            result_inventory as Arc<dyn Inventory>,
            repair_inventory,
            0,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    async fn update_result(&self) {
        let input = self.repair_inventory.get_stack(0).await;
        let input = input.lock().await.clone();
        let additional = self.repair_inventory.get_stack(1).await;
        let additional = additional.lock().await.clone();

        self.result_inventory
            .set_stack(0, compute_result(&input, &additional))
            .await;
    }
}

impl ScreenHandler for GrindstoneScreenHandler {
    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            for i in 0..2 {
                let stack = self.repair_inventory.remove_stack(i).await;
                if !stack.is_empty() {
                    offer_or_drop_stack(player, stack).await;
                }
            }
        })
    }

    fn on_slot_click<'a>(
        &'a mut self,
        slot_index: i32,
        button: i32,
        action_type: SlotActionType,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.internal_on_slot_click(slot_index, button, action_type, player)
                .await;
            if slot_index == 0 || slot_index == 1 {
                self.update_result().await;
            }
        })
    }

    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let mut stack_left = ItemStack::EMPTY.clone();
            let slot = self.get_behaviour().slots.get(slot_index as usize).cloned();

            let Some(slot) = slot else {
                return stack_left;
            };
            if !slot.has_stack().await {
                return stack_left;
            }

            let mut item = slot.get_cloned_stack().await;
            stack_left = item.clone();

            if slot_index == 2 {
                if !self.insert_item(&mut item, 3, 39, true).await {
                    return ItemStack::EMPTY.clone();
                }
                slot.on_quick_move_crafted(item.clone(), stack_left.clone())
                    .await;
            } else if slot_index != 0 && slot_index != 1 {
                let input_empty = self
                    .repair_inventory
                    .get_stack(0)
                    .await
                    .lock()
                    .await
                    .is_empty();
                let additional_empty = self
                    .repair_inventory
                    .get_stack(1)
                    .await
                    .lock()
                    .await
                    .is_empty();

                if !input_empty && !additional_empty {
                    if (3..30).contains(&slot_index) {
                        if !self.insert_item(&mut item, 30, 39, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    } else if (30..39).contains(&slot_index)
                        && !self.insert_item(&mut item, 3, 30, false).await
                    {
                        return ItemStack::EMPTY.clone();
                    }
                } else if !self.insert_item(&mut item, 0, 2, false).await {
                    return ItemStack::EMPTY.clone();
                }
            } else if !self.insert_item(&mut item, 3, 39, false).await {
                return ItemStack::EMPTY.clone();
            }

            if item.is_empty() {
                slot.set_stack(ItemStack::EMPTY.clone()).await;
            } else {
                slot.mark_dirty().await;
            }

            if item.item_count == stack_left.item_count {
                return ItemStack::EMPTY.clone();
            }

            slot.on_take_item(player, &item).await;
            if slot_index == 0 || slot_index == 1 {
                self.update_result().await;
            }
            stack_left
        })
    }
}

/// `mayPlace` predicate shared by both repair slots (GrindstoneMenu.java:50-52/56-58).
struct GrindstoneRepairSlot {
    inventory: Arc<dyn Inventory>,
    index: usize,
    id: std::sync::atomic::AtomicU8,
}

impl GrindstoneRepairSlot {
    fn new(inventory: Arc<dyn Inventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: std::sync::atomic::AtomicU8::new(0),
        }
    }
}

impl Slot for GrindstoneRepairSlot {
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

    fn can_insert<'a>(&'a self, stack: &'a ItemStack) -> BoxFuture<'a, bool> {
        Box::pin(async move { stack.is_damageable() || has_any_enchantments(stack) })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}

/// `GrindstoneMenu`'s result slot (GrindstoneMenu.java:60-105): never accepts items, and on take
/// awards XP (see module docs for the physical-orb-vs-direct-award note) and clears both repair
/// slots.
struct GrindstoneResultSlot {
    inventory: Arc<dyn Inventory>,
    repair_inventory: Arc<SimpleInventory>,
    index: usize,
    id: std::sync::atomic::AtomicU8,
}

impl GrindstoneResultSlot {
    fn new(
        inventory: Arc<dyn Inventory>,
        repair_inventory: Arc<SimpleInventory>,
        index: usize,
    ) -> Self {
        Self {
            inventory,
            repair_inventory,
            index,
            id: std::sync::atomic::AtomicU8::new(0),
        }
    }
}

impl Slot for GrindstoneResultSlot {
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

    fn on_take_item<'a>(
        &'a self,
        player: &'a dyn InventoryPlayer,
        _stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let input = self.repair_inventory.get_stack(0).await;
            let input = input.lock().await.clone();
            let additional = self.repair_inventory.get_stack(1).await;
            let additional = additional.lock().await.clone();

            let amount = experience_amount(&input, &additional);
            if amount > 0 {
                player.award_experience(amount).await;
            }

            self.repair_inventory
                .set_stack(0, ItemStack::EMPTY.clone())
                .await;
            self.repair_inventory
                .set_stack(1, ItemStack::EMPTY.clone())
                .await;
            self.mark_dirty().await;
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_equipment_slots, entity_equipment::EntityEquipment};
    use pumpkin_data::item::Item;
    use tokio::sync::Mutex as TokioMutex;

    fn handler() -> GrindstoneScreenHandler {
        let player_inventory = Arc::new(PlayerInventory::new(
            Arc::new(TokioMutex::new(EntityEquipment::new())),
            Arc::new(build_equipment_slots()),
        ));
        GrindstoneScreenHandler::new(0, &player_inventory)
    }

    #[test]
    fn repair_cost_matches_vanilla_formula() {
        assert_eq!(calculate_increased_repair_cost(0), 1);
        assert_eq!(calculate_increased_repair_cost(1), 3);
        assert_eq!(calculate_increased_repair_cost(3), 7);
        // Saturates rather than overflowing i32.
        assert_eq!(calculate_increased_repair_cost(i32::MAX), i32::MAX);
        assert_eq!(calculate_increased_repair_cost(i32::MAX / 2), i32::MAX);
    }

    #[test]
    fn merging_two_damaged_swords_averages_durability_plus_bonus() {
        let mut a = ItemStack::new(1, &Item::IRON_PICKAXE);
        a.set_damage(100);
        let mut b = ItemStack::new(1, &Item::IRON_PICKAXE);
        b.set_damage(150);

        let result = merge_items(&a, &b);
        assert!(result.item == &Item::IRON_PICKAXE);
        // durability = max_damage(250); remaining = (250-100)+(250-150)+250*5/100 = 150+100+12 = 262
        // damage = max(250-262, 0) = 0
        assert_eq!(result.get_damage(), 0);
    }

    #[test]
    fn merging_mismatched_items_returns_empty() {
        let a = ItemStack::new(1, &Item::IRON_PICKAXE);
        let b = ItemStack::new(1, &Item::DIAMOND_PICKAXE);
        assert!(merge_items(&a, &b).is_empty());
    }

    #[test]
    fn non_damageable_stack_of_two_matching_books_merges() {
        let a = ItemStack::new(1, &Item::BOOK);
        let b = ItemStack::new(1, &Item::BOOK);
        let result = merge_items(&a, &b);
        assert!(result.item == &Item::BOOK);
        assert_eq!(result.item_count, 2);
    }

    #[test]
    fn non_damageable_stack_of_two_non_matching_items_is_empty() {
        let a = ItemStack::new(1, &Item::BOOK);
        let mut b = ItemStack::new(1, &Item::BOOK);
        b.set_custom_name("Different".to_string());
        assert!(merge_items(&a, &b).is_empty());
    }

    #[test]
    fn single_enchanted_item_strips_non_curses() {
        let mut item = ItemStack::new(1, &Item::IRON_PICKAXE);
        item.enchant(&Enchantment::EFFICIENCY, 3);
        item.enchant(&Enchantment::VANISHING_CURSE, 1);

        let result = compute_result(&item, &ItemStack::EMPTY.clone());
        assert_eq!(result.get_enchantment_level(&Enchantment::EFFICIENCY), 0);
        assert_eq!(
            result.get_enchantment_level(&Enchantment::VANISHING_CURSE),
            1
        );
    }

    #[test]
    fn plain_item_alone_yields_no_result() {
        let item = ItemStack::new(1, &Item::IRON_PICKAXE);
        let result = compute_result(&item, &ItemStack::EMPTY.clone());
        assert!(result.is_empty());
    }

    #[test]
    fn curse_transfers_onto_a_merge_target_that_lacks_it() {
        let mut a = ItemStack::new(1, &Item::IRON_PICKAXE);
        a.enchant(&Enchantment::EFFICIENCY, 2);
        let mut b = ItemStack::new(1, &Item::IRON_PICKAXE);
        b.enchant(&Enchantment::VANISHING_CURSE, 1);

        let result = merge_items(&a, &b);
        // `a` has no vanishing curse yet, so vanilla's `newEnchantments.getLevel(enchant) == 0`
        // check lets it transfer during the merge; `remove_non_curses_from` then strips the
        // non-curse Efficiency but keeps the curse.
        assert_eq!(
            result.get_enchantment_level(&Enchantment::VANISHING_CURSE),
            1
        );
        assert_eq!(result.get_enchantment_level(&Enchantment::EFFICIENCY), 0);
    }

    #[test]
    fn enchanted_book_with_no_remaining_enchantments_becomes_a_plain_book() {
        let mut book = ItemStack::new(1, &Item::ENCHANTED_BOOK);
        set_enchantments_for_crafting(&mut book, vec![(&Enchantment::EFFICIENCY, 3)]);

        let result = remove_non_curses_from(book);
        assert!(result.item == &Item::BOOK);
    }

    #[test]
    fn experience_from_item_sums_non_curse_min_costs() {
        let mut item = ItemStack::new(1, &Item::IRON_PICKAXE);
        item.enchant(&Enchantment::EFFICIENCY, 1);
        item.enchant(&Enchantment::VANISHING_CURSE, 1);

        let expected = Enchantment::EFFICIENCY.min_cost.calculate(1);
        assert_eq!(experience_from_item(&item), expected);
    }

    #[tokio::test]
    async fn placing_two_matching_pickaxes_refreshes_the_merged_result() {
        let handler = handler();
        handler
            .repair_inventory
            .set_stack(0, ItemStack::new(1, &Item::IRON_PICKAXE))
            .await;
        handler
            .repair_inventory
            .set_stack(1, ItemStack::new(1, &Item::IRON_PICKAXE))
            .await;
        handler.update_result().await;

        let result_slot = handler.get_behaviour().slots[2].clone();
        assert!(result_slot.has_stack().await);
        assert!(result_slot.get_cloned_stack().await.item == &Item::IRON_PICKAXE);
    }
}
