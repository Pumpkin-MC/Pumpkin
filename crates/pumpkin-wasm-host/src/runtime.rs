#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::{any::Any, fs, net::SocketAddr, path::Path, sync::Arc};
use wasmtime::{Cache, CacheConfig, Engine, component::Component, component::Linker};
use wasmtime_wasi::{FsPerms, WasiCtxBuilder, sockets::SocketAddrUse};

use pumpkin_core::plugin::{Context, PluginMetadata, cache::calculate_hash_for_bytes, permissions};
use pumpkin_plugin_runtime::RuntimeSpawner;
use pumpkin_wasm_host_common::{
    concurrent_store::{self, LegacySyncReentry, StoreMode},
    plugin::{PluginApiVersion, PluginInitError, WasmPlugin, scan_component_names},
    scheduler::TaskScheduler,
    signature,
    state::PluginHostState,
};

#[derive(Clone, Copy, Default)]
struct SocketPolicy {
    tcp_connect: bool,
    tcp_bind: bool,
    udp_send: bool,
    udp_receive: bool,
    udp_bind: bool,
    loopback_only: bool,
}

impl SocketPolicy {
    const fn allows(self, addr: SocketAddr, reason: SocketAddrUse) -> bool {
        // Wasmtime performs a wildcard bind before an outbound connect/send.
        // Permit only that precursor here; the later destination check still
        // enforces the actual protocol and loopback policy.
        let implicit_outbound_bind = addr.ip().is_unspecified()
            && addr.port() == 0
            && match reason {
                SocketAddrUse::TcpBind => self.tcp_connect,
                SocketAddrUse::UdpBind => self.udp_send,
                _ => false,
            };

        let operation_allowed = match reason {
            SocketAddrUse::TcpConnect => self.tcp_connect,
            SocketAddrUse::TcpBind => self.tcp_bind || implicit_outbound_bind,
            SocketAddrUse::TcpListen | SocketAddrUse::TcpAccept => self.tcp_bind,
            SocketAddrUse::UdpBind => self.udp_bind || implicit_outbound_bind,
            SocketAddrUse::UdpSend => self.udp_send,
            SocketAddrUse::UdpReceive => self.udp_receive,
        };

        operation_allowed
            && (!self.loopback_only || addr.ip().is_loopback() || implicit_outbound_bind)
    }
}

fn socket_policy_for_permissions(
    has_permission: impl Fn(&str) -> bool,
    loopback_only: bool,
) -> SocketPolicy {
    let network_outbound = has_permission(permissions::NETWORK_OUTBOUND);
    let tcp_allowed = has_permission(permissions::NETWORK_TCP);
    let udp_allowed = has_permission(permissions::NETWORK_UDP);
    let tcp_connect =
        tcp_allowed || network_outbound || has_permission(permissions::NETWORK_TCP_CONNECT);
    let tcp_bind = tcp_allowed || has_permission(permissions::NETWORK_TCP_BIND);
    let udp_connected =
        udp_allowed || network_outbound || has_permission(permissions::NETWORK_UDP_CONNECT);
    let udp_bind = udp_allowed || has_permission(permissions::NETWORK_UDP_BIND);
    let udp_send = udp_connected || has_permission(permissions::NETWORK_UDP_OUTGOING_DATAGRAM);
    let udp_receive = udp_connected || udp_bind;

    SocketPolicy {
        tcp_connect,
        tcp_bind,
        udp_send,
        udp_receive,
        udp_bind,
        loopback_only,
    }
}

/// Wasmtime only lets a host import call back into the guest of the store it is running on while
/// concurrency support is off, and only runs async components and WASI 0.3 while it is on. Each API
/// version gets the engine that suits its host imports: v0.1 imports call back into the guest,
/// v0.2 imports are concurrent.
pub struct PluginRuntime {
    engine_v0_1: Engine,
    #[cfg(feature = "v0_2")]
    engine_v0_2: Engine,
    cache_dir: std::path::PathBuf,
    linker_v0_1: wasmtime::component::Linker<PluginHostState>,
    #[cfg(feature = "v0_2")]
    linker_v0_2: wasmtime::component::Linker<PluginHostState>,
    legacy_sync_reentry: LegacySyncReentry,
    task_scheduler: Arc<TaskScheduler>,
    store_spawner: Arc<dyn RuntimeSpawner>,
}

