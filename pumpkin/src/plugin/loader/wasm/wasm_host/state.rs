use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use pumpkin_util::text::TextComponent;
use tokio::sync::Mutex;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
use wasmtime_wasi_http::{
    WasiHttpCtx,
    p2::{
        HttpError, HttpResult, WasiHttpCtxView, WasiHttpHooks, WasiHttpView,
        bindings::http::types::ErrorCode, default_send_request,
    },
};

use crate::{
    command::{
        CommandSender,
        args::ConsumedArgs,
        tree::{CommandTree, builder::NonLeafNodeBuilder},
    },
    entity::EntityBase,
    entity::player::Player,
    plugin::{
        Context,
        api::gui::PluginGui,
        loader::wasm::wasm_host::{WasmPlugin, args::OwnedArg},
    },
    server::{RecipeManager, Server},
    world::World,
};

pub struct WasmResource<T> {
    pub provider: T,
}

pub type ServerResource = WasmResource<Arc<Server>>;
pub type ContextResource = WasmResource<Arc<Context>>;
pub type PlayerResource = WasmResource<Arc<Player>>;
pub type JavaPlayerResource = WasmResource<Arc<Player>>;
pub type BedrockPlayerResource = WasmResource<Arc<Player>>;
pub type EntityResource = WasmResource<Arc<dyn EntityBase>>;
pub type WorldResource = WasmResource<Arc<World>>;
pub type ScoreboardResource = WasmResource<Arc<World>>;
pub type GuiResource = WasmResource<Arc<Mutex<PluginGui>>>;
pub type BossBarResource = WasmResource<
    Arc<Mutex<crate::plugin::loader::wasm::wasm_host::wit::v0_1::boss_bar::PluginBossBar>>,
>;
pub type TextComponentResource = WasmResource<TextComponent>;
pub type CommandResource = WasmResource<CommandTree>;
pub type CommandSenderResource = WasmResource<CommandSender>;
pub type ConsumedArgsResource = WasmResource<OwnedConsumedArgs>;
pub type CommandNodeResource = WasmResource<NonLeafNodeBuilder>;
pub type ItemStackResource = WasmResource<Arc<Mutex<pumpkin_data::item_stack::ItemStack>>>;
pub type RecipeManagerResource = WasmResource<Arc<RecipeManager>>;
pub type BlockEntityResource = WasmResource<Arc<dyn crate::block::entities::BlockEntity>>;

pub type OwnedConsumedArgs = HashMap<String, OwnedArg>;

pub struct PluginHostState {
    pub wasi_ctx: WasiCtx,
    pub wasi_http_ctx: WasiHttpCtx,
    pub wasi_http_hooks: PluginHttpHooks,
    pub resource_table: ResourceTable,
    pub plugin: Option<Weak<WasmPlugin>>,
    pub server: Option<Arc<Server>>,
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
            wasi_http_ctx: WasiHttpCtx::new(),
            wasi_http_hooks: PluginHttpHooks::new(),
            resource_table,
            plugin: None,
            server: None,
            permissions: Vec::new(),
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

    pub fn add_java_player<T>(
        &mut self,
        provider: Arc<Player>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(JavaPlayerResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_bedrock_player<T>(
        &mut self,
        provider: Arc<Player>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self
            .resource_table
            .push(BedrockPlayerResource { provider })?;
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

    pub fn add_item_stack<T>(
        &mut self,
        provider: Arc<Mutex<pumpkin_data::item_stack::ItemStack>>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(ItemStackResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_recipe_manager<T>(
        &mut self,
        provider: Arc<RecipeManager>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self
            .resource_table
            .push(RecipeManagerResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_block_entity<T>(
        &mut self,
        provider: Arc<dyn crate::block::entities::BlockEntity>,
    ) -> wasmtime::Result<wasmtime::component::Resource<T>> {
        let resource = self.resource_table.push(BlockEntityResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }
}

pub struct PluginHttpHooks {
    pub allow_outbound: bool,
}

impl PluginHttpHooks {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            allow_outbound: false,
        }
    }
}

impl Default for PluginHttpHooks {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiHttpHooks for PluginHttpHooks {
    fn send_request(
        &mut self,
        request: hyper::Request<wasmtime_wasi_http::p2::body::HyperOutgoingBody>,
        config: wasmtime_wasi_http::p2::types::OutgoingRequestConfig,
    ) -> HttpResult<wasmtime_wasi_http::p2::types::HostFutureIncomingResponse> {
        if !self.allow_outbound {
            return Err(HttpError::from(ErrorCode::HttpRequestDenied));
        }

        Ok(default_send_request(request, config))
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

impl WasiHttpView for PluginHostState {
    fn http(&mut self) -> WasiHttpCtxView<'_> {
        WasiHttpCtxView {
            ctx: &mut self.wasi_http_ctx,
            table: &mut self.resource_table,
            hooks: &mut self.wasi_http_hooks,
        }
    }
}
