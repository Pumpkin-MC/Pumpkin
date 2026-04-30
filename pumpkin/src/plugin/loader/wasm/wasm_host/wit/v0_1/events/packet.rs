use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PlayerResource,
        state::PluginHostState,
        wit::v0_1::{
            events::ToFromWasmEvent,
            packet::PluginPacketHandle,
            pumpkin::plugin::event::{Event, RawPacketEventData},
        },
    },
    packet::RawPacketEvent,
};
use wasmtime::component::Resource;

impl ToFromWasmEvent for RawPacketEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = self.player.as_ref().map(|player| {
            state
                .add_player(player.clone())
                .expect("failed to add player resource")
        });

        let packet = state
            .add_packet_handle(PluginPacketHandle {
                event: self.clone(),
            })
            .expect("failed to add packet-handle resource");

        Event::RawPacketEvent(RawPacketEventData { player, packet })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        let Event::RawPacketEvent(RawPacketEventData { player, packet }) = event else {
            panic!("unexpected event");
        };

        if let Some(player) = player {
            state
                .resource_table
                .delete::<PlayerResource>(Resource::new_own(player.rep()))
                .expect("invalid player resource handle");
        }

        state
            .resource_table
            .delete::<crate::plugin::loader::wasm::wasm_host::state::PacketHandleResource>(
                Resource::new_own(packet.rep()),
            )
            .expect("invalid packet-handle resource")
            .provider
            .event
    }
}