impl PluginRuntime {
    pub fn new<P: AsRef<Path>>(
        path: P,
        legacy_sync_reentry: LegacySyncReentry,
        task_scheduler: Arc<TaskScheduler>,
        store_spawner: Arc<dyn RuntimeSpawner>,
    ) -> Result<Self, PluginInitError> {
        let mut path =
            std::path::absolute(path.as_ref()).map_err(PluginInitError::PathResolutionFailed)?;
        path.pop();
        path.push("cache");

        let engine_v0_1 = new_engine(&path, StoreMode::Sequential)?;
        #[cfg(feature = "v0_2")]
        let engine_v0_2 = new_engine(&path, StoreMode::Concurrent)?;

        let linker_v0_1 =
            setup_linker_v0_1(&engine_v0_1).map_err(PluginInitError::LinkerSetupFailed)?;
        #[cfg(feature = "v0_2")]
        let linker_v0_2 =
            setup_linker_v0_2(&engine_v0_2).map_err(PluginInitError::LinkerSetupFailed)?;

        Ok(Self {
            engine_v0_1,
            #[cfg(feature = "v0_2")]
            engine_v0_2,
            cache_dir: path,
            linker_v0_1,
            #[cfg(feature = "v0_2")]
            linker_v0_2,
            legacy_sync_reentry,
            task_scheduler,
            store_spawner,
        })
    }

    const fn engine(&self, api_version: PluginApiVersion) -> &Engine {
        match api_version {
            PluginApiVersion::V0_1 => &self.engine_v0_1,
            #[cfg(feature = "v0_2")]
            PluginApiVersion::V0_2 => &self.engine_v0_2,
            #[cfg(not(feature = "v0_2"))]
            PluginApiVersion::V0_2 => &self.engine_v0_1,
        }
    }

