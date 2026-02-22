use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1_0::{
            events::IntoV0_1_0WasmEvent,
            pumpkin::plugin::event::{Event, PlayerJoinEventData},
        },
    },
    player::player_join::PlayerJoinEvent,
};

impl IntoV0_1_0WasmEvent for PlayerJoinEvent {
    fn into_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player_resource = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        // TODO - Do not leave this around before doing a v1.0.0 official release, we should have a resource for text components in the future
        let text_component =
            serde_json::to_vec(&self.join_message).expect("failed to serialize text component");

        Event::PlayerJoinEvent(PlayerJoinEventData {
            player: player_resource,
            text_component: text_component,
            cancelled: self.cancelled,
        })
    }
}
