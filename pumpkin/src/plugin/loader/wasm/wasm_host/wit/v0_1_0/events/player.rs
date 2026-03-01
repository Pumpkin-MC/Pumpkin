use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1_0::{
            events::ToV0_1_0WasmEvent,
            pumpkin::plugin::event::{Event, PlayerJoinEventData},
        },
    },
    player::player_join::PlayerJoinEvent,
};

impl ToV0_1_0WasmEvent for PlayerJoinEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player_resource = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        let text_component_resource = state.add_text_component(self.join_message.clone()).unwrap();

        Event::PlayerJoinEvent(PlayerJoinEventData {
            player: player_resource,
            text_component: text_component_resource,
            cancelled: self.cancelled,
        })
    }
}