    pub async fn init_plugin<P: AsRef<Path>>(
        &self,
        path: P,
        verify_signatures: bool,
    ) -> Result<(Arc<WasmPlugin>, PluginMetadata), PluginInitError> {
        let wasm_bytes = std::fs::read(&path).map_err(PluginInitError::FileReadFailed)?;
        let marketplace_metadata = maybe_verify_wasm_plugin(
            &wasm_bytes,
            &path.as_ref().to_string_lossy(),
            verify_signatures,
            |bytes, path_str| {
                let verification = signature::verify_wasm_plugin(bytes, path_str);
                if verification.is_signed && verification.is_valid {
                    verification.metadata
                } else {
                    None
                }
            },
        )
        .flatten();

        let wasm_bytes = signature::strip_pumpkin_sections(&wasm_bytes).unwrap_or(wasm_bytes);

        let api_version = plugin_api_version(&wasm_bytes)?;
        let engine = self.engine(api_version);
        let component = load_component(engine, &wasm_bytes, &self.cache_dir)?;

        let (plugin_instance, store, metadata) = match api_version {
            PluginApiVersion::V0_1 => {
                let instance_pre = self
                    .linker_v0_1
                    .instantiate_pre(&component)
                    .map_err(PluginInitError::ApiMismatch)?;

                let plugin_pre = pumpkin_wasm_host_v0_1::prepare_plugin(&instance_pre)
                    .map_err(PluginInitError::ApiMismatch)?;

                let (plugin, store, metadata) = pumpkin_wasm_host_v0_1::init_plugin(
                    engine,
                    plugin_pre,
                    &self.legacy_sync_reentry,
                )
                .await?;
                (
                    Arc::new(plugin) as Arc<dyn Any + Send + Sync>,
                    store,
                    metadata,
                )
            }
            #[cfg(feature = "v0_2")]
            PluginApiVersion::V0_2 => {
                let instance_pre = self
                    .linker_v0_2
                    .instantiate_pre(&component)
                    .map_err(PluginInitError::ApiMismatch)?;

                let plugin_pre = pumpkin_wasm_host_v0_2::prepare_plugin(&instance_pre)
                    .map_err(PluginInitError::ApiMismatch)?;

                let (plugin, store, metadata) = pumpkin_wasm_host_v0_2::init_plugin(
                    engine,
                    plugin_pre,
                    &self.legacy_sync_reentry,
                )
                .await?;
                (
                    Arc::new(plugin) as Arc<dyn Any + Send + Sync>,
                    store,
                    metadata,
                )
            }
            #[cfg(not(feature = "v0_2"))]
            PluginApiVersion::V0_2 => return Err(PluginInitError::ApiMismatch(v0_2_disabled())),
        };

        let store = concurrent_store::start_legacy_store(
            store,
            self.legacy_sync_reentry.clone(),
            Arc::clone(&self.store_spawner),
            match api_version {
                PluginApiVersion::V0_1 => StoreMode::Sequential,
                PluginApiVersion::V0_2 => StoreMode::Concurrent,
            },
        )
        .await
        .map_err(PluginInitError::InstantiationFailed)?;
        let wasm_plugin = Arc::new(WasmPlugin {
            plugin_instance,
            api_version,
            store,
            task_scheduler: Arc::clone(&self.task_scheduler),
        });
        let weak_plugin = Arc::downgrade(&wasm_plugin);
        wasm_plugin
            .store
            .call(move |state| {
                state.plugin = Some(weak_plugin);
                state.marketplace_metadata = marketplace_metadata;
            })
            .await
            .map_err(PluginInitError::InstantiationFailed)?;

        tracing::debug!(
            wasm_plugin_api = match api_version {
                PluginApiVersion::V0_1 => "0.1",
                PluginApiVersion::V0_2 => "0.2",
            },
            wasm_plugin_policy = LegacySyncReentry::NAME,
            "Loaded Wasm plugin with synchronous compatibility policy"
        );

        Ok((wasm_plugin, metadata))
    }
}

fn maybe_verify_wasm_plugin<T, F>(
    wasm_bytes: &[u8],
    path_str: &str,
    verify_signatures: bool,
    verify: F,
) -> Option<T>
where
    F: FnOnce(&[u8], &str) -> T,
{
    verify_signatures.then(|| verify(wasm_bytes, path_str))
}

/// Reads the plugin's API version from its bytes, before choosing the engine to compile it on.
fn plugin_api_version(wasm_bytes: &[u8]) -> Result<PluginApiVersion, PluginInitError> {
    let names = scan_component_names(wasm_bytes)?;
    let Some(version) = names.metadata_version else {
        return Err(PluginInitError::UnknownApiVersion());
    };
    // `starts_with` to ignore "minor" version differences
    // WIT enforces semver. Will always be major.minor.patch
    // For early/unstable release: 0.major.minor
    if version.starts_with("0.1.") {
        // WASI 0.3 needs concurrency support, which the v0.1 engine doesn't have, so the linker
        // would report this as an API mismatch.
        if let Some(import) = names.wasi_p3_import {
            return Err(PluginInitError::UnsupportedWasi(import));
        }
        Ok(PluginApiVersion::V0_1)
    } else if version.starts_with("0.2.") {
        v0_2_api_version(&version)
    } else {
        Err(PluginInitError::UnsupportedApiVersion(version))
    }
}

/// Builds the engine for one store mode, with the settings every plugin engine shares.
fn new_engine(cache_dir: &Path, mode: StoreMode) -> Result<Engine, PluginInitError> {
    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);
    if mode == StoreMode::Concurrent {
        config.wasm_component_model_async(true);
    }
    config.concurrency_support(mode == StoreMode::Concurrent);

    let mut cache_config = CacheConfig::new();
    cache_config.with_directory(cache_dir);
    let cache = Cache::new(cache_config).map_err(PluginInitError::CacheCreationFailed)?;
    config.cache(Some(cache));

    config.gc_support(true);
    config.wasm_gc(true);
    config.wasm_exceptions(true);
    config.wasm_function_references(true);

    Engine::new(&config).map_err(PluginInitError::EngineCreationFailed)
}

