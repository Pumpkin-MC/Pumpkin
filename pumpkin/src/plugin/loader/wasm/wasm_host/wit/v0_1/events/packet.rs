use bytes::Bytes;

use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PlayerResource,
        state::PluginHostState,
        wit::v0_1::{
            events::ToFromWasmEvent,
            pumpkin::plugin::{
                event::{Event, RawPacketEventData},
                packet::{
                    BedrockState as WasmBedrockState, ConnectionState as WasmConnectionState,
                    JavaState as WasmJavaState, PacketDirection as WasmPacketDirection,
                    RawPacket as WasmRawPacket, RawPacketEvent as WasmRawPacketEvent,
                },
            },
        },
    },
    packet::{
        BedrockConnectionState, JavaConnectionState, PacketConnectionState, PacketDirection,
        RawPacketData, RawPacketEvent,
    },
};
use wasmtime::component::Resource;

const fn to_wasm_direction(direction: PacketDirection) -> WasmPacketDirection {
    match direction {
        PacketDirection::Serverbound => WasmPacketDirection::Serverbound,
        PacketDirection::Clientbound => WasmPacketDirection::Clientbound,
    }
}

const fn from_wasm_direction(direction: WasmPacketDirection) -> PacketDirection {
    match direction {
        WasmPacketDirection::Serverbound => PacketDirection::Serverbound,
        WasmPacketDirection::Clientbound => PacketDirection::Clientbound,
    }
}

const fn to_wasm_state(state: PacketConnectionState) -> WasmConnectionState {
    match state {
        PacketConnectionState::Java(state) => WasmConnectionState::Java(match state {
            JavaConnectionState::Handshake => WasmJavaState::Handshake,
            JavaConnectionState::Status => WasmJavaState::Status,
            JavaConnectionState::Login => WasmJavaState::Login,
            JavaConnectionState::Config => WasmJavaState::Config,
            JavaConnectionState::Play => WasmJavaState::Play,
            JavaConnectionState::Transfer => WasmJavaState::Transfer,
        }),
        PacketConnectionState::Bedrock(state) => WasmConnectionState::Bedrock(match state {
            BedrockConnectionState::Offline => WasmBedrockState::Offline,
            BedrockConnectionState::Raknet => WasmBedrockState::Raknet,
            BedrockConnectionState::Game => WasmBedrockState::Game,
        }),
    }
}

const fn from_wasm_state(state: WasmConnectionState) -> PacketConnectionState {
    match state {
        WasmConnectionState::Java(state) => PacketConnectionState::Java(match state {
            WasmJavaState::Handshake => JavaConnectionState::Handshake,
            WasmJavaState::Status => JavaConnectionState::Status,
            WasmJavaState::Login => JavaConnectionState::Login,
            WasmJavaState::Config => JavaConnectionState::Config,
            WasmJavaState::Play => JavaConnectionState::Play,
            WasmJavaState::Transfer => JavaConnectionState::Transfer,
        }),
        WasmConnectionState::Bedrock(state) => PacketConnectionState::Bedrock(match state {
            WasmBedrockState::Offline => BedrockConnectionState::Offline,
            WasmBedrockState::Raknet => BedrockConnectionState::Raknet,
            WasmBedrockState::Game => BedrockConnectionState::Game,
        }),
    }
}

fn to_wasm_raw_packet(packet: &RawPacketData) -> WasmRawPacket {
    WasmRawPacket {
        id: packet.id,
        payload: packet.payload.to_vec(),
    }
}

fn from_wasm_raw_packet(packet: WasmRawPacket) -> RawPacketData {
    RawPacketData {
        id: packet.id,
        payload: Bytes::from(packet.payload),
    }
}

impl ToFromWasmEvent for RawPacketEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = self.player.as_ref().map(|player| {
            state
                .add_player(player.clone())
                .expect("failed to add player resource")
        });

        let raw_packet = to_wasm_raw_packet(&self.packet);
        let packet_event = WasmRawPacketEvent {
            direction: to_wasm_direction(self.direction),
            state: to_wasm_state(self.state),
            packet: raw_packet,
            cancelled: self.cancelled,
        };

        Event::RawPacketEvent(RawPacketEventData {
            player,
            packet: packet_event,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        let Event::RawPacketEvent(RawPacketEventData { player, packet }) = event else {
            panic!("unexpected event");
        };

        let player = player.map(|player| take_player(state, &player));
        let raw_packet = from_wasm_raw_packet(packet.packet);

        Self::new(
            player,
            from_wasm_direction(packet.direction),
            from_wasm_state(packet.state),
            raw_packet,
        )
        .with_cancelled(packet.cancelled)
    }
}

trait WithCancelled {
    fn with_cancelled(self, cancelled: bool) -> Self;
}

impl WithCancelled for RawPacketEvent {
    fn with_cancelled(mut self, cancelled: bool) -> Self {
        self.cancelled = cancelled;
        self
    }
}

fn take_player(
    state: &mut PluginHostState,
    player: &Resource<
        crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::player::Player,
    >,
) -> std::sync::Arc<crate::entity::player::Player> {
    state
        .resource_table
        .delete::<PlayerResource>(Resource::new_own(player.rep()))
        .expect("invalid player resource handle")
        .provider
}
