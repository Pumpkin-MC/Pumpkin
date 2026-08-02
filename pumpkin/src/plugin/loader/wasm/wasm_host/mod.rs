use std::{fs, path::Path, sync::Arc};
use thiserror::Error;
use tokio::sync::Mutex;
use wasmtime::{Cache, CacheConfig, Engine, Store, component::Component, component::Linker};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtxBuilder, sockets::SocketAddrUse};

use crate::plugin::{
    Context, PluginMetadata, cache::calculate_hash_for_bytes,
    loader::wasm::wasm_host::state::PluginHostState, permissions,
};

pub mod args;
pub mod logging;
pub mod signature;
pub mod state;
pub mod wit;

#[derive(Error, Debug)]
pub enum PluginInitError {
    #[error("Engine creation failed: {0}")]
    EngineCreationFailed(wasmtime::Error),
    #[error("Failed to setup linker: {0}")]
    LinkerSetupFailed(wasmtime::Error),
    #[error("Plugin is built against a different API version: {0}")]
    ApiVersionMismatch(wasmtime::Error),
    #[error("Failed to read plugin file: {0}")]
    FileReadFailed(std::io::Error),
    #[error("Failed to load plugin as component: {0}")]
    ComponentNewFailed(wasmtime::Error),
    #[error("Failed to create cache data for plugin: {0}")]
    ComponentCacheSerializeFailed(wasmtime::Error),
    #[error("Failed to write cache file for plugin: {0}")]
    ComponentCacheWriteFailed(std::io::Error),
    #[error("Failed to instantiate plugin: {0}")]
    InstantiationFailed(wasmtime::Error),
    #[error("Calling `init_plugin` failed: {0}")]
    CallInitPluginFailed(wasmtime::Error),
    #[error("Calling `get_metadata` failed: {0}")]
    CallGetMetadataFailed(wasmtime::Error),
}

pub struct PluginRuntime {
    engine: Engine,
    cache_dir: std::path::PathBuf,
    linker: wasmtime::component::Linker<PluginHostState>,
}

pub enum PluginInstance {
    V0_1(wit::v0_1::Plugin),
}

pub struct WasmPlugin {
    pub plugin_instance: PluginInstance,
    pub store: Mutex<Store<PluginHostState>>,
}

impl PluginRuntime {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, PluginInitError> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        let mut path = std::path::absolute(path.as_ref()).expect("Failed to get absolute path");
        path.pop();
        path.push("cache");
        let mut cache_config = CacheConfig::new();
        cache_config.with_directory(&path);
        config.cache(Some(
            Cache::new(cache_config).expect("Failed to create cache"),
        ));

        config.gc_support(true);
        config.wasm_gc(true);
        config.wasm_exceptions(true);
        config.wasm_function_references(true);

        let engine = Engine::new(&config).map_err(PluginInitError::EngineCreationFailed)?;

        let linker = setup_linker(&engine).map_err(PluginInitError::LinkerSetupFailed)?;

        Ok(Self {
            engine,
            cache_dir: path,
            linker,
        })
    }

    pub async fn init_plugin<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(Arc<WasmPlugin>, PluginMetadata), PluginInitError> {
        let wasm_bytes = std::fs::read(&path).map_err(PluginInitError::FileReadFailed)?;

        signature::verify_wasm_plugin(&wasm_bytes, &path.as_ref().to_string_lossy());

        let wasm_bytes = signature::strip_pumpkin_sections(&wasm_bytes).unwrap_or(wasm_bytes);

        let component = load_component(&self.engine, &wasm_bytes, &self.cache_dir)?;

        let instance_pre = self
            .linker
            .instantiate_pre(&component)
            .map_err(PluginInitError::ApiVersionMismatch)?;

        let (wasm_plugin, metadata) = {
            let plugin_pre = wit::v0_1::prepare_plugin(&instance_pre)
                .map_err(PluginInitError::ApiVersionMismatch)?;

            wit::v0_1::init_plugin(&self.engine, plugin_pre).await?
        };

        let wasm_plugin = Arc::new(wasm_plugin);
        wasm_plugin.store.lock().await.data_mut().plugin = Some(Arc::downgrade(&wasm_plugin));
        Ok((wasm_plugin, metadata))
    }
}

fn setup_linker(engine: &Engine) -> wasmtime::Result<Linker<PluginHostState>> {
    let mut linker = Linker::<PluginHostState>::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_http::p2::add_only_http_to_linker_async(&mut linker)?;
    wit::v0_1::add_to_linker(&mut linker)?;
    Ok(linker)
}

fn load_component(
    engine: &Engine,
    wasm_bytes: &[u8],
    cache_dir: &Path,
) -> Result<Component, PluginInitError> {
    let hash = calculate_hash_for_bytes(wasm_bytes);
    let cache_name = format!("{hash}-{}.cwasm", env!("CARGO_PKG_VERSION"));
    let cache_path = cache_dir.join(cache_name);

    if cache_path.exists() {
        match unsafe { Component::deserialize_file(engine, &cache_path) } {
            Ok(component) => return Ok(component),
            Err(_) => {
                let _ = fs::remove_file(&cache_path);
            }
        }
    }

    let component =
        Component::new(engine, wasm_bytes).map_err(PluginInitError::ComponentNewFailed)?;
    fs::write(
        &cache_path,
        component
            .serialize()
            .map_err(PluginInitError::ComponentCacheSerializeFailed)?,
    )
    .map_err(PluginInitError::ComponentCacheWriteFailed)?;
    Ok(component)
}