fn setup_linker_v0_1(engine: &Engine) -> wasmtime::Result<Linker<PluginHostState>> {
    let mut linker = Linker::<PluginHostState>::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_http::p2::add_only_http_to_linker_async(&mut linker)?;
    // WASI 0.3 is left out: its host functions are concurrent, and the v0.1 engine has
    // concurrency support off.
    pumpkin_wasm_host_v0_1::add_to_linker(&mut linker)?;
    Ok(linker)
}

#[cfg(feature = "v0_2")]
#[allow(clippy::unnecessary_wraps)] // same signature as the build without the feature
const fn v0_2_api_version(_version: &str) -> Result<PluginApiVersion, PluginInitError> {
    Ok(PluginApiVersion::V0_2)
}

#[cfg(not(feature = "v0_2"))]
fn v0_2_api_version(version: &str) -> Result<PluginApiVersion, PluginInitError> {
    Err(PluginInitError::UnsupportedApiVersion(format!(
        "{version} (this server was built without the `plugin-v0_2` feature)"
    )))
}

/// The error for the `V0_2` arms when the `v0_2` feature is off. `plugin_api_version` already
/// rejects v0.2 plugins at load time, so these arms are only a fallback.
#[cfg(not(feature = "v0_2"))]
pub fn v0_2_disabled() -> wasmtime::Error {
    wasmtime::Error::msg("this server was built without the `plugin-v0_2` feature")
}

#[cfg(feature = "v0_2")]
fn setup_linker_v0_2(engine: &Engine) -> wasmtime::Result<Linker<PluginHostState>> {
    let mut linker = Linker::<PluginHostState>::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_http::p2::add_only_http_to_linker_async(&mut linker)?;
    wasmtime_wasi::p3::add_to_linker(&mut linker)?;
    wasmtime_wasi_http::p3::add_to_linker(&mut linker)?;
    pumpkin_wasm_host_v0_2::add_to_linker(&mut linker)?;
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
        // SAFETY: Cache file was generated by Wasmtime for this exact WASM binary hash and package version.
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

pub fn guest_exports<T: 'static>(instance: &Arc<dyn Any + Send + Sync>) -> wasmtime::Result<&T> {
    instance
        .downcast_ref()
        .ok_or_else(|| wasmtime::Error::msg("Plugin exports do not match its API version"))
}

