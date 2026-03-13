use wasmtime::component::Resource;

use crate::plugin::{
    loader::wasm::wasm_host::{
        state::{PlayerResource, PluginHostState, TextComponentResource},
        wit::v0_1_0::{
            events::ToFromV0_1_0WasmEvent,
            pumpkin::plugin::event::{Event, PlayerJoinEventData, PlayerLeaveEventData},
        },
    },
    player::{player_join::PlayerJoinEvent, player_leave::PlayerLeaveEvent},
};

impl ToFromV0_1_0WasmEvent for PlayerJoinEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player_resource = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        let text_component_resource = state.add_text_component(self.join_message.clone()).unwrap();

        Event::PlayerJoinEvent(PlayerJoinEventData {
            player: player_resource,
            join_message: text_component_resource,
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(
        event: crate::plugin::loader::wasm::wasm_host::wit::v0_1_0::pumpkin::plugin::event::Event,
        state: &mut PluginHostState,
    ) -> Self {
        #[allow(clippy::match_wildcard_for_single_variants)]
        match event {
            Event::PlayerJoinEvent(data) => {
                let player_resource = state
                    .resource_table
                    .delete::<PlayerResource>(Resource::new_own(data.player.rep()))
                    .unwrap();

                let text_component_resource = state
                    .resource_table
                    .delete::<TextComponentResource>(Resource::new_own(data.join_message.rep()))
                    .unwrap();

                Self {
                    player: player_resource.provider,
                    join_message: text_component_resource.provider,
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerLeaveEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player_resource = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        let text_component_resource = state
            .add_text_component(self.leave_message.clone())
            .unwrap();

        Event::PlayerLeaveEvent(PlayerLeaveEventData {
            player: player_resource,
            leave_message: text_component_resource,
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(
        event: crate::plugin::loader::wasm::wasm_host::wit::v0_1_0::pumpkin::plugin::event::Event,
        state: &mut PluginHostState,
    ) -> Self {
        #[allow(clippy::match_wildcard_for_single_variants)]
        match event {
            Event::PlayerLeaveEvent(data) => {
                let player_resource = state
                    .resource_table
                    .delete::<PlayerResource>(Resource::new_own(data.player.rep()))
                    .unwrap();

                let text_component_resource = state
                    .resource_table
                    .delete::<TextComponentResource>(Resource::new_own(data.leave_message.rep()))
                    .unwrap();

                Self {
                    player: player_resource.provider,
                    leave_message: text_component_resource.provider,
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}
