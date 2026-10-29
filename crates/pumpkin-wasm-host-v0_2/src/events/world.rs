use std::sync::Arc;

use crate::{
    events::{
        CleanupEvent, ToFromWasmEvent, consume_world, from_wasm_block_position,
        to_wasm_block_position,
    },
    pumpkin::plugin::event::{
        ChunkLoadEvent as ChunkLoadEventData, ChunkSaveEvent as ChunkSaveEventData,
        ChunkSendEvent as ChunkSendEventData, SpawnChangeEvent as SpawnChangeEventData,
        ThunderChangeEvent as ThunderChangeEventData, WeatherChangeEvent as WeatherChangeEventData,
        WorldLoadEvent as WorldLoadEventData, WorldUnloadEvent as WorldUnloadEventData,
    },
};
use pumpkin_core::plugin::world::{
    chunk_load::ChunkLoad, chunk_save::ChunkSave, chunk_send::ChunkSend,
    spawn_change::SpawnChangeEvent,
};
use pumpkin_wasm_host_common::state::PluginHostState;

impl ToFromWasmEvent for SpawnChangeEvent {
    type Wit = SpawnChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        Self::Wit {
            target_world: world,
            previous_position: to_wasm_block_position(self.previous_position),
            previous_yaw: self.previous_yaw,
            previous_pitch: self.previous_pitch,
            new_position: to_wasm_block_position(self.new_position),
            new_yaw: self.new_yaw,
            new_pitch: self.new_pitch,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            world: consume_world(state, &data.target_world),
            previous_position: from_wasm_block_position(data.previous_position),
            previous_yaw: data.previous_yaw,
            previous_pitch: data.previous_pitch,
            new_position: from_wasm_block_position(data.new_position),
            new_yaw: data.new_yaw,
            new_pitch: data.new_pitch,
        }
    }
}

impl ToFromWasmEvent for ChunkLoad {
    type Wit = ChunkLoadEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        let (chunk_x, chunk_z) = self
            .chunk
            .try_read()
            .map_or((0, 0), |guard| (guard.x, guard.z));