impl WasmPlugin {
    pub async fn on_load(
        &self,
        context: Arc<Context>,
    ) -> Result<Result<(), String>, wasmtime::Error> {
        let mut store = self.store.lock().await;

        let mut builder = WasiCtxBuilder::new();

        builder.inherit_stdout();
        builder.inherit_stderr();

        let metadata = context.get_metadata();
        let blocked_permissions = &context.server.advanced_config.plugins.blocked_permissions;

        let filtered_permissions: Vec<String> = metadata
            .permissions
            .iter()
            .filter(|p| !blocked_permissions.iter().any(|blocked| blocked == *p))
            .cloned()
            .collect();

        let has_permission = |p: &str| filtered_permissions.iter().any(|perm| perm == p);

        if has_permission(permissions::NETWORK_DNS) {
            builder.allow_ip_name_lookup(true);
        }

        let tcp_allowed = has_permission(permissions::NETWORK_TCP);
        let udp_allowed = has_permission(permissions::NETWORK_UDP);
        let tcp_connect = tcp_allowed || has_permission(permissions::NETWORK_TCP_CONNECT);
        let tcp_bind = tcp_allowed || has_permission(permissions::NETWORK_TCP_BIND);
        let udp_connect = udp_allowed || has_permission(permissions::NETWORK_UDP_CONNECT);
        let udp_bind = udp_allowed || has_permission(permissions::NETWORK_UDP_BIND);
        let udp_outgoing_datagram =
            udp_allowed || has_permission(permissions::NETWORK_UDP_OUTGOING_DATAGRAM);

        let loopback_only = has_permission(permissions::NETWORK_LOOPBACK);

        builder.allow_tcp(tcp_connect || tcp_bind);
        builder.allow_udp(udp_connect || udp_bind);

        builder.socket_addr_check(move |addr, reason| {
            Box::pin(async move {
                let ok = match reason {
                    SocketAddrUse::TcpConnect => tcp_connect,
                    SocketAddrUse::TcpBind => tcp_bind,
                    SocketAddrUse::UdpConnect => udp_connect,
                    SocketAddrUse::UdpBind => udp_bind,
                    SocketAddrUse::UdpOutgoingDatagram => udp_outgoing_datagram,
                };

                if loopback_only {
                    ok && addr.ip().is_loopback()
                } else {
                    ok
                }
            })
        });

        if has_permission(permissions::NETWORK_OUTBOUND) {
            builder.inherit_network();
        }

        // --- System Permissions ---

        // Environment Variables
        if has_permission(permissions::SYS_ENV) {
            builder.inherit_env();
        } else {
            for (key, value) in std::env::vars() {
                let perm = format!("{}{}", permissions::SYS_ENV_PREFIX, key);
                if has_permission(&perm) {
                    builder.env(key, value);
                }
            }
        }

        builder.preopened_dir(
            context.get_data_folder(),
            "data",
            if has_permission(permissions::FS_READ_DATA)
                || has_permission(permissions::FS_WRITE_DATA)
            {
                DirPerms::READ
            } else {
                DirPerms::empty()
            } | if has_permission(permissions::FS_WRITE_DATA) {
                DirPerms::MUTATE
            } else {
                DirPerms::empty()
            },
            if has_permission(permissions::FS_READ_DATA) {
                FilePerms::READ
            } else {
                FilePerms::empty()
            } | if has_permission(permissions::FS_WRITE_DATA) {
                FilePerms::WRITE
            } else {
                FilePerms::empty()
            },
        )?;

        if has_permission(permissions::HTTP_OUTBOUND) {
            store.data_mut().wasi_http_hooks.allow_outbound = true;
        }

        store.data_mut().permissions = filtered_permissions;
        store.data_mut().wasi_ctx = builder.build();

        store.data_mut().server = Some(context.server.clone());

        match self.plugin_instance {
            PluginInstance::V0_1(ref plugin) => {
                let context = store.data_mut().add_context(context)?;
                plugin.call_on_load(&mut *store, context).await
            }
        }
    }

    pub async fn on_unload(
        &self,
        context: Arc<Context>,
    ) -> Result<Result<(), String>, wasmtime::Error> {
        let mut store = self.store.lock().await;

        if let Some(weak_plugin) = &store.data().plugin
            && let Some(plugin) = weak_plugin.upgrade()
        {
            context
                .server
                .task_scheduler
                .cancel_all_tasks(&plugin)
                .await;
        }

        match self.plugin_instance {
            PluginInstance::V0_1(ref plugin) => {
                let context = store.data_mut().add_context(context)?;
                plugin.call_on_unload(&mut *store, context).await
            }
        }
    }
}

pub trait DowncastResourceExt<E> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a E;
    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut E;
    fn consume(self, state: &mut PluginHostState) -> E;
}
