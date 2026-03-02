use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use pumpkin_util::text::TextComponent;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::{
    command::{
        CommandSender,
        args::ConsumedArgs,
        tree::{CommandTree, builder::NonLeafNodeBuilder},
    },
    entity::player::Player,
    plugin::{
        Context,
        loader::wasm::wasm_host::{WasmPlugin, args::OwnedArg},
    },
    server::Server,
};

pub struct WasmResource<T> {
    pub provider: T,
}

pub type ServerResource = WasmResource<Arc<Server>>;
pub type ContextResource = WasmResource<Arc<Context>>;
pub type PlayerResource = WasmResource<Arc<Player>>;
pub type TextComponentResource = WasmResource<TextComponent>;
pub type CommandResource = WasmResource<CommandTree>;
pub type CommandSenderResource = WasmResource<CommandSender>;
pub type ConsumedArgsResource = WasmResource<OwnedConsumedArgs>;
pub type CommandNodeResource = WasmResource<NonLeafNodeBuilder>;

pub type OwnedConsumedArgs = HashMap<String, OwnedArg>;

pub struct PluginHostState {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
    pub plugin: Option<Weak<WasmPlugin>>,
    pub server: Option<Arc<Server>>,
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
        }
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
