use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicI16, Ordering};

use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use pumpkin_world::inventory::Inventory;

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture, offer_or_drop_stack,
    },
    slot::{BoxFuture, NormalSlot, TakeOnlySlot, TakeSlotCharge},
    window_property::{Anvil, WindowProperty},
};

pub struct AnvilScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    behaviour: ScreenHandlerBehaviour,
    pub rename_text: String,
    /// Repair cost in XP levels, shared with the output slot's charge hook.
    pub repair_cost: Arc<AtomicI16>,
}

/// Charges XP levels and the base item on the anvil output slot.
///
/// Fires from the slot's take path, so every verified take — pickup,
/// shift-click, throw, swap — is charged exactly once; there is no
/// per-action gate that could forget one.
struct AnvilSlotCharge {
    inventory: Arc<dyn Inventory>,
    repair_cost: Arc<AtomicI16>,
}

impl TakeSlotCharge for AnvilSlotCharge {
    fn can_take(&self, player: &dyn InventoryPlayer) -> bool {
        player.is_creative()
            || player.experience_level() >= i32::from(self.repair_cost.load(Ordering::Relaxed))
    }

    fn on_take<'a>(
        &'a self,
        player: &'a dyn InventoryPlayer,
        _stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if !player.is_creative() {
                player
                    .add_experience_levels(-i32::from(self.repair_cost.load(Ordering::Relaxed)))
                    .await;
            }

            // Rename-only path consumes the base item in slot 0.
            self.inventory.set_stack(0, ItemStack::EMPTY.clone()).await;
            self.inventory.mark_dirty();
        })
    }
}

