use crate::{
    events::{ToFromWasmEvent, cleanup_event, from_wasm_block_position, to_wasm_block_position},
    pumpkin::plugin::event::{
        Event, RaidFinishEvent as RaidFinishEventData,
        RaidSpawnWaveEvent as RaidSpawnWaveEventData, RaidStopEvent as RaidStopEventData,
        RaidTriggerEvent as RaidTriggerEventData,
    },
};
use pumpkin_core::plugin::raid::{
    raid_finish::RaidFinishEvent, raid_spawn_wave::RaidSpawnWaveEvent, raid_stop::RaidStopEvent,
    raid_trigger::RaidTriggerEvent,
};
use pumpkin_wasm_host_common::state::PluginHostState;

impl ToFromWasmEvent for RaidFinishEvent {
    type Wit = RaidFinishEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            victory: self.victory,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            victory: data.victory,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for RaidSpawnWaveEvent {
    type Wit = RaidSpawnWaveEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            wave: self.wave,
            pos: to_wasm_block_position(self.pos),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            wave: data.wave,
            pos: from_wasm_block_position(data.pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for RaidStopEvent {
    type Wit = RaidStopEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            reason: self.reason.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.reason = data.reason;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            reason: data.reason,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for RaidTriggerEvent {
    type Wit = RaidTriggerEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            pos: to_wasm_block_position(self.pos),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            pos: from_wasm_block_position(data.pos),
            cancelled: data.cancelled,
        }
    }
}