#[allow(clippy::too_many_lines)]
pub async fn on_load(
    wasm_plugin: &WasmPlugin,
    context: Arc<Context>,
) -> Result<Result<(), String>, wasmtime::Error> {
    let mut builder = WasiCtxBuilder::new();

    builder.inherit_stdout();
    builder.inherit_stderr();

    let metadata = context.get_metadata();
    let plugin_config = &context.server.advanced_config.plugins;
    let plugin_override = plugin_config.overrides.get(&metadata.name);

    let is_blocked = |p: &str| {
        plugin_config.blocked_permissions.iter().any(|b| b == p)
            || plugin_override.is_some_and(|o| o.blocked_permissions.iter().any(|b| b == p))
    };

    let filtered_permissions: Vec<String> = metadata
        .permissions
        .iter()
        .filter(|p| !is_blocked(p))
        .cloned()
        .collect();

    let has_permission = |p: &str| filtered_permissions.iter().any(|perm| perm == p);

    if has_permission(permissions::NETWORK_DNS) {
        builder.allow_ip_name_lookup(true);
    }

    let loopback_only = plugin_override
        .and_then(|o| o.loopback_only)
        .unwrap_or(plugin_config.loopback_only)
        || has_permission(permissions::NETWORK_LOOPBACK);

    let socket_policy = socket_policy_for_permissions(has_permission, loopback_only);

    builder.allow_tcp(socket_policy.tcp_connect || socket_policy.tcp_bind);
    builder
        .allow_udp(socket_policy.udp_send || socket_policy.udp_receive || socket_policy.udp_bind);
    builder.socket_addr_check(move |addr, reason| {
        Box::pin(async move { socket_policy.allows(addr, reason) })
    });

    let allow_http_outbound = has_permission(permissions::HTTP_OUTBOUND);

    // --- System Permissions & Environment Variables ---

    // Environment Variables
    if plugin_config.inherit_env || has_permission(permissions::SYS_ENV) {
        builder.inherit_env();
    } else {
        for (key, value) in std::env::vars() {
            let perm = format!("{}{}", permissions::SYS_ENV_PREFIX, key);
            if has_permission(&perm) {
                builder.env(key, value);
            }
        }
    }

    // Injected environment variables from plugin override
    if let Some(plugin_override) = plugin_override {
        for (key, value) in &plugin_override.environment {
            builder.env(key, value);
        }
    }

    let may_write_data = has_permission(permissions::FS_WRITE_DATA);
    let may_read_data = has_permission(permissions::FS_READ_DATA);
    if may_read_data || may_write_data {
        builder.preopened_dir(
            context.get_data_folder(),
            "data",
            if may_write_data {
                FsPerms::ReadWrite
            } else {
                FsPerms::ReadOnly
            },
        )?;
    }

    let max_memory_mb = plugin_override
        .and_then(|o| o.max_memory_mb)
        .or(plugin_config.max_memory_mb);
    let wasi_ctx = builder.build();
    let server = context.server.clone();
    let name = metadata.name.clone();

    let plugin_instance = wasm_plugin.plugin_instance.clone();
    let api_version = wasm_plugin.api_version;
    wasm_plugin
        .store
        .call_guest(move |mut guest| {
            Box::pin(async move {
                guest.with(|mut store| {
                    if let Some(mb) = max_memory_mb {
                        let limit_bytes = (mb as usize).saturating_mul(1024 * 1024);
                        store.data_mut().limits = wasmtime::StoreLimitsBuilder::new()
                            .memory_size(limit_bytes)
                            .build();
                    }

                    store.data_mut().permissions = filtered_permissions;
                    store.data_mut().wasi_ctx = wasi_ctx;
                    store.data_mut().wasi_http_hooks.allow_outbound = allow_http_outbound;
                    store.data_mut().server = Some(server);
                    store.data_mut().name = Some(name);
                });

                match api_version {
                    PluginApiVersion::V0_1 => {
                        let plugin =
                            guest_exports::<pumpkin_wasm_host_v0_1::Plugin>(&plugin_instance)?;
                        let context_res = guest.with(|mut store| store.data_mut().add(context))?;

                        guest
                            .call(plugin.func_on_load(), (context_res,))
                            .await
                            .map(|(result,)| result)
                    }
                    #[cfg(feature = "v0_2")]
                    PluginApiVersion::V0_2 => {
                        let plugin =
                            guest_exports::<pumpkin_wasm_host_v0_2::Plugin>(&plugin_instance)?;
                        let context_res = guest.with(|mut store| store.data_mut().add(context))?;

                        guest
                            .call(plugin.func_on_load(), (context_res,))
                            .await
                            .map(|(result,)| result)
                    }
                    #[cfg(not(feature = "v0_2"))]
                    PluginApiVersion::V0_2 => Err(v0_2_disabled()),
                }
            })
        })
        .await
}