impl AnvilScreenHandler {
    #[expect(clippy::needless_pass_by_value)]
    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
    ) -> Self {
        let repair_cost = Arc::new(AtomicI16::new(0));
        let mut handler = Self {
            inventory: inventory.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Anvil)),
            rename_text: String::new(),
            repair_cost: repair_cost.clone(),
        };

        // Anvil: 2 input slots + take-only output (PickupAll must not sweep output).
        // The output slot owns the charge hook so every take path charges once.
        handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 0)));
        handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 1)));
        handler.add_slot(Arc::new(TakeOnlySlot::with_charge(
            inventory.clone(),
            2,
            Arc::new(AnvilSlotCharge {
                inventory: inventory.clone(),
                repair_cost,
            }),
        )));
        // Container slots precede the player slots; used by Bedrock slot mapping.
        handler.behaviour.container_slots = handler.behaviour.slots.len();

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    pub async fn update_item_name(&mut self, name: String) {
        self.rename_text = name;
        self.update_result_slot().await;
        self.send_content_updates().await;
    }

    pub async fn update_result_slot(&mut self) {
        let input_a = {
            let lock = self.inventory.get_stack(0).await;
            lock.lock().await.clone()
        };

        if input_a.is_empty() {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
            self.set_repair_cost(0).await;
            return;
        }

        let mut result_item = input_a.clone();
        let mut cost = 0;

        // Basic renaming logic for now
        if !self.rename_text.is_empty() {
            result_item.set_custom_name(self.rename_text.clone());
            cost += 1;
        }

        // If combining with another item... we'll skip complex anvil logic for now
        // and just support renaming.
        if cost > 0 {
            self.inventory.set_stack(2, result_item).await;
            self.set_repair_cost(cost).await;
        } else {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone()).await;
            self.set_repair_cost(0).await;
        }
    }

    pub async fn set_repair_cost(&mut self, cost: i16) {
        self.repair_cost.store(cost, Ordering::Relaxed);
        if let Some(sync_handler) = self.behaviour.sync_handler.as_ref() {
            let (property_id, property_value) =
                WindowProperty::new(Anvil::RepairCost, cost).into_tuple();
            sync_handler
                .update_property(&self.behaviour, property_id as i32, property_value as i32)
                .await;
        }
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
            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if slot.has_stack().await {
                let slot_stack_lock = slot.get_stack().await;
                let mut slot_stack_mut = slot_stack_lock.lock().await;
                let stack_prev = slot_stack_mut.clone();

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

                let stack = slot_stack_mut.clone();
                drop(slot_stack_mut);
                if stack.is_empty() {
                    slot.set_stack(ItemStack::EMPTY.clone()).await;
                } else {
                    slot.mark_dirty().await;
                }
                if stack.item_count == stack_prev.item_count {
                    return ItemStack::EMPTY.clone();
                }

                // Charge only what was actually moved; a failed delivery above
                // returns before this point and charges nothing.
                let mut taken_stack = stack_prev.clone();
                taken_stack.set_count(stack_prev.item_count - stack.item_count);
                slot.on_take_item(player, &taken_stack).await;

                return stack_prev;
            }

            ItemStack::EMPTY.clone()
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
            self.internal_on_slot_click(slot_index, button, action_type, player)
                .await;
            if slot_index == 0 || slot_index == 1 || slot_index == 2 {
                self.update_result_slot().await;
                self.send_content_updates().await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pumpkin_data::item::Item;
    use pumpkin_protocol::java::server::play::SlotActionType;
    use pumpkin_world::inventory::SimpleInventory;

    use crate::screen_handler::ScreenHandler;
    use crate::test_util::MockPlayer;

    async fn setup() -> (AnvilScreenHandler, Arc<MockPlayer>) {
        let player = Arc::new(MockPlayer::new());
        player.xp_levels.store(5, Ordering::Relaxed);
        let inventory: Arc<dyn Inventory> = Arc::new(SimpleInventory::new(3));
        inventory
            .set_stack(0, ItemStack::new(1, &Item::IRON_SWORD))
            .await;
        let mut handler = AnvilScreenHandler::new(1, &player.player_inventory, inventory);
        handler.update_item_name("Pointy".to_string()).await;
        assert_eq!(handler.get_behaviour().container_slots, 3);
        assert_eq!(handler.repair_cost.load(Ordering::Relaxed), 1);
        (handler, player)
    }

    async fn inv_stack(inventory: &Arc<dyn Inventory>, index: usize) -> ItemStack {
        inventory.get_stack(index).await.lock().await.clone()
    }

    fn xp_levels(player: &MockPlayer) -> i32 {
        player.xp_levels.load(Ordering::Relaxed)
    }

    #[tokio::test]
    async fn throw_on_output_charges_exactly_once() {
        let (mut handler, player) = setup().await;
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 1);

        // Q on the result slot: one take, one charge. A double-fired take
        // hook would show up as two levels charged for one delivered item.
        handler
            .on_slot_click(2, 0, SlotActionType::Throw, player.as_ref())
            .await;
        assert_eq!(xp_levels(&player), 4);
        assert!(inv_stack(&handler.inventory, 0).await.is_empty());
        let dropped = player.dropped.lock().await;
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].item.id, Item::IRON_SWORD.id);
        drop(dropped);
        // Input consumed -> result cleared, nothing more to take.
        assert!(inv_stack(&handler.inventory, 2).await.is_empty());

        handler
            .on_slot_click(2, 0, SlotActionType::Throw, player.as_ref())
            .await;
        assert_eq!(xp_levels(&player), 4, "no result, no second charge");
        assert_eq!(player.dropped.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn swap_on_output_charges_once() {
        let (mut handler, player) = setup().await;

        handler
            .on_slot_click(2, 0, SlotActionType::Swap, player.as_ref())
            .await;
        assert_eq!(xp_levels(&player), 4);
        assert!(inv_stack(&handler.inventory, 0).await.is_empty());
        let hotbar = player.player_inventory.get_stack(0).await;
        let hotbar = hotbar.lock().await;
        assert_eq!(hotbar.item.id, Item::IRON_SWORD.id);
    }

    #[tokio::test]
    async fn pickup_all_on_output_is_noop() {
        let (mut handler, player) = setup().await;
        let output = inv_stack(&handler.inventory, 2).await;
        *handler.get_behaviour_mut().cursor_stack.lock().await = output;

        // Double-click sweep must skip the take-only output slot entirely.
        handler
            .on_slot_click(2, 0, SlotActionType::PickupAll, player.as_ref())
            .await;
        assert_eq!(xp_levels(&player), 5, "PickupAll must not charge");
        assert!(!inv_stack(&handler.inventory, 0).await.is_empty());
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 1);
        assert_eq!(
            handler.get_behaviour().cursor_stack.lock().await.item_count,
            1
        );
    }

    #[tokio::test]
    async fn quick_move_into_full_inventory_charges_nothing() {
        let (mut handler, player) = setup().await;
        for i in 0..36 {
            player
                .player_inventory
                .set_stack(i, ItemStack::new(64, &Item::COBBLESTONE))
                .await;
        }

        handler
            .on_slot_click(2, 0, SlotActionType::QuickMove, player.as_ref())
            .await;
        assert_eq!(xp_levels(&player), 5, "failed delivery must not charge");
        assert!(!inv_stack(&handler.inventory, 0).await.is_empty());
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 1);
    }

    #[tokio::test]
    async fn insufficient_xp_blocks_take() {
        let (mut handler, player) = setup().await;
        player.xp_levels.store(0, Ordering::Relaxed);

        handler
            .on_slot_click(2, 0, SlotActionType::Pickup, player.as_ref())
            .await;
        assert_eq!(xp_levels(&player), 0);
        assert!(!inv_stack(&handler.inventory, 0).await.is_empty());
        assert_eq!(inv_stack(&handler.inventory, 2).await.item_count, 1);
        assert!(handler.get_behaviour().cursor_stack.lock().await.is_empty());
    }

    #[tokio::test]
    async fn pickup_on_output_charges_once() {
        let (mut handler, player) = setup().await;

        handler
            .on_slot_click(2, 0, SlotActionType::Pickup, player.as_ref())
            .await;
        assert_eq!(xp_levels(&player), 4);
        assert!(inv_stack(&handler.inventory, 0).await.is_empty());
        let cursor = handler.get_behaviour().cursor_stack.lock().await;
        assert_eq!(cursor.item.id, Item::IRON_SWORD.id);
        drop(cursor);
        assert!(inv_stack(&handler.inventory, 2).await.is_empty());
    }

    #[tokio::test]
    async fn quick_move_on_output_charges_once() {
        let (mut handler, player) = setup().await;

        handler
            .on_slot_click(2, 0, SlotActionType::QuickMove, player.as_ref())
            .await;
        assert_eq!(xp_levels(&player), 4);
        assert!(inv_stack(&handler.inventory, 0).await.is_empty());
        let mut moved = 0;
        for i in 0..36 {
            let stack = player.player_inventory.get_stack(i).await;
            let stack = stack.lock().await;
            if stack.item.id == Item::IRON_SWORD.id {
                moved += stack.item_count;
            }
        }
        assert_eq!(moved, 1, "the result must reach the player");
    }
}
