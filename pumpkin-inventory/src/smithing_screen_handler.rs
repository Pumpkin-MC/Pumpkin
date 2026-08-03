use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use tokio::sync::Mutex;

use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
    offer_or_drop_stack,
};
use crate::slot::{BoxFuture, Slot};

use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, TrimImpl};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::screen::WindowType;
use pumpkin_data::smithing;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_protocol::java::server::play::SlotActionType;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::inventory::SimpleInventory;

pub const TEMPLATE_SLOT: usize = 0;
pub const BASE_SLOT: usize = 1;
pub const ADDITION_SLOT: usize = 2;

pub struct SmithingScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    pub input_inventory: Arc<SimpleInventory>,
    pub output_inventory: Arc<SimpleInventory>,
}

impl SmithingScreenHandler {
    pub fn new(sync_id: u8, player_inventory: &Arc<PlayerInventory>) -> Self {
        let behaviour = ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Smithing));
        let input_inventory = Arc::new(SimpleInventory::new(3));
        let output_inventory = Arc::new(SimpleInventory::new(1));

        let mut handler = Self {
            behaviour,
            input_inventory: input_inventory.clone(),
            output_inventory: output_inventory.clone(),
        };

        handler.add_slot(Arc::new(SmithingInputSlot::new(
            input_inventory.clone() as Arc<dyn Inventory>,
            TEMPLATE_SLOT,
            is_template_candidate,
        )));
        handler.add_slot(Arc::new(SmithingInputSlot::new(
            input_inventory.clone() as Arc<dyn Inventory>,
            BASE_SLOT,
            is_base_candidate,
        )));
        handler.add_slot(Arc::new(SmithingInputSlot::new(
            input_inventory.clone() as Arc<dyn Inventory>,
            ADDITION_SLOT,
            is_addition_candidate,
        )));
        handler.add_slot(Arc::new(SmithingOutputSlot::new(
            output_inventory as Arc<dyn Inventory>,
            input_inventory as Arc<dyn Inventory>,
            0,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    // net/minecraft/world/item/crafting/SmithingRecipe#matches +
    // SmithingTransformRecipe/SmithingTrimRecipe#assemble.
    fn compute_result(template: &ItemStack, base: &ItemStack, addition: &ItemStack) -> ItemStack {
        if base.is_empty() {
            return ItemStack::EMPTY.clone();
        }

        if smithing::is_template_item(template.item)
            && smithing::is_transform_addition(addition.item)
            && let Some(recipe) = smithing::find_transform_recipe(base.item)
        {
            let mut result = base.copy_with_count(1);
            result.item = recipe.result;
            return result;
        }

        if let Some(pattern) = smithing::trim_pattern_for_template(template.item)
            && smithing::is_trimmable_base(base.item)
            && let Some(material) = smithing::trim_material_for_addition(addition.item)
        {
            let new_trim = TrimImpl { material, pattern };
            if base.get_data_component::<TrimImpl>() == Some(&new_trim) {
                return ItemStack::EMPTY.clone();
            }

            let mut result = base.copy_with_count(1);
            if let Some(pos) = result
                .patch
                .iter()
                .position(|(id, _)| *id == DataComponent::Trim)
            {
                result.patch[pos].1 = Some(new_trim.to_dyn());
            } else {
                result
                    .patch
                    .push((DataComponent::Trim, Some(new_trim.to_dyn())));
            }
            return result;
        }

        ItemStack::EMPTY.clone()
    }

    async fn update_output(&self) {
        let template = self.input_inventory.get_stack(TEMPLATE_SLOT).await;
        let base = self.input_inventory.get_stack(BASE_SLOT).await;
        let addition = self.input_inventory.get_stack(ADDITION_SLOT).await;

        let template = template.lock().await;
        let base = base.lock().await;
        let addition = addition.lock().await;

        let result = Self::compute_result(&template, &base, &addition);
        self.output_inventory.set_stack(0, result).await;
    }
}

impl ScreenHandler for SmithingScreenHandler {
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

    // net/minecraft/world/inventory/ItemCombinerMenu#removed: input slots are returned to the
    // player when the menu closes (the result slot is never populated with a real item to drop).
    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            for index in [TEMPLATE_SLOT, BASE_SLOT, ADDITION_SLOT] {
                let stack = self.input_inventory.remove_stack(index).await;
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
            if (0..4).contains(&slot_index) {
                self.update_output().await;
            }
        })
    }

    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ScreenHandlerFuture<'a, ItemStack> {
        Box::pin(async move {
            let mut stack = ItemStack::EMPTY.clone();
            let slot = self.get_behaviour().slots.get(slot_index as usize).cloned();

            if let Some(slot) = slot {
                let mut slot_stack = slot.get_cloned_stack().await;
                if !slot_stack.is_empty() {
                    stack = slot_stack.clone();
                    if slot_index < 4 {
                        if !self.insert_item(&mut slot_stack, 4, 40, true).await {
                            return ItemStack::EMPTY.clone();
                        }
                        slot.on_quick_move_crafted(slot_stack.clone(), stack.clone())
                            .await;
                    } else if !self.insert_item(&mut slot_stack, 0, 3, false).await {
                        return ItemStack::EMPTY.clone();
                    }

                    let moved_count = stack.item_count - slot_stack.item_count;
                    if slot_stack.is_empty() {
                        slot.set_stack(ItemStack::EMPTY.clone()).await;
                    } else {
                        slot.set_stack(slot_stack).await;
                    }

                    if slot_index == 3 {
                        let mut taken_stack = stack.clone();
                        taken_stack.set_count(moved_count);
                        slot.on_take_item(player, &taken_stack).await;
                    }

                    self.update_output().await;
                }
            }
            stack
        })
    }
}

