use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{
                ToFromWasmEvent, consume_chunk, consume_world, from_wasm_block_position,
                to_wasm_block_position,
            },
            pumpkin::plugin::{
                event::{ChunkSendEventData, Event, SpawnChangeEventData},
                world::Chunk as WitChunk,
            },
        },
    },
    world::{chunk_send::ChunkSend, spawn_change::SpawnChangeEvent},
};
use std::sync::Arc;

impl ToFromWasmEvent for SpawnChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        Event::SpawnChangeEvent(SpawnChangeEventData {
            target_world: world,
            previous_position: to_wasm_block_position(self.previous_position),
            previous_yaw: self.previous_yaw,
            previous_pitch: self.previous_pitch,
            new_position: to_wasm_block_position(self.new_position),
            new_yaw: self.new_yaw,
            new_pitch: self.new_pitch,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::SpawnChangeEvent(data) => Self {
                world: consume_world(state, &data.target_world),
                previous_position: from_wasm_block_position(data.previous_position),
                previous_yaw: data.previous_yaw,
                previous_pitch: data.previous_pitch,
                new_position: from_wasm_block_position(data.new_position),
                new_yaw: data.new_yaw,
                new_pitch: data.new_pitch,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for ChunkSend {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let chunk = state
            .add_chunk::<WitChunk>(self.world.clone(), Arc::downgrade(&self.chunk))
            .expect("failed to add chunk resource");

        Event::ChunkSendEvent(ChunkSendEventData {
            target_world: world,
            chunk,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::ChunkSendEvent(data) => {
                let world = consume_world(state, &data.target_world);
                let (_, chunk) = consume_chunk(state, &data.chunk);
                Self {
                    world,
                    chunk,
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}