        Self::Wit {
            target_world,
            chunk_x,
            chunk_z,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        let world = consume_world(state, &data.target_world);
        let chunk_data = pumpkin_world::chunk::ChunkData {
            section: pumpkin_world::chunk::ChunkSections::new(24, -64),
            heightmap: std::sync::Mutex::new(pumpkin_world::chunk::ChunkHeightmaps::default()),
            x: data.chunk_x,
            z: data.chunk_z,
            block_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
            fluid_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
            pending_block_entities: std::sync::Mutex::new(std::collections::HashMap::default()),
            light_engine: std::sync::Mutex::new(pumpkin_world::chunk::ChunkLight::default()),
            light_populated: std::sync::atomic::AtomicBool::new(false),
            status: pumpkin_data::chunk::ChunkStatus::Empty,
            blending_data: None,
            dirty: std::sync::atomic::AtomicBool::new(false),
            inhabited_time: std::sync::atomic::AtomicU64::new(0),
            custom_data: std::sync::Mutex::new(pumpkin_nbt::compound::NbtCompound::new()),
        };
        Self {
            world,
            chunk: Arc::new(tokio::sync::RwLock::new(chunk_data)),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for ChunkSave {
    type Wit = ChunkSaveEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        let (chunk_x, chunk_z) = self
            .chunk
            .try_read()
            .map_or((0, 0), |guard| (guard.x, guard.z));

        Self::Wit {
            target_world,
            chunk_x,
            chunk_z,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        let world = consume_world(state, &data.target_world);
        let chunk_data = pumpkin_world::chunk::ChunkData {
            section: pumpkin_world::chunk::ChunkSections::new(24, -64),
            heightmap: std::sync::Mutex::new(pumpkin_world::chunk::ChunkHeightmaps::default()),
            x: data.chunk_x,
            z: data.chunk_z,
            block_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
            fluid_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
            pending_block_entities: std::sync::Mutex::new(std::collections::HashMap::default()),
            light_engine: std::sync::Mutex::new(pumpkin_world::chunk::ChunkLight::default()),
            light_populated: std::sync::atomic::AtomicBool::new(false),
            status: pumpkin_data::chunk::ChunkStatus::Empty,
            blending_data: None,
            dirty: std::sync::atomic::AtomicBool::new(false),
            inhabited_time: std::sync::atomic::AtomicU64::new(0),
            custom_data: std::sync::Mutex::new(pumpkin_nbt::compound::NbtCompound::new()),
        };
        Self {
            world,
            chunk: Arc::new(tokio::sync::RwLock::new(chunk_data)),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for ChunkSend {
    type Wit = ChunkSendEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        Self::Wit {
            target_world,
            chunk_x: self.chunk.x,
            chunk_z: self.chunk.z,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        let world = consume_world(state, &data.target_world);
        let chunk_data = pumpkin_world::chunk::ChunkData {
            section: pumpkin_world::chunk::ChunkSections::new(24, -64),
            heightmap: std::sync::Mutex::new(pumpkin_world::chunk::ChunkHeightmaps::default()),
            x: data.chunk_x,
            z: data.chunk_z,
            block_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
            fluid_ticks: pumpkin_world::tick::scheduler::ChunkTickScheduler::default(),
            pending_block_entities: std::sync::Mutex::new(std::collections::HashMap::default()),
            light_engine: std::sync::Mutex::new(pumpkin_world::chunk::ChunkLight::default()),
            light_populated: std::sync::atomic::AtomicBool::new(false),
            status: pumpkin_data::chunk::ChunkStatus::Empty,
            blending_data: None,
            dirty: std::sync::atomic::AtomicBool::new(false),
            inhabited_time: std::sync::atomic::AtomicU64::new(0),
            custom_data: std::sync::Mutex::new(pumpkin_nbt::compound::NbtCompound::new()),
        };
        Self {
            world,
            chunk: Arc::new(chunk_data),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::world::weather_change::WeatherChangeEvent
{
    type Wit = WeatherChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        Self::Wit {
            target_world,
            to_weather_state: self.to_weather_state,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            world: consume_world(state, &data.target_world),
            to_weather_state: data.to_weather_state,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::world::weather_change::ThunderChangeEvent
{
    type Wit = ThunderChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        Self::Wit {
            target_world,
            to_thunder_state: self.to_thunder_state,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            world: consume_world(state, &data.target_world),
            to_thunder_state: data.to_thunder_state,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::api::events::world::world_load::WorldLoadEvent {
    type Wit = WorldLoadEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        Self::Wit { target_world }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            world: consume_world(state, &data.target_world),
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::api::events::world::world_load::WorldUnloadEvent {
    type Wit = WorldUnloadEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        Self::Wit {
            target_world,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            world: consume_world(state, &data.target_world),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::world::async_structure_generate::AsyncStructureGenerateEvent
{
    type Wit = crate::pumpkin::plugin::event::AsyncStructureGenerateEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            world_name: self.world_name.clone(),
            structure_name: self.structure_name.clone(),
            pos: to_wasm_block_position(self.pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            world_name: data.world_name,
            structure_name: data.structure_name,
            pos: from_wasm_block_position(data.pos),
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::world::async_structure_spawn::AsyncStructureSpawnEvent
{
    type Wit = crate::pumpkin::plugin::event::AsyncStructureSpawnEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            world_name: self.world_name.clone(),
            structure_name: self.structure_name.clone(),
            pos: to_wasm_block_position(self.pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            world_name: data.world_name,
            structure_name: data.structure_name,
            pos: from_wasm_block_position(data.pos),
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::world::chunk_populate::ChunkPopulateEvent
{
    type Wit = crate::pumpkin::plugin::event::ChunkPopulateEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            chunk_x: self.chunk_pos.x,
            chunk_z: self.chunk_pos.y,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            chunk_pos: pumpkin_util::math::vector2::Vector2::new(data.chunk_x, data.chunk_z),
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::api::events::world::chunk_unload::ChunkUnloadEvent {
    type Wit = crate::pumpkin::plugin::event::ChunkUnloadEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            chunk_x: self.chunk_pos.x,
            chunk_z: self.chunk_pos.y,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            chunk_pos: pumpkin_util::math::vector2::Vector2::new(data.chunk_x, data.chunk_z),
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::world::entities_load::EntitiesLoadEvent
{
    type Wit = crate::pumpkin::plugin::event::EntitiesLoadEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            chunk_x: self.chunk_pos.x,
            chunk_z: self.chunk_pos.y,
            entity_count: self.entity_count as u32,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            chunk_pos: pumpkin_util::math::vector2::Vector2::new(data.chunk_x, data.chunk_z),
            entity_count: data.entity_count as usize,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::world::entities_unload::EntitiesUnloadEvent
{
    type Wit = crate::pumpkin::plugin::event::EntitiesUnloadEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            chunk_x: self.chunk_pos.x,
            chunk_z: self.chunk_pos.y,
            entity_count: self.entity_count as u32,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            chunk_pos: pumpkin_util::math::vector2::Vector2::new(data.chunk_x, data.chunk_z),
            entity_count: data.entity_count as usize,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::api::events::world::generic_game::GenericGameEvent {
    type Wit = crate::pumpkin::plugin::event::GenericGameEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            event_id: self.event_key.clone(),
            pos: (self.position.x, self.position.y, self.position.z),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            event_key: data.event_id,
            position: pumpkin_util::math::vector3::Vector3::new(data.pos.0, data.pos.1, data.pos.2),
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::world::loot_generate::LootGenerateEvent
{
    type Wit = crate::pumpkin::plugin::event::LootGenerateEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            loot_table: self.loot_table.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            loot_table: data.loot_table,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::world::portal_create::PortalCreateEvent
{
    type Wit = crate::pumpkin::plugin::event::PortalCreateEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            pos: to_wasm_block_position(self.block_pos),
            portal_type: format!("{:?}", self.portal_type),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.pos),
            portal_type: match data.portal_type.as_str() {
                "Nether" => {
                    pumpkin_core::plugin::api::events::world::portal_create::PortalType::Nether
                }
                "End" => pumpkin_core::plugin::api::events::world::portal_create::PortalType::End,
                _ => pumpkin_core::plugin::api::events::world::portal_create::PortalType::Custom,
            },
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::world::structure_grow::StructureGrowEvent
{
    type Wit = crate::pumpkin::plugin::event::StructureGrowEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            pos: to_wasm_block_position(self.block_pos),
            species: format!("{:?}", self.species),
            bone_meal: self.bone_meal,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.pos),
            species: match data.species.as_str() {
                "Oak" => pumpkin_core::plugin::api::events::world::structure_grow::TreeType::Oak,
                "Spruce" => pumpkin_core::plugin::api::events::world::structure_grow::TreeType::Spruce,
                "Birch" => pumpkin_core::plugin::api::events::world::structure_grow::TreeType::Birch,
                "Jungle" => pumpkin_core::plugin::api::events::world::structure_grow::TreeType::Jungle,
                "Acacia" => pumpkin_core::plugin::api::events::world::structure_grow::TreeType::Acacia,
                "DarkOak" => {
                    pumpkin_core::plugin::api::events::world::structure_grow::TreeType::DarkOak
                }
                "Mangrove" => {
                    pumpkin_core::plugin::api::events::world::structure_grow::TreeType::Mangrove
                }
                "Cherry" => pumpkin_core::plugin::api::events::world::structure_grow::TreeType::Cherry,
                "Poplar" => pumpkin_core::plugin::api::events::world::structure_grow::TreeType::Poplar,
                "Azalea" => pumpkin_core::plugin::api::events::world::structure_grow::TreeType::Azalea,
                "BrownMushroom" => {
                    pumpkin_core::plugin::api::events::world::structure_grow::TreeType::BrownMushroom
                }
                "RedMushroom" => {
                    pumpkin_core::plugin::api::events::world::structure_grow::TreeType::RedMushroom
                }
                _ => pumpkin_core::plugin::api::events::world::structure_grow::TreeType::Custom,
            },
            bone_meal: data.bone_meal,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::api::events::world::time_skip::TimeSkipEvent {
    type Wit = crate::pumpkin::plugin::event::TimeSkipEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            skip_amount: self.skip_amount,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            skip_amount: data.skip_amount,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::api::events::world::world_init::WorldInitEvent {
    type Wit = crate::pumpkin::plugin::event::WorldInitEvent;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        Self::Wit { target_world }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            world: consume_world(state, &data.target_world),
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::api::events::world::world_save::WorldSaveEvent {
    type Wit = crate::pumpkin::plugin::event::WorldSaveEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            world_name: self.world_name.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            world_name: data.world_name,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::world::lightning_strike::LightningStrikeEvent
{
    type Wit = crate::pumpkin::plugin::event::LightningStrikeEvent;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            position: crate::events::to_wasm_position(self.position),
            is_effect: self.is_effect,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            position: crate::events::from_wasm_position(data.position),
            is_effect: data.is_effect,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}
