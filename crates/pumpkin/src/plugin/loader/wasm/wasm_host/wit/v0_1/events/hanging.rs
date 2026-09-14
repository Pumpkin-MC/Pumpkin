use crate::plugin::{
    hanging::{
        hanging_break::HangingBreakEvent, hanging_break_by_entity::HangingBreakByEntityEvent,
        hanging_place::HangingPlaceEvent,
    },
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{ToFromWasmEvent, cleanup_event, to_wasm_block_position},
            pumpkin::plugin::event::{
                Event, HangingBreakByEntityEventData, HangingBreakEventData, HangingPlaceEventData,
            },
        },
    },
};

impl ToFromWasmEvent for HangingBreakEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::HangingBreakEvent(HangingBreakEventData {
            entity_id: self.entity.get_entity().entity_id,
            remover_entity_id: self.remover.as_ref().map(|r| r.get_entity().entity_id),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::HangingBreakEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::HangingBreakEvent(_) => {
                panic!("Cannot construct HangingBreakEvent from WASM event data alone");
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for HangingBreakByEntityEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::HangingBreakByEntityEvent(HangingBreakByEntityEventData {
            entity_id: self.entity.get_entity().entity_id,
            remover_entity_id: self.remover.get_entity().entity_id,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::HangingBreakByEntityEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::HangingBreakByEntityEvent(_) => {
                panic!("Cannot construct HangingBreakByEntityEvent from WASM event data alone");
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for HangingPlaceEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = self.player.as_ref().map(|p| {
            state
                .add_player(p.clone())
                .expect("failed to add player resource")
        });

        Event::HangingPlaceEvent(HangingPlaceEventData {
            entity_id: self.entity.get_entity().entity_id,
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            block_face: format!("{:?}", self.block_face),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::HangingPlaceEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::HangingPlaceEvent(_) => {
                panic!("Cannot construct HangingPlaceEvent from WASM event data alone");
            }
            _ => panic!("unexpected event type"),
        }
    }
}