pub async fn on_unload(
    wasm_plugin: &WasmPlugin,
    context: Arc<Context>,
) -> Result<Result<(), String>, wasmtime::Error> {
    let loaded_plugin = wasm_plugin
        .store
        .call(|state| state.plugin.as_ref().and_then(std::sync::Weak::upgrade))
        .await?;

    if let Some(plugin) = loaded_plugin {
        plugin.task_scheduler.disable_plugin(&plugin);
    }

    let plugin_instance = wasm_plugin.plugin_instance.clone();
    let api_version = wasm_plugin.api_version;
    wasm_plugin
        .store
        .shutdown(move |mut guest| {
            Box::pin(async move {
                match api_version {
                    PluginApiVersion::V0_1 => {
                        let plugin =
                            guest_exports::<pumpkin_wasm_host_v0_1::Plugin>(&plugin_instance)?;
                        let (context_res, context_rep) = guest.with(|mut store| {
                            let resource = store.data_mut().add(context)?;
                            let rep = resource.rep();
                            Ok::<_, wasmtime::Error>((resource, rep))
                        })?;

                        let result = guest
                            .call(plugin.func_on_unload(), (context_res,))
                            .await
                            .map(|(result,)| result);

                        guest.with(|mut store| {
                            let _ = store.data_mut().resource_table.delete::<Arc<Context>>(
                                wasmtime::component::Resource::new_own(context_rep),
                            );
                        });

                        result
                    }
                    #[cfg(feature = "v0_2")]
                    PluginApiVersion::V0_2 => {
                        let plugin =
                            guest_exports::<pumpkin_wasm_host_v0_2::Plugin>(&plugin_instance)?;
                        let (context_res, context_rep) = guest.with(|mut store| {
                            let resource = store.data_mut().add(context)?;
                            let rep = resource.rep();
                            Ok::<_, wasmtime::Error>((resource, rep))
                        })?;

                        let result = guest
                            .call(plugin.func_on_unload(), (context_res,))
                            .await
                            .map(|(result,)| result);

                        guest.with(|mut store| {
                            let _ = store.data_mut().resource_table.delete::<Arc<Context>>(
                                wasmtime::component::Resource::new_own(context_rep),
                            );
                        });

                        result
                    }
                    #[cfg(not(feature = "v0_2"))]
                    PluginApiVersion::V0_2 => Err(v0_2_disabled()),
                }
            })
        })
        .await
}

