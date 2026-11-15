use crate::{
    events::{
        ToFromWasmEvent, cleanup_event, consume_world, from_wasm_block_position,
        to_wasm_block_position,
    },
    pumpkin::plugin::event::{
        ChunkLoadEventData, ChunkSaveEventData, ChunkSendEventData, Event, SpawnChangeEventData,
        ThunderChangeEventData, WeatherChangeEventData, WorldLoadEventData, WorldUnloadEventData,
    },
};
use pumpkin_core::plugin::{ChunkLoadEvent, ChunkSaveEvent, ChunkSendEvent, SpawnChangeEvent};
use pumpkin_wasm_host_common::state::PluginHostState;

impl ToFromWasmEvent for SpawnChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let world = state
            .add(self.target_world.clone())
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
                target_world: consume_world(state, &data.target_world),
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

impl ToFromWasmEvent for ChunkLoadEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add(self.target_world.clone())
            .expect("failed to add world resource");

        Event::ChunkLoadEvent(ChunkLoadEventData {
            target_world,
            chunk_x: self.chunk_x,
            chunk_z: self.chunk_z,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::ChunkLoadEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for ChunkSaveEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add(self.target_world.clone())
            .expect("failed to add world resource");

        Event::ChunkSaveEvent(ChunkSaveEventData {
            target_world,
            chunk_x: self.chunk_x,
            chunk_z: self.chunk_z,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::ChunkSaveEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for ChunkSendEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add(self.target_world.clone())
            .expect("failed to add world resource");

        Event::ChunkSendEvent(ChunkSendEventData {
            target_world,
            chunk_x: self.chunk_x,
            chunk_z: self.chunk_z,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::ChunkSendEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::WeatherChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add(self.target_world.clone())
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
                target_world: consume_world(state, &data.target_world),
                to_weather_state: data.to_weather_state,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::ThunderChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add(self.target_world.clone())
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
                target_world: consume_world(state, &data.target_world),
                to_thunder_state: data.to_thunder_state,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::WorldLoadEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add(self.target_world.clone())
            .expect("failed to add world resource");

        Event::WorldLoadEvent(WorldLoadEventData { target_world })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::WorldLoadEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::WorldUnloadEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add(self.target_world.clone())
            .expect("failed to add world resource");

        Event::WorldUnloadEvent(WorldUnloadEventData {
            target_world,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::WorldUnloadEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::AsyncStructureGenerateEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::AsyncStructureGenerateEvent(
            crate::pumpkin::plugin::event::AsyncStructureGenerateEventData {
                world_name: self.world_name.clone(),
                structure_name: self.structure_name.clone(),
                pos: to_wasm_block_position(self.pos),
                cancelled: self.cancelled,
            },
        )
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::AsyncStructureGenerateEvent(data) => Self {
                world_name: data.world_name,
                structure_name: data.structure_name,
                pos: from_wasm_block_position(data.pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::AsyncStructureGenerateEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::AsyncStructureSpawnEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::AsyncStructureSpawnEvent(
            crate::pumpkin::plugin::event::AsyncStructureSpawnEventData {
                world_name: self.world_name.clone(),
                structure_name: self.structure_name.clone(),
                pos: to_wasm_block_position(self.pos),
                cancelled: self.cancelled,
            },
        )
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::AsyncStructureSpawnEvent(data) => Self {
                world_name: data.world_name,
                structure_name: data.structure_name,
                pos: from_wasm_block_position(data.pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::AsyncStructureSpawnEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::ChunkPopulateEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::ChunkPopulateEvent(crate::pumpkin::plugin::event::ChunkPopulateEventData {
            chunk_x: self.chunk_x,
            chunk_z: self.chunk_z,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::ChunkPopulateEvent(data) => Self {
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::ChunkPopulateEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::ChunkUnloadEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::ChunkUnloadEvent(crate::pumpkin::plugin::event::ChunkUnloadEventData {
            chunk_x: self.chunk_x,
            chunk_z: self.chunk_z,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::ChunkUnloadEvent(data) => Self {
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::ChunkUnloadEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::EntitiesLoadEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntitiesLoadEvent(crate::pumpkin::plugin::event::EntitiesLoadEventData {
            chunk_x: self.chunk_x,
            chunk_z: self.chunk_z,
            entity_count: self.entity_count,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntitiesLoadEvent(data) => Self {
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                entity_count: data.entity_count,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::EntitiesLoadEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::EntitiesUnloadEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntitiesUnloadEvent(crate::pumpkin::plugin::event::EntitiesUnloadEventData {
            chunk_x: self.chunk_x,
            chunk_z: self.chunk_z,
            entity_count: self.entity_count,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntitiesUnloadEvent(data) => Self {
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                entity_count: data.entity_count,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::EntitiesUnloadEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::GenericGameEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::GenericGameEvent(crate::pumpkin::plugin::event::GenericGameEventData {
            event_id: self.event_id.clone(),
            pos: (self.pos.x, self.pos.y, self.pos.z),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::GenericGameEvent(data) => Self {
                event_id: data.event_id,
                pos: pumpkin_util::math::vector3::Vector3::new(data.pos.0, data.pos.1, data.pos.2),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::GenericGameEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::LootGenerateEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::LootGenerateEvent(crate::pumpkin::plugin::event::LootGenerateEventData {
            loot_table: self.loot_table.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::LootGenerateEvent(data) => Self {
                loot_table: data.loot_table,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::LootGenerateEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::PortalCreateEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::PortalCreateEvent(crate::pumpkin::plugin::event::PortalCreateEventData {
            pos: to_wasm_block_position(self.pos),
            portal_type: format!("{:?}", self.portal_type),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::PortalCreateEvent(data) => Self {
                pos: from_wasm_block_position(data.pos),
                portal_type: match data.portal_type.as_str() {
                    "Nether" => pumpkin_core::plugin::PortalType::Nether,
                    "End" => pumpkin_core::plugin::PortalType::End,
                    _ => pumpkin_core::plugin::PortalType::Custom,
                },
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::PortalCreateEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::StructureGrowEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::StructureGrowEvent(crate::pumpkin::plugin::event::StructureGrowEventData {
            pos: to_wasm_block_position(self.pos),
            species: format!("{:?}", self.species),
            bone_meal: self.bone_meal,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::StructureGrowEvent(data) => Self {
                pos: from_wasm_block_position(data.pos),
                species: match data.species.as_str() {
                    "Oak" => pumpkin_core::plugin::TreeType::Oak,
                    "Spruce" => pumpkin_core::plugin::TreeType::Spruce,
                    "Birch" => pumpkin_core::plugin::TreeType::Birch,
                    "Jungle" => pumpkin_core::plugin::TreeType::Jungle,
                    "Acacia" => pumpkin_core::plugin::TreeType::Acacia,
                    "DarkOak" => pumpkin_core::plugin::TreeType::DarkOak,
                    "Mangrove" => pumpkin_core::plugin::TreeType::Mangrove,
                    "Cherry" => pumpkin_core::plugin::TreeType::Cherry,
                    "Poplar" => pumpkin_core::plugin::TreeType::Poplar,
                    "Azalea" => pumpkin_core::plugin::TreeType::Azalea,
                    "BrownMushroom" => pumpkin_core::plugin::TreeType::BrownMushroom,
                    "RedMushroom" => pumpkin_core::plugin::TreeType::RedMushroom,
                    _ => pumpkin_core::plugin::TreeType::Custom,
                },
                bone_meal: data.bone_meal,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::StructureGrowEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::TimeSkipEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::TimeSkipEvent(crate::pumpkin::plugin::event::TimeSkipEventData {
            skip_amount: self.skip_amount,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::TimeSkipEvent(data) => Self {
                skip_amount: data.skip_amount,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::TimeSkipEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::WorldInitEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add(self.target_world.clone())
            .expect("failed to add world resource");

        Event::WorldInitEvent(crate::pumpkin::plugin::event::WorldInitEventData { target_world })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::WorldInitEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::WorldSaveEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::WorldSaveEvent(crate::pumpkin::plugin::event::WorldSaveEventData {
            world_name: self.world_name.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::WorldSaveEvent(data) => Self {
                world_name: data.world_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::WorldSaveEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::LightningStrikeEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::LightningStrikeEvent(crate::pumpkin::plugin::event::LightningStrikeEventData {
            position: crate::events::to_wasm_position(self.position),
            is_effect: self.is_effect,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::LightningStrikeEvent(data) => Self {
                position: crate::events::from_wasm_position(data.position),
                is_effect: data.is_effect,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::LightningStrikeEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}
