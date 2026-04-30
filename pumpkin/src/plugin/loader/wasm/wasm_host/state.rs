use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::text::TextComponent;
use tokio::sync::Mutex;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::{
    command::{
        CommandSender,
        args::ConsumedArgs,
        tree::{CommandTree, builder::NonLeafNodeBuilder},
    },
    entity::EntityBase,
    entity::player::Player,
    net::ClientPlatform,
    plugin::{
        Context,
        api::gui::PluginGui,
        loader::wasm::wasm_host::{WasmPlugin, args::OwnedArg},
    },
    server::Server,
    world::World,
};

pub struct WasmResource<T> {
    pub provider: T,
}

pub type ServerResource = WasmResource<Arc<Server>>;
pub type ContextResource = WasmResource<Arc<Context>>;
pub type PlayerResource = WasmResource<Arc<Player>>;
pub type EntityResource = WasmResource<Arc<dyn EntityBase>>;
pub type WorldResource = WasmResource<Arc<World>>;
pub type ScoreboardResource = WasmResource<Arc<World>>;
pub type GuiResource = WasmResource<Arc<Mutex<PluginGui>>>;
pub type BossBarResource = WasmResource<
    Arc<Mutex<crate::plugin::loader::wasm::wasm_host::wit::v0_1::boss_bar::PluginBossBar>>,
>;
pub type PacketHandleResource =
    WasmResource<crate::plugin::loader::wasm::wasm_host::wit::v0_1::packet::PluginPacketHandle>;
pub type PacketReaderResource =
    WasmResource<crate::plugin::loader::wasm::wasm_host::wit::v0_1::packet::PluginPacketReader>;
pub type PacketWriterResource =
    WasmResource<crate::plugin::loader::wasm::wasm_host::wit::v0_1::packet::PluginPacketWriter>;
pub type TextComponentResource = WasmResource<TextComponent>;
pub type CommandResource = WasmResource<CommandTree>;
pub type CommandSenderResource = WasmResource<CommandSender>;
pub type ConsumedArgsResource = WasmResource<OwnedConsumedArgs>;
pub type CommandNodeResource = WasmResource<NonLeafNodeBuilder>;

pub type OwnedConsumedArgs = HashMap<String, OwnedArg>;

pub enum PendingEffect {
    PlayerSystemMessage {
        player: Arc<Player>,
        text: TextComponent,
        overlay: bool,
    },
    PlayerCustomPayload {
        player: Arc<Player>,
        channel: String,
        data: Vec<u8>,
    },
    PlayerRawPacket {
        player: Arc<Player>,
        id: i32,
        payload: Vec<u8>,
    },
    ServerBroadcastCustomPayload {
        server: Arc<Server>,
        channel: String,
        data: Vec<u8>,
    },
    ServerBroadcastRawPacket {
        server: Arc<Server>,
        id: i32,
        payload: Vec<u8>,
    },
}