// net/minecraft/world/inventory/SmithingMenu#createInputSlotDefinitions: each input slot only
// accepts items that appear as that role's ingredient in at least one smithing recipe.
fn is_template_candidate(item: &pumpkin_data::item::Item) -> bool {
    smithing::is_template_item(item) || smithing::trim_pattern_for_template(item).is_some()
}

fn is_base_candidate(item: &pumpkin_data::item::Item) -> bool {
    smithing::find_transform_recipe(item).is_some() || smithing::is_trimmable_base(item)
}

fn is_addition_candidate(item: &pumpkin_data::item::Item) -> bool {
    smithing::is_transform_addition(item) || smithing::is_trim_addition(item)
}

pub struct SmithingInputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
    pub predicate: fn(&pumpkin_data::item::Item) -> bool,
}

impl SmithingInputSlot {
    pub fn new(
        inventory: Arc<dyn Inventory>,
        index: usize,
        predicate: fn(&pumpkin_data::item::Item) -> bool,
    ) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
            predicate,
        }
    }
}

impl Slot for SmithingInputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn can_insert(&self, stack: &ItemStack) -> BoxFuture<'_, bool> {
        let matches = (self.predicate)(stack.item);
        Box::pin(async move { matches })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}

pub struct SmithingOutputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub input_inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
}

