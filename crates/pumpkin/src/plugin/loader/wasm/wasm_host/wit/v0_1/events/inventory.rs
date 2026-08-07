use crate::plugin::{
    inventory::{
        craft_item::CraftItemEvent, furnace_smelt::FurnaceSmeltEvent,
        inventory_drag::InventoryDragEvent, inventory_open::InventoryOpenEvent,
    },
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{
                ToFromWasmEvent, consume_player, from_wasm_block_position, to_wasm_block_position,
            },
            pumpkin::plugin::event::{
                CraftItemEventData, Event, FurnaceSmeltEventData, InventoryDragEventData,
                InventoryOpenEventData,
            },
        },
    },
};

impl ToFromWasmEvent for InventoryOpenEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::InventoryOpenEvent(InventoryOpenEventData {
            player,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::InventoryOpenEvent(data) => Self {
                player: consume_player(state, &data.player),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for InventoryDragEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::InventoryDragEvent(InventoryDragEventData {
            player,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::InventoryDragEvent(data) => Self {
                player: consume_player(state, &data.player),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for CraftItemEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::CraftItemEvent(CraftItemEventData {
            player,
            recipe_id: self.recipe_id.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::CraftItemEvent(data) => Self {
                player: consume_player(state, &data.player),
                recipe_id: data.recipe_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for FurnaceSmeltEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::FurnaceSmeltEvent(FurnaceSmeltEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            source_item: self.source_item.clone(),
            result_item: self.result_item.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::FurnaceSmeltEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                source_item: data.source_item,
                result_item: data.result_item,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
