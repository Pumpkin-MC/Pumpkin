use crate::{
    events::{ToFromWasmEvent, cleanup_event, to_wasm_block_position},
    pumpkin::plugin::event::{
        Event, HangingBreakByEntityEvent as HangingBreakByEntityEventData,
        HangingBreakEvent as HangingBreakEventData, HangingPlaceEvent as HangingPlaceEventData,
    },
};
use pumpkin_core::plugin::hanging::{
    hanging_break::HangingBreakEvent, hanging_break_by_entity::HangingBreakByEntityEvent,
    hanging_place::HangingPlaceEvent,
};
use pumpkin_wasm_host_common::state::PluginHostState;
impl ToFromWasmEvent for HangingBreakEvent {
    type Wit = HangingBreakEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity.get_entity().entity_id,
            remover_entity_id: self.remover.as_ref().map(|r| r.get_entity().entity_id),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct HangingBreakEvent from WASM event data alone");
    }
}

impl ToFromWasmEvent for HangingBreakByEntityEvent {
    type Wit = HangingBreakByEntityEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity.get_entity().entity_id,
            remover_entity_id: self.remover.get_entity().entity_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct HangingBreakByEntityEvent from WASM event data alone");
    }
}

impl ToFromWasmEvent for HangingPlaceEvent {
    type Wit = HangingPlaceEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = self
            .player
            .as_ref()
            .map(|p| state.add(p.clone()).expect("failed to add player resource"));

        Self::Wit {
            entity_id: self.entity.get_entity().entity_id,
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            block_face: format!("{:?}", self.block_face),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct HangingPlaceEvent from WASM event data alone");
    }
}