impl SmithingOutputSlot {
    pub fn new(
        inventory: Arc<dyn Inventory>,
        input_inventory: Arc<dyn Inventory>,
        index: usize,
    ) -> Self {
        Self {
            inventory,
            input_inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for SmithingOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    // net/minecraft/world/inventory/SmithingMenu#onTake.
    fn on_take_item<'a>(
        &'a self,
        player: &'a dyn InventoryPlayer,
        stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            player
                .increment_stat(
                    StatisticCategory::Crafted,
                    stack.item.id as i32,
                    stack.item_count as i32,
                )
                .await;

            for index in [TEMPLATE_SLOT, BASE_SLOT, ADDITION_SLOT] {
                let input_stack = self.input_inventory.get_stack(index).await;
                let mut input_lock = input_stack.lock().await;
                if !input_lock.is_empty() {
                    input_lock.item_count -= 1;
                    if input_lock.item_count == 0 {
                        *input_lock = ItemStack::EMPTY.clone();
                    }
                }
            }
            self.mark_dirty().await;
        })
    }

    fn can_insert(&self, _stack: &ItemStack) -> BoxFuture<'_, bool> {
        Box::pin(async move { false })
    }

    fn get_stack(&self) -> BoxFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.inventory.get_stack(self.index).await })
    }

    fn get_cloned_stack(&self) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move {
            let stack = self.inventory.get_stack(self.index).await;
            stack.lock().await.clone()
        })
    }

    fn has_stack(&self) -> BoxFuture<'_, bool> {
        Box::pin(async move {
            let stack = self.inventory.get_stack(self.index).await;
            !stack.lock().await.is_empty()
        })
    }

    fn set_stack(&self, stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.set_stack(self.index, stack).await;
        })
    }

    fn set_stack_prev(&self, _stack: ItemStack, _previous_stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {})
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
    use pumpkin_data::trim::{TrimMaterial, TrimPattern};

    fn handler() -> SmithingScreenHandler {
        let player_inventory = Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(build_equipment_slots()),
        ));
        SmithingScreenHandler::new(0, &player_inventory)
    }

    #[tokio::test]
    async fn netherite_upgrade_produces_netherite_item_and_consumes_inputs() {
        let handler = handler();
        handler
            .input_inventory
            .set_stack(
                TEMPLATE_SLOT,
                ItemStack::new(1, &Item::NETHERITE_UPGRADE_SMITHING_TEMPLATE),
            )
            .await;
        handler
            .input_inventory
            .set_stack(BASE_SLOT, ItemStack::new(1, &Item::DIAMOND_SWORD))
            .await;
        handler
            .input_inventory
            .set_stack(ADDITION_SLOT, ItemStack::new(1, &Item::NETHERITE_INGOT))
            .await;
        handler.update_output().await;

        let output = handler.output_inventory.get_stack(0).await;
        let output = output.lock().await;
        assert!(output.item == &Item::NETHERITE_SWORD);
        assert_eq!(output.item_count, 1);
    }

    #[tokio::test]
    async fn armor_trim_sets_trim_component() {
        let handler = handler();
        handler
            .input_inventory
            .set_stack(
                TEMPLATE_SLOT,
                ItemStack::new(1, &Item::SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE),
            )
            .await;
        handler
            .input_inventory
            .set_stack(BASE_SLOT, ItemStack::new(1, &Item::DIAMOND_CHESTPLATE))
            .await;
        handler
            .input_inventory
            .set_stack(ADDITION_SLOT, ItemStack::new(1, &Item::QUARTZ))
            .await;
        handler.update_output().await;

        let output = handler.output_inventory.get_stack(0).await;
        let output = output.lock().await;
        assert!(output.item == &Item::DIAMOND_CHESTPLATE);
        let trim = output.get_data_component::<TrimImpl>().unwrap();
        assert_eq!(trim.material, TrimMaterial::Quartz);
        assert_eq!(trim.pattern, TrimPattern::Sentry);
    }

    #[tokio::test]
    async fn reapplying_identical_trim_yields_no_result() {
        let handler = handler();
        let mut trimmed = ItemStack::new(1, &Item::DIAMOND_CHESTPLATE);
        trimmed.patch.push((
            DataComponent::Trim,
            Some(
                TrimImpl {
                    material: TrimMaterial::Quartz,
                    pattern: TrimPattern::Sentry,
                }
                .to_dyn(),
            ),
        ));
        handler
            .input_inventory
            .set_stack(
                TEMPLATE_SLOT,
                ItemStack::new(1, &Item::SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE),
            )
            .await;
        handler.input_inventory.set_stack(BASE_SLOT, trimmed).await;
        handler
            .input_inventory
            .set_stack(ADDITION_SLOT, ItemStack::new(1, &Item::QUARTZ))
            .await;
        handler.update_output().await;

        let output = handler.output_inventory.get_stack(0).await;
        assert!(output.lock().await.is_empty());
    }

    #[tokio::test]
    async fn mismatched_ingredients_produce_no_result() {
        let handler = handler();
        handler
            .input_inventory
            .set_stack(BASE_SLOT, ItemStack::new(1, &Item::DIAMOND_SWORD))
            .await;
        handler.update_output().await;

        let output = handler.output_inventory.get_stack(0).await;
        assert!(output.lock().await.is_empty());
    }

    #[tokio::test]
    async fn output_repopulates_after_taking_with_stacked_inputs() {
        let handler = handler();
        handler
            .input_inventory
            .set_stack(
                TEMPLATE_SLOT,
                ItemStack::new(2, &Item::NETHERITE_UPGRADE_SMITHING_TEMPLATE),
            )
            .await;
        handler
            .input_inventory
            .set_stack(BASE_SLOT, ItemStack::new(1, &Item::DIAMOND_SWORD))
            .await;
        handler
            .input_inventory
            .set_stack(ADDITION_SLOT, ItemStack::new(2, &Item::NETHERITE_INGOT))
            .await;
        handler.update_output().await;

        let output = handler.output_inventory.get_stack(0).await;
        assert!(output.lock().await.item == &Item::NETHERITE_SWORD);

        // Simulate SmithingOutputSlot::on_take_item shrinking each input by 1.
        for index in [TEMPLATE_SLOT, BASE_SLOT, ADDITION_SLOT] {
            let stack = handler.input_inventory.get_stack(index).await;
            let mut stack = stack.lock().await;
            stack.item_count -= 1;
            if stack.item_count == 0 {
                *stack = ItemStack::EMPTY.clone();
            }
        }
        handler.update_output().await;

        let output = handler.output_inventory.get_stack(0).await;
        assert!(
            output.lock().await.item == &Item::NETHERITE_SWORD,
            "output should repopulate from the remaining stacked template/addition after a craft"
        );
    }

    #[tokio::test]
    async fn input_slots_reject_items_outside_their_role() {
        let inventory: Arc<dyn Inventory> = Arc::new(SimpleInventory::new(3));
        let template_slot =
            SmithingInputSlot::new(inventory.clone(), TEMPLATE_SLOT, is_template_candidate);
        let base_slot = SmithingInputSlot::new(inventory.clone(), BASE_SLOT, is_base_candidate);
        let addition_slot = SmithingInputSlot::new(inventory, ADDITION_SLOT, is_addition_candidate);

        assert!(
            !template_slot
                .can_insert(&ItemStack::new(1, &Item::DIRT))
                .await
        );
        assert!(
            template_slot
                .can_insert(&ItemStack::new(
                    1,
                    &Item::SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE
                ))
                .await
        );

        assert!(!base_slot.can_insert(&ItemStack::new(1, &Item::DIRT)).await);
        assert!(
            base_slot
                .can_insert(&ItemStack::new(1, &Item::DIAMOND_CHESTPLATE))
                .await
        );

        assert!(
            !addition_slot
                .can_insert(&ItemStack::new(1, &Item::DIRT))
                .await
        );
        assert!(
            addition_slot
                .can_insert(&ItemStack::new(1, &Item::QUARTZ))
                .await
        );
    }
}
