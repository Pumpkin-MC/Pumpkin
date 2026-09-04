use std::{
    future::Future,
    sync::{Arc, Weak},
};

use wasmtime::component::{Resource, ResourceTable};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
use wasmtime_wasi_http::{
    RequestOptions, WasiBody, WasiHttpCtx, WasiHttpCtxView, WasiHttpHooks, WasiHttpView,
};

use crate::{
    entity::player::Player,
    plugin::
        loader::wasm::wasm_host::WasmPlugin
    ,
    server::Server,
    world::World,
};

pub struct WasmCommand {
    pub names: Vec<String>,
    pub builder: crate::command::argument_builder::CommandArgumentBuilder,
}

impl WasmCommand {
    #[must_use]
    pub fn new(names: Vec<String>, description: String) -> Self {
        let primary = names.first().cloned().unwrap_or_default();
        let builder = crate::command::argument_builder::command(primary, description);
        Self { names, builder }
    }

    #[must_use]
    pub fn then(mut self, child: WasmCommandNode) -> Self {
        use crate::command::argument_builder::ArgumentBuilder;
        self.builder = self.builder.then(child.into_detached_node());
        self
    }

    #[must_use]
    pub fn executes(
        mut self,
        executor: impl crate::command::node::CommandExecutor + 'static,
    ) -> Self {
        use crate::command::argument_builder::ArgumentBuilder;
        self.builder = self.builder.executes(executor);
        self
    }
}

pub enum WasmCommandNode {
    Literal(crate::command::argument_builder::LiteralArgumentBuilder),
    Argument(crate::command::argument_builder::RequiredArgumentBuilder),
}

impl WasmCommandNode {
    #[must_use]
    pub fn then(self, child: Self) -> Self {
        use crate::command::argument_builder::ArgumentBuilder;
        match self {
            Self::Literal(b) => Self::Literal(b.then(child.into_detached_node())),
            Self::Argument(b) => Self::Argument(b.then(child.into_detached_node())),
        }
    }

    #[must_use]
    pub fn executes(self, executor: impl crate::command::node::CommandExecutor + 'static) -> Self {
        use crate::command::argument_builder::ArgumentBuilder;
        match self {
            Self::Literal(b) => Self::Literal(b.executes(executor)),
            Self::Argument(b) => Self::Argument(b.executes(executor)),
        }
    }

    #[must_use]
    pub fn suggests(
        self,
        provider: impl crate::command::suggestion::provider::SuggestionProvider + 'static,
    ) -> Self {
        match self {
            Self::Literal(b) => Self::Literal(b),
            Self::Argument(b) => Self::Argument(b.suggests(provider)),
        }
    }

    #[must_use]
    pub fn into_detached_node(self) -> crate::command::node::detached::DetachedNode {
        use crate::command::argument_builder::ArgumentBuilder;
        match self {
            Self::Literal(b) => crate::command::node::detached::DetachedNode::Literal(b.build()),
            Self::Argument(b) => crate::command::node::detached::DetachedNode::Argument(b.build()),
        }
    }
}

#[derive(Clone)]
pub enum ScoreboardProvider {
    World(Arc<World>),
    Player(Arc<Player>),
}

#[derive(Clone)]
pub enum InventoryProvider {
    Generic(Arc<dyn pumpkin_world::inventory::Inventory>),
    PlayerMain(Arc<Player>),
    PlayerEnderChest(Arc<Player>),
}

#[derive(Clone)]
pub struct ContainerBlockEntity {
    pub provider: Arc<dyn crate::block::entities::BlockEntity>,
    pub inventory: Arc<dyn pumpkin_world::inventory::Inventory>,
}

#[derive(Clone)]
pub struct ChunkBuffer {
    pub x: i32,
    pub z: i32,
    pub min_y: i32,
    pub height: u32,
    pub proto_chunk: Arc<std::sync::Mutex<pumpkin_world::ProtoChunk>>,
}

pub struct PluginHostState {
    pub wasi_ctx: WasiCtx,
    pub wasi_http_ctx: WasiHttpCtx,
    pub wasi_http_hooks: PluginHttpHooks,
    pub resource_table: ResourceTable,
    pub limits: wasmtime::StoreLimits,
    pub plugin: Option<Weak<WasmPlugin>>,
    pub server: Option<Arc<Server>>,
    pub permissions: Vec<String>,
    pub name: Option<String>,
    pub marketplace_metadata:
        Option<crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::context::MarketplaceMetadata>,
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
            wasi_ctx: WasiCtxBuilder::new()
                .inherit_stdout() // allow messages & errors to be printed
                .inherit_stderr() // before `on_load`, e.g. during metadata retrieval
                .build(),
            wasi_http_ctx: WasiHttpCtx::new(),
            wasi_http_hooks: PluginHttpHooks::new(),
            resource_table,
            limits: wasmtime::StoreLimitsBuilder::new().build(),
            plugin: None,
            server: None,
            permissions: Vec::new(),
            name: None,
            marketplace_metadata: None,
        }
    }

    /// get a **borrowed** (or **owned**) [`Resource`]'s value from the resource table
    pub fn get<T: FromResource>(&self, res: &Resource<T>) -> wasmtime::Result<&T::Internal> {
        Ok(self.resource_table.get(&Resource::new_borrow(res.rep()))?)
    }

    pub fn get_mut<T: FromResource>(
        &mut self,
        res: &Resource<T>,
    ) -> wasmtime::Result<&mut T::Internal> {
        Ok(self
            .resource_table
            .get_mut(&Resource::new_borrow(res.rep()))?)
    }

    /// always returns an owned [`Resource`]
    pub fn add<T: FromResource>(&mut self, item: T::Internal) -> wasmtime::Result<Resource<T>> {
        Ok(Resource::new_own(self.resource_table.push(item)?.rep()))
    }

    /// take the value of an **owned** [`Resource`] from the resource table
    pub fn take<T: FromResource>(&mut self, res: Resource<T>) -> wasmtime::Result<T::Internal> {
        Ok(self.resource_table.delete(Resource::new_own(res.rep()))?)
    }

    /// requires an **owned** [`Resource`]
    pub fn drop<T: FromResource>(&mut self, res: Resource<T>) -> wasmtime::Result<()> {
        self.take(res)?;
        Ok(())
    }

    pub fn discard<T: FromResource>(&mut self, res: Resource<T>) {
        let _ = self.drop(res);
    }

    pub fn discard_to_be_removed<T: FromResource>(&mut self, res: &Resource<T>) {
        assert!(res.owned());
        let _ = self.resource_table.delete::<T::Internal>(Resource::new_own(res.rep()));
    }
}

/// This trait could also be replaced with a bunch of `with: {}` definitions in the-
/// bindgen! macro. That would however enforce that said macro stays in this crate.
pub trait FromResource: Sized + 'static
where
    Resource<Self>: Send + Sync + 'static,
{
    type Internal: Send + Sync + 'static;
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
        request: hyper::Request<WasiBody>,
        options: Option<RequestOptions>,
        fut: Box<dyn Future<Output = wasmtime_wasi_http::Result<()>> + Send>,
    ) -> Box<
        dyn Future<
                Output = wasmtime_wasi_http::Result<(
                    hyper::Response<WasiBody>,
                    Box<dyn Future<Output = wasmtime_wasi_http::Result<()>> + Send>,
                )>,
            > + Send,
    > {
        if !self.allow_outbound {
            return Box::new(async { Err(wasmtime_wasi_http::Error::HttpRequestDenied) });
        }

        wasmtime_wasi_http::default_hooks().send_request(request, options, fut)
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