impl PendingEffect {
    pub async fn execute(self) -> Result<(), wasmtime::Error> {
        match self {
            Self::PlayerSystemMessage {
                player,
                text,
                overlay,
            } => {
                player.send_system_message_raw(&text, overlay).await;
            }
            Self::PlayerCustomPayload {
                player,
                channel,
                data,
            } => {
                player.send_custom_payload(&channel, &data).await;
            }
            Self::PlayerRawPacket {
                player,
                id,
                payload,
            } => match &player.client {
                ClientPlatform::Java(java) => {
                    let mut buf = Vec::new();
                    VarInt(id)
                        .encode(&mut buf)
                        .map_err(|err| wasmtime::Error::msg(err.to_string()))?;
                    buf.extend_from_slice(&payload);
                    java.enqueue_packet_data(buf.into()).await;
                }
                ClientPlatform::Bedrock(bedrock) => {
                    bedrock
                        .send_raw_game_packet(id, payload)
                        .await
                        .map_err(|err| wasmtime::Error::msg(err.to_string()))?;
                }
            },
            Self::ServerBroadcastCustomPayload {
                server,
                channel,
                data,
            } => {
                for player in server.get_all_players() {
                    player.send_custom_payload(&channel, &data).await;
                }
            }
            Self::ServerBroadcastRawPacket {
                server,
                id,
                payload,
            } => {
                for player in server.get_all_players() {
                    match &player.client {
                        ClientPlatform::Java(java) => {
                            let mut buf = Vec::new();
                            VarInt(id)
                                .encode(&mut buf)
                                .map_err(|err| wasmtime::Error::msg(err.to_string()))?;
                            buf.extend_from_slice(&payload);
                            java.enqueue_packet_data(buf.clone().into()).await;
                        }
                        ClientPlatform::Bedrock(bedrock) => {
                            bedrock
                                .send_raw_game_packet(id, payload.clone())
                                .await
                                .map_err(|err| wasmtime::Error::msg(err.to_string()))?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct PluginHostState {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
    pub plugin: Option<Weak<WasmPlugin>>,
    pub server: Option<Arc<Server>>,
    pub event_dispatch_depth: usize,
    pub pending_effects: Vec<PendingEffect>,
    pub permissions: Vec<String>,
}

impl Default for PluginHostState {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginHostState {
    #[must_use]
    pub fn new() -> Self {
        let resource_table = ResourceTable::new();
        Self {
            wasi_ctx: WasiCtxBuilder::new().build(),
            resource_table,
            plugin: None,
            server: None,
            event_dispatch_depth: 0,
            pending_effects: Vec::new(),
            permissions: Vec::new(),
        }
    }

    pub const fn begin_event_dispatch(&mut self) {
        self.event_dispatch_depth += 1;
    }

    pub fn finish_event_dispatch(&mut self) -> Vec<PendingEffect> {
        self.event_dispatch_depth = self.event_dispatch_depth.saturating_sub(1);
        if self.event_dispatch_depth == 0 {
            return std::mem::take(&mut self.pending_effects);
        }
        Vec::new()
    }

    #[must_use]
    pub const fn should_defer_effects(&self) -> bool {
        self.event_dispatch_depth > 0
    }

    pub fn defer_effect(&mut self, effect: PendingEffect) {
        self.pending_effects.push(effect);
    }

    pub fn add_server<T>(
        &mut self,
        provider: Arc<Server>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(ServerResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_context<T>(
        &mut self,
        provider: Arc<Context>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(ContextResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_player<T>(
        &mut self,
        provider: Arc<Player>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(PlayerResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_entity<T>(
        &mut self,
        provider: Arc<dyn EntityBase>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(EntityResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_world<T>(
        &mut self,
        provider: Arc<World>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(WorldResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_scoreboard<T>(
        &mut self,
        provider: Arc<World>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(ScoreboardResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_gui<T>(
        &mut self,
        provider: Arc<Mutex<PluginGui>>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(GuiResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_packet_handle<T>(
        &mut self,
        provider: crate::plugin::loader::wasm::wasm_host::wit::v0_1::packet::PluginPacketHandle,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self
            .resource_table
            .push(PacketHandleResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_packet_reader<T>(
        &mut self,
        provider: crate::plugin::loader::wasm::wasm_host::wit::v0_1::packet::PluginPacketReader,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self
            .resource_table
            .push(PacketReaderResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_packet_writer<T>(
        &mut self,
        provider: crate::plugin::loader::wasm::wasm_host::wit::v0_1::packet::PluginPacketWriter,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self
            .resource_table
            .push(PacketWriterResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_boss_bar<T>(
        &mut self,
        provider: Arc<
            Mutex<crate::plugin::loader::wasm::wasm_host::wit::v0_1::boss_bar::PluginBossBar>,
        >,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(BossBarResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_text_component<T>(
        &mut self,
        provider: TextComponent,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self
            .resource_table
            .push(TextComponentResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_command<T>(
        &mut self,
        provider: CommandTree,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(CommandResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_command_sender<T>(
        &mut self,
        command_sender: CommandSender,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(CommandSenderResource {
            provider: command_sender,
        })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_consumed_args<T>(
        &mut self,
        provider: &ConsumedArgs<'_>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let owned: HashMap<String, OwnedArg> = provider
            .iter()
            .map(|(k, v)| (k.to_string(), OwnedArg::from_arg(v)))
            .collect();
        let resource = self
            .resource_table
            .push(ConsumedArgsResource { provider: owned })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_command_node<T>(
        &mut self,
        provider: NonLeafNodeBuilder,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(CommandNodeResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }
}

impl WasiView for PluginHostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}
