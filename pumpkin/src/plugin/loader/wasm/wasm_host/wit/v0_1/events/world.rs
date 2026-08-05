use std::sync::Arc;

use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{
                ToFromWasmEvent, consume_world, from_wasm_block_position, to_wasm_block_position,
            },
            pumpkin::plugin::event::{
                ChunkLoadEventData, ChunkSaveEventData, ChunkSendEventData, Event,
                SpawnChangeEventData, ThunderChangeEventData, WeatherChangeEventData,
                WorldLoadEventData, WorldUnloadEventData,
            },
        },
    },
    world::{
        chunk_load::ChunkLoad, chunk_save::ChunkSave, chunk_send::ChunkSend,
        spawn_change::SpawnChangeEvent,
    },
};

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

impl ToFromWasmEvent for ChunkLoad {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        let guard = self.chunk.blocking_read();
        Event::ChunkLoadEvent(ChunkLoadEventData {
            target_world,
            chunk_x: guard.x,
            chunk_z: guard.z,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::ChunkLoadEvent(data) => {
                let world = consume_world(state, &data.target_world);
                let chunk_data = pumpkin_world::chunk::ChunkData {
                    section: pumpkin_world::chunk::ChunkSections::new(24, -64),
                    heightmap: std::sync::Mutex::new(
                        pumpkin_world::chunk::ChunkHeightmaps::default(),
                    ),
                    x: data.chunk_x,
                    z: data.chunk_z,
                    block_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
                    fluid_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
                    pending_block_entities: std::sync::Mutex::new(
                        std::collections::HashMap::default(),
                    ),
                    light_engine: std::sync::Mutex::new(pumpkin_world::chunk::ChunkLight::default()),
                    light_populated: std::sync::atomic::AtomicBool::new(false),
                    status: pumpkin_data::chunk::ChunkStatus::Empty,
                    blending_data: None,
                    dirty: std::sync::atomic::AtomicBool::new(false),
                    inhabited_time: std::sync::atomic::AtomicU64::new(0),
                };
                Self {
                    world,
                    chunk: Arc::new(tokio::sync::RwLock::new(chunk_data)),
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for ChunkSave {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        let guard = self.chunk.blocking_read();
        Event::ChunkSaveEvent(ChunkSaveEventData {
            target_world,
            chunk_x: guard.x,
            chunk_z: guard.z,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::ChunkSaveEvent(data) => {
                let world = consume_world(state, &data.target_world);
                let chunk_data = pumpkin_world::chunk::ChunkData {
                    section: pumpkin_world::chunk::ChunkSections::new(24, -64),
                    heightmap: std::sync::Mutex::new(
                        pumpkin_world::chunk::ChunkHeightmaps::default(),
                    ),
                    x: data.chunk_x,
                    z: data.chunk_z,
                    block_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
                    fluid_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
                    pending_block_entities: std::sync::Mutex::new(
                        std::collections::HashMap::default(),
                    ),
                    light_engine: std::sync::Mutex::new(pumpkin_world::chunk::ChunkLight::default()),
                    light_populated: std::sync::atomic::AtomicBool::new(false),
                    status: pumpkin_data::chunk::ChunkStatus::Empty,
                    blending_data: None,
                    dirty: std::sync::atomic::AtomicBool::new(false),
                    inhabited_time: std::sync::atomic::AtomicU64::new(0),
                };
                Self {
                    world,
                    chunk: Arc::new(tokio::sync::RwLock::new(chunk_data)),
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for ChunkSend {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        Event::ChunkSendEvent(ChunkSendEventData {
            target_world,
            chunk_x: self.chunk.x,
            chunk_z: self.chunk.z,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::ChunkSendEvent(data) => {
                let world = consume_world(state, &data.target_world);
                let chunk_data = pumpkin_world::chunk::ChunkData {
                    section: pumpkin_world::chunk::ChunkSections::new(24, -64),
                    heightmap: std::sync::Mutex::new(
                        pumpkin_world::chunk::ChunkHeightmaps::default(),
                    ),
                    x: data.chunk_x,
                    z: data.chunk_z,
                    block_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
                    fluid_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
                    pending_block_entities: std::sync::Mutex::new(
                        std::collections::HashMap::default(),
                    ),
                    light_engine: std::sync::Mutex::new(pumpkin_world::chunk::ChunkLight::default()),
                    light_populated: std::sync::atomic::AtomicBool::new(false),
                    status: pumpkin_data::chunk::ChunkStatus::Empty,
                    blending_data: None,
                    dirty: std::sync::atomic::AtomicBool::new(false),
                    inhabited_time: std::sync::atomic::AtomicU64::new(0),
                };
                Self {
                    world,
                    chunk: Arc::new(chunk_data),
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::world::weather_change::WeatherChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        Event::WeatherChangeEvent(WeatherChangeEventData {
            target_world,
            to_weather_state: self.to_weather_state,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::WeatherChangeEvent(data) => Self {
                world: consume_world(state, &data.target_world),
                to_weather_state: data.to_weather_state,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::world::weather_change::ThunderChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        Event::ThunderChangeEvent(ThunderChangeEventData {
            target_world,
            to_thunder_state: self.to_thunder_state,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::ThunderChangeEvent(data) => Self {
                world: consume_world(state, &data.target_world),
                to_thunder_state: data.to_thunder_state,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::world::world_load::WorldLoadEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        Event::WorldLoadEvent(WorldLoadEventData { target_world })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::WorldLoadEvent(data) => Self {
                world: consume_world(state, &data.target_world),
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for crate::plugin::api::events::world::world_load::WorldUnloadEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        Event::WorldUnloadEvent(WorldUnloadEventData {
            target_world,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::WorldUnloadEvent(data) => Self {
                world: consume_world(state, &data.target_world),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
