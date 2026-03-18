use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1_0::{
            events::{
                ToFromV0_1_0WasmEvent, consume_world, from_wasm_block_position,
                to_wasm_block_position,
            },
            pumpkin::plugin::event::{Event, SpawnChangeEventData},
        },
    },
    world::spawn_change::SpawnChangeEvent,
};

impl ToFromV0_1_0WasmEvent for SpawnChangeEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
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

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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