pub async fn handle_ipc_message(
    wasm_plugin: &WasmPlugin,
    sender: &str,
    message: &[u8],
) -> Result<Result<Vec<u8>, String>, wasmtime::Error> {
    let sender = sender.to_owned();
    let message = message.to_owned();
    let function = match wasm_plugin.api_version {
        PluginApiVersion::V0_1 => {
            *guest_exports::<pumpkin_wasm_host_v0_1::Plugin>(&wasm_plugin.plugin_instance)?
                .func_handle_ipc_message()
                .func()
        }
        #[cfg(feature = "v0_2")]
        PluginApiVersion::V0_2 => {
            *guest_exports::<pumpkin_wasm_host_v0_2::Plugin>(&wasm_plugin.plugin_instance)?
                .func_handle_ipc_message()
                .func()
        }
        #[cfg(not(feature = "v0_2"))]
        PluginApiVersion::V0_2 => return Err(v0_2_disabled()),
    };

    wasm_plugin
        .store
        .call_guest(move |mut context| {
            Box::pin(async move {
                context
                    .call_func::<(String, Vec<u8>), (Result<Vec<u8>, String>,)>(
                        function,
                        (sender, message),
                    )
                    .await
                    .map(|(result,)| result)
            })
        })
        .await
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    use wasmtime_wasi::sockets::SocketAddrUse;

    use wasm_encoder::{ComponentBuilder, ComponentExportKind, ComponentTypeRef, InstanceType};

    use super::{
        PluginApiVersion, PluginInitError, SocketPolicy, permissions, plugin_api_version,
        socket_policy_for_permissions,
    };

    const PORT: u16 = 25_565;

    /// A component that imports each named instance and, when `metadata` is set, exports
    /// `pumpkin:plugin/metadata@<metadata>`.
    fn component(metadata: Option<&str>, imports: &[&str]) -> Vec<u8> {
        let mut component = ComponentBuilder::default();
        let mut first = None;
        for name in imports {
            let ty = component.type_instance(None, &InstanceType::new());
            let index = component.import(*name, ComponentTypeRef::Instance(ty));
            first.get_or_insert(index);
        }
        if let Some(version) = metadata {
            let instance = first.unwrap_or_else(|| {
                let ty = component.type_instance(None, &InstanceType::new());
                component.import("unused:dep/dep@0.0.1", ComponentTypeRef::Instance(ty))
            });
            component.export(
                format!("pumpkin:plugin/metadata@{version}"),
                ComponentExportKind::Instance,
                instance,
                None,
            );
        }
        component.finish()
    }

    #[test]
    fn v0_1_plugin_is_identified_by_its_metadata_export() {
        let bytes = component(Some("0.1.0"), &["wasi:random/random@0.2.6"]);
        assert!(matches!(
            plugin_api_version(&bytes),
            Ok(PluginApiVersion::V0_1)
        ));
    }

    #[test]
    fn v0_1_plugin_importing_wasi_p3_is_rejected_with_the_import_name() {
        let bytes = component(Some("0.1.0"), &["wasi:random/random@0.3.0"]);
        assert!(matches!(
            plugin_api_version(&bytes),
            Err(PluginInitError::UnsupportedWasi(name)) if name == "wasi:random/random@0.3.0"
        ));
    }

    #[test]
    fn components_without_a_usable_metadata_export_are_rejected() {
        assert!(matches!(
            plugin_api_version(&component(None, &["wasi:random/random@0.2.6"])),
            Err(PluginInitError::UnknownApiVersion())
        ));
        assert!(matches!(
            plugin_api_version(&component(Some("9.0.0"), &[])),
            Err(PluginInitError::UnsupportedApiVersion(version)) if version == "9.0.0"
        ));
    }

    fn addr(ip: IpAddr) -> SocketAddr {
        SocketAddr::new(ip, PORT)
    }

    fn implicit_addr(ip: IpAddr) -> SocketAddr {
        SocketAddr::new(ip, 0)
    }

    #[test]
    fn tcp_connect_allows_implicit_bind_but_not_listen() {
        let unrestricted = SocketPolicy {
            tcp_connect: true,
            ..SocketPolicy::default()
        };
        let loopback = SocketPolicy {
            loopback_only: true,
            ..unrestricted
        };
        let wildcard_v4 = implicit_addr(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let wildcard_v6 = implicit_addr(IpAddr::V6(Ipv6Addr::UNSPECIFIED));
        let explicit_wildcard = addr(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let loopback_v4 = addr(IpAddr::V4(Ipv4Addr::LOCALHOST));
        let public = addr(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)));

        assert!(unrestricted.allows(wildcard_v4, SocketAddrUse::TcpBind));
        assert!(unrestricted.allows(wildcard_v6, SocketAddrUse::TcpBind));
        assert!(!unrestricted.allows(explicit_wildcard, SocketAddrUse::TcpBind));
        assert!(unrestricted.allows(public, SocketAddrUse::TcpConnect));
        assert!(!unrestricted.allows(wildcard_v4, SocketAddrUse::TcpListen));
        assert!(!unrestricted.allows(wildcard_v4, SocketAddrUse::TcpAccept));

        assert!(loopback.allows(wildcard_v4, SocketAddrUse::TcpBind));
        assert!(loopback.allows(loopback_v4, SocketAddrUse::TcpConnect));
        assert!(!loopback.allows(public, SocketAddrUse::TcpConnect));
        assert!(!loopback.allows(wildcard_v4, SocketAddrUse::TcpListen));
    }

    #[test]
    fn udp_send_allows_implicit_bind_but_not_receive() {
        let unrestricted = SocketPolicy {
            udp_send: true,
            ..SocketPolicy::default()
        };
        let loopback = SocketPolicy {
            loopback_only: true,
            ..unrestricted
        };
        let wildcard = implicit_addr(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let explicit_wildcard = addr(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let loopback_addr = addr(IpAddr::V4(Ipv4Addr::LOCALHOST));
        let public = addr(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)));

        assert!(unrestricted.allows(wildcard, SocketAddrUse::UdpBind));
        assert!(!unrestricted.allows(explicit_wildcard, SocketAddrUse::UdpBind));
        assert!(unrestricted.allows(public, SocketAddrUse::UdpSend));
        assert!(!unrestricted.allows(public, SocketAddrUse::UdpReceive));

        assert!(loopback.allows(wildcard, SocketAddrUse::UdpBind));
        assert!(loopback.allows(loopback_addr, SocketAddrUse::UdpSend));
        assert!(!loopback.allows(public, SocketAddrUse::UdpSend));
    }

    #[test]
    fn network_outbound_supports_udp_replies_without_granting_explicit_bind() {
        let outbound = socket_policy_for_permissions(
            |permission| permission == permissions::NETWORK_OUTBOUND,
            false,
        );
        let outgoing_datagram = socket_policy_for_permissions(
            |permission| permission == permissions::NETWORK_UDP_OUTGOING_DATAGRAM,
            false,
        );
        let wildcard = implicit_addr(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let explicit_wildcard = addr(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let public = addr(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)));

        assert!(outbound.allows(public, SocketAddrUse::TcpConnect));
        assert!(outbound.allows(public, SocketAddrUse::UdpSend));
        assert!(outbound.allows(public, SocketAddrUse::UdpReceive));
        assert!(outbound.allows(wildcard, SocketAddrUse::TcpBind));
        assert!(outbound.allows(wildcard, SocketAddrUse::UdpBind));
        assert!(!outbound.allows(explicit_wildcard, SocketAddrUse::TcpBind));
        assert!(!outbound.allows(explicit_wildcard, SocketAddrUse::UdpBind));

        assert!(outgoing_datagram.allows(public, SocketAddrUse::UdpSend));
        assert!(!outgoing_datagram.allows(public, SocketAddrUse::UdpReceive));
    }

    #[test]
    fn bind_only_permissions_do_not_grant_outbound_access() {
        let bind_only = SocketPolicy {
            tcp_bind: true,
            udp_bind: true,
            udp_receive: true,
            ..SocketPolicy::default()
        };
        let loopback_bind_only = SocketPolicy {
            loopback_only: true,
            ..bind_only
        };
        let wildcard = addr(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let loopback = addr(IpAddr::V4(Ipv4Addr::LOCALHOST));

        assert!(bind_only.allows(wildcard, SocketAddrUse::TcpBind));
        assert!(bind_only.allows(wildcard, SocketAddrUse::TcpListen));
        assert!(!bind_only.allows(loopback, SocketAddrUse::TcpConnect));
        assert!(bind_only.allows(wildcard, SocketAddrUse::UdpBind));
        assert!(bind_only.allows(loopback, SocketAddrUse::UdpReceive));
        assert!(!bind_only.allows(loopback, SocketAddrUse::UdpSend));

        assert!(!loopback_bind_only.allows(wildcard, SocketAddrUse::TcpBind));
        assert!(loopback_bind_only.allows(loopback, SocketAddrUse::TcpBind));
        assert!(!loopback_bind_only.allows(wildcard, SocketAddrUse::UdpBind));
        assert!(loopback_bind_only.allows(loopback, SocketAddrUse::UdpBind));
    }

    #[test]
    fn empty_socket_policy_denies_every_operation() {
        let policy = SocketPolicy::default();
        let address = addr(IpAddr::V4(Ipv4Addr::LOCALHOST));
        for reason in [
            SocketAddrUse::TcpConnect,
            SocketAddrUse::TcpBind,
            SocketAddrUse::TcpListen,
            SocketAddrUse::TcpAccept,
            SocketAddrUse::UdpBind,
            SocketAddrUse::UdpSend,
            SocketAddrUse::UdpReceive,
        ] {
            assert!(!policy.allows(address, reason));
        }
    }

    #[cfg(feature = "v0_2")]
    #[test]
    fn v0_2_plugins_are_accepted_with_the_feature() {
        assert!(matches!(
            super::v0_2_api_version("0.2.0"),
            Ok(super::PluginApiVersion::V0_2)
        ));
    }

    #[cfg(not(feature = "v0_2"))]
    #[test]
    fn v0_2_plugins_are_rejected_without_the_feature() {
        assert!(matches!(
            super::v0_2_api_version("0.2.0"),
            Err(super::PluginInitError::UnsupportedApiVersion(version)) if version.contains("plugin-v0_2")
        ));
    }
}
