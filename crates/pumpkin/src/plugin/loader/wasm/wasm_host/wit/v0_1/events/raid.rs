use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{ToFromWasmEvent, from_wasm_block_position, to_wasm_block_position},
            pumpkin::plugin::event::{
                Event, RaidFinishEventData, RaidSpawnWaveEventData, RaidStopEventData,
                RaidTriggerEventData,
            },
        },
    },
    raid::{
        raid_finish::RaidFinishEvent, raid_spawn_wave::RaidSpawnWaveEvent,
        raid_stop::RaidStopEvent, raid_trigger::RaidTriggerEvent,
    },
};

impl ToFromWasmEvent for RaidFinishEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::RaidFinishEvent(RaidFinishEventData {
            victory: self.victory,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::RaidFinishEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::RaidFinishEvent(data) => Self {
                victory: data.victory,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for RaidSpawnWaveEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::RaidSpawnWaveEvent(RaidSpawnWaveEventData {
            wave: self.wave,
            pos: to_wasm_block_position(self.pos),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::RaidSpawnWaveEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::RaidSpawnWaveEvent(data) => Self {
                wave: data.wave,
                pos: from_wasm_block_position(data.pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for RaidStopEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::RaidStopEvent(RaidStopEventData {
            reason: self.reason.clone(),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::RaidStopEvent(data) = event {
            self.cancelled = data.cancelled;
            self.reason = data.reason;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::RaidStopEvent(data) => Self {
                reason: data.reason,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for RaidTriggerEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::RaidTriggerEvent(RaidTriggerEventData {
            pos: to_wasm_block_position(self.pos),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::RaidTriggerEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::RaidTriggerEvent(data) => Self {
                pos: from_wasm_block_position(data.pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
