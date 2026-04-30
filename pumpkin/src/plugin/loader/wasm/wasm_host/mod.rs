use std::{fs, path::Path, sync::Arc};
use thiserror::Error;
use tokio::sync::Mutex;
use wasmtime::{Cache, CacheConfig, Engine, Store, component::Component, component::Linker};
use wasmtime_wasi::{WasiCtxBuilder, sockets::SocketAddrUse};

use crate::plugin::{
    Context, PluginMetadata, loader::wasm::wasm_host::state::PluginHostState, permissions,
};

pub mod args;
pub mod logging;
pub mod state;
pub mod wit;

#[derive(Error, Debug)]
pub enum PluginInitError {
    #[error("Engine creation failed")]
    EngineCreationFailed(wasmtime::Error),
    #[error("Failed to setup linker")]
    LinkerSetupFailed(wasmtime::Error),
    #[error("plugin API version mismatch")]
    ApiVersionMismatch,
    #[error("failed to read payload for plugin")]
    FailedToReadPayload(#[from] wasmparser::BinaryReaderError),
    #[error("failed to read plugin bytes")]
    FailedToReadPluginBytes(#[from] std::io::Error),
    #[error("plugin failed to load with error: {0}")]
    PluginFailedToLoad(#[from] wasmtime::Error),
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
        let wasm_bytes = std::fs::read(&path)?;

        let component = load_component(&self.engine, &wasm_bytes, path.as_ref(), &self.cache_dir)?;

        let instance_pre = self.linker.instantiate_pre(&component)?;

        let (wasm_plugin, metadata) = {
            if let Ok(plugin_pre) = wit::v0_1::prepare_plugin(&instance_pre) {
                wit::v0_1::init_plugin(&self.engine, plugin_pre).await?
            } else {
                return Err(PluginInitError::ApiVersionMismatch);
            }
        };

        let wasm_plugin = Arc::new(wasm_plugin);
        wasm_plugin.store.lock().await.data_mut().plugin = Some(Arc::downgrade(&wasm_plugin));
        Ok((wasm_plugin, metadata))
    }
}

fn setup_linker(engine: &Engine) -> wasmtime::Result<Linker<PluginHostState>> {
    let mut linker = Linker::<PluginHostState>::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    wit::v0_1::add_to_linker(&mut linker)?;
    Ok(linker)
}

fn cache_key(wasm_path: &Path) -> Result<String, std::io::Error> {
    let metadata = fs::metadata(wasm_path)?;
    let file_name = wasm_path.file_stem().unwrap().to_string_lossy();
    let len = metadata.len();
    let modified = metadata
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(format!(
        "{file_name}-{len}-{modified}-{}.cwasm",
        env!("CARGO_PKG_VERSION"),
    ))
}

fn load_component(
    engine: &Engine,
    wasm_bytes: &[u8],
    wasm_path: &Path,
    cache_dir: &Path,
) -> Result<Component, PluginInitError> {
    let cache_name = cache_key(wasm_path)?;
    let cache_path = cache_dir.join(cache_name);

    if cache_path.exists() {
        match unsafe { Component::deserialize_file(engine, &cache_path) } {
            Ok(component) => return Ok(component),
            Err(_) => {
                let _ = fs::remove_file(&cache_path);
            }
        }
    }

    let component = Component::new(engine, wasm_bytes)?;
    fs::write(&cache_path, component.serialize()?)?;
    Ok(component)
}

impl WasmPlugin {
    pub async fn on_load(
        &self,
        context: Arc<Context>,
    ) -> Result<Result<(), String>, wasmtime::Error> {
        let mut store = self.store.lock().await;

        let mut builder = WasiCtxBuilder::new();

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

        let data_folder = context.get_data_folder();
        let preopen_path =
            if has_permission(permissions::FS_READ) || has_permission(permissions::FS_WRITE) {
                Path::new(".")
            } else {
                data_folder.as_path()
            };

        // Determine permissions for the preopened directory
        let (dir_perms, file_perms) = if has_permission(permissions::FS_WRITE) {
            (
                wasmtime_wasi::DirPerms::all(),
                wasmtime_wasi::FilePerms::all(),
            )
        } else if has_permission(permissions::FS_READ) {
            (
                wasmtime_wasi::DirPerms::READ,
                wasmtime_wasi::FilePerms::READ,
            )
        } else {
            // Scoped to data folder
            let can_write = has_permission(permissions::FS_WRITE_DATA);
            if can_write {
                (
                    wasmtime_wasi::DirPerms::all(),
                    wasmtime_wasi::FilePerms::all(),
                )
            } else {
                // Default to READ if no write permission is given for data folder
                // (Plugins should at least be able to read their own config)
                (
                    wasmtime_wasi::DirPerms::READ,
                    wasmtime_wasi::FilePerms::READ,
                )
            }
        };

        builder.preopened_dir(
            preopen_path,
            preopen_path.to_string_lossy(),
            dir_perms,
            file_perms,
        )?;

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
