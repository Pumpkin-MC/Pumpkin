use crate::{
    events::{ToFromWasmEvent, consume_player, from_wasm_block_position, to_wasm_block_position},
    pumpkin::plugin::event::{
        BrewEvent as BrewEventData, BrewingStandFuelEvent as BrewingStandFuelEventData,
        CraftItemEvent as CraftItemEventData, Event, FurnaceBurnEvent as FurnaceBurnEventData,
        FurnaceExtractEvent as FurnaceExtractEventData, FurnaceSmeltEvent as FurnaceSmeltEventData,
        FurnaceStartSmeltEvent as FurnaceStartSmeltEventData,
        HopperInventorySearchEvent as HopperInventorySearchEventData,
        InventoryCreativeEvent as InventoryCreativeEventData,
        InventoryDragEvent as InventoryDragEventData,
        InventoryInteractEvent as InventoryInteractEventData,
        InventoryMoveItemEvent as InventoryMoveItemEventData,
        InventoryOpenEvent as InventoryOpenEventData,
        InventoryPickupItemEvent as InventoryPickupItemEventData,
        PrepareAnvilEvent as PrepareAnvilEventData,
        PrepareGrindstoneEvent as PrepareGrindstoneEventData,
        PrepareInventoryResultEvent as PrepareInventoryResultEventData,
        PrepareItemCraftEvent as PrepareItemCraftEventData,
        PrepareSmithingEvent as PrepareSmithingEventData, SmithItemEvent as SmithItemEventData,
        TradeSelectEvent as TradeSelectEventData,
    },
};
use pumpkin_core::plugin::inventory::{
    brew::BrewEvent, brewing_stand_fuel::BrewingStandFuelEvent, craft_item::CraftItemEvent,
    furnace_burn::FurnaceBurnEvent, furnace_extract::FurnaceExtractEvent,
    furnace_smelt::FurnaceSmeltEvent, furnace_start_smelt::FurnaceStartSmeltEvent,
    hopper_inventory_search::HopperInventorySearchEvent,
    inventory_creative::InventoryCreativeEvent, inventory_drag::InventoryDragEvent,
    inventory_interact::InventoryInteractEvent, inventory_move_item::InventoryMoveItemEvent,
    inventory_open::InventoryOpenEvent, inventory_pickup_item::InventoryPickupItemEvent,
    prepare_anvil::PrepareAnvilEvent, prepare_grindstone::PrepareGrindstoneEvent,
    prepare_inventory_result::PrepareInventoryResultEvent,
    prepare_item_craft::PrepareItemCraftEvent, prepare_smithing::PrepareSmithingEvent,
    smith_item::SmithItemEvent, trade_select::TradeSelectEvent,
};
use pumpkin_wasm_host_common::state::PluginHostState;

impl ToFromWasmEvent for InventoryOpenEvent {
    type Wit = InventoryOpenEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for InventoryDragEvent {
    type Wit = InventoryDragEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for CraftItemEvent {
    type Wit = CraftItemEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            recipe_id: self.recipe_id.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            recipe_id: data.recipe_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for FurnaceSmeltEvent {
    type Wit = FurnaceSmeltEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            source_item: self.source_item.clone(),
            result_item: self.result_item.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.source_item = data.source_item;
        self.result_item = data.result_item;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            source_item: data.source_item,
            result_item: data.result_item,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BrewEvent {
    type Wit = BrewEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            fuel_level: self.fuel_level,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.fuel_level = data.fuel_level;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            fuel_level: data.fuel_level,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BrewingStandFuelEvent {
    type Wit = BrewingStandFuelEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            fuel_power: self.fuel_power,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.fuel_power = data.fuel_power;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            fuel_power: data.fuel_power,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for FurnaceBurnEvent {
    type Wit = FurnaceBurnEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            fuel_item: self.fuel_item.clone(),
            burn_time: self.burn_time,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.fuel_item = data.fuel_item;
        self.burn_time = data.burn_time;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            fuel_item: data.fuel_item,
            burn_time: data.burn_time,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for FurnaceExtractEvent {
    type Wit = FurnaceExtractEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            item_id: self.item_id.clone(),
            item_amount: self.item_amount,
            exp_gained: self.exp_gained,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            block_pos: from_wasm_block_position(data.block_pos),
            item_id: data.item_id,
            item_amount: data.item_amount,
            exp_gained: data.exp_gained,
        }
    }
}

impl ToFromWasmEvent for FurnaceStartSmeltEvent {
    type Wit = FurnaceStartSmeltEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            source_item: self.source_item.clone(),
            cooking_time: self.cooking_time,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.source_item = data.source_item;
        self.cooking_time = data.cooking_time;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            source_item: data.source_item,
            cooking_time: data.cooking_time,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for HopperInventorySearchEvent {
    type Wit = HopperInventorySearchEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            search_pos: to_wasm_block_position(self.search_pos),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            search_pos: from_wasm_block_position(data.search_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for InventoryCreativeEvent {
    type Wit = InventoryCreativeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            slot: self.slot,
            item_id: self.item_id.clone(),
            item_count: self.item_count,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.slot = data.slot;
        self.item_id = data.item_id;
        self.item_count = data.item_count;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            slot: data.slot,
            item_id: data.item_id,
            item_count: data.item_count,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for InventoryInteractEvent {
    type Wit = InventoryInteractEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for InventoryMoveItemEvent {
    type Wit = InventoryMoveItemEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            source_pos: to_wasm_block_position(self.source_pos),
            target_pos: to_wasm_block_position(self.target_pos),
            item_id: self.item_id.clone(),
            item_amount: self.item_amount,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.item_amount = data.item_amount;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            source_pos: from_wasm_block_position(data.source_pos),
            target_pos: from_wasm_block_position(data.target_pos),
            item_id: data.item_id,
            item_amount: data.item_amount,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for InventoryPickupItemEvent {
    type Wit = InventoryPickupItemEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            item_entity_id: self.item_entity_id,
            item_id: self.item_id.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            item_entity_id: data.item_entity_id,
            item_id: data.item_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PrepareAnvilEvent {
    type Wit = PrepareAnvilEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            rename_text: self.rename_text.clone(),
            repair_cost: self.repair_cost,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            rename_text: data.rename_text,
            repair_cost: data.repair_cost,
        }
    }
}

impl ToFromWasmEvent for PrepareGrindstoneEvent {
    type Wit = PrepareGrindstoneEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            result_item: self.result_item.clone(),
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            result_item: data.result_item,
        }
    }
}

impl ToFromWasmEvent for PrepareInventoryResultEvent {
    type Wit = PrepareInventoryResultEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            result_item: self.result_item.clone(),
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            result_item: data.result_item,
        }
    }
}

impl ToFromWasmEvent for PrepareItemCraftEvent {
    type Wit = PrepareItemCraftEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            recipe_id: self.recipe_id.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            recipe_id: data.recipe_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PrepareSmithingEvent {
    type Wit = PrepareSmithingEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            result_item: self.result_item.clone(),
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            result_item: data.result_item,
        }
    }
}

impl ToFromWasmEvent for SmithItemEvent {
    type Wit = SmithItemEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            recipe_id: self.recipe_id.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            recipe_id: data.recipe_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for TradeSelectEvent {
    type Wit = TradeSelectEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            slot_index: self.slot_index,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.slot_index = data.slot_index;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            slot_index: data.slot_index,
            cancelled: data.cancelled,
        }
    }
}
