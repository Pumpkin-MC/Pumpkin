#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::{fs, net::SocketAddr, path::Path, sync::Arc};
use thiserror::Error;
use wasmtime::{Cache, CacheConfig, Engine, component::Component, component::Linker};
use wasmtime_wasi::{FsPerms, WasiCtxBuilder, sockets::SocketAddrUse};

use crate::plugin::{
    Context, PluginMetadata, cache::calculate_hash_for_bytes,
    loader::wasm::wasm_host::state::PluginHostState, permissions,
};
use pumpkin_plugin_runtime::RuntimeSpawner;

pub mod args;
pub mod concurrent_store;
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
    #[error("Failed to get absolute path: {0}")]
    PathResolutionFailed(std::io::Error),
    #[error("Failed to create cache: {0}")]
    CacheCreationFailed(wasmtime::Error),
}

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

pub struct PluginRuntime {
    engine: Engine,
    cache_dir: std::path::PathBuf,
    linker: wasmtime::component::Linker<PluginHostState>,
    legacy_sync_reentry: concurrent_store::LegacySyncReentry,
    store_spawner: Arc<dyn RuntimeSpawner>,
}

pub enum PluginInstance {
    V0_1(wit::v0_1::Plugin),
}

pub struct WasmPlugin {
    pub plugin_instance: Arc<PluginInstance>,
    pub store: concurrent_store::LegacyStore,
}

impl PluginRuntime {
    pub fn new<P: AsRef<Path>>(
        path: P,
        legacy_sync_reentry: concurrent_store::LegacySyncReentry,
        store_spawner: Arc<dyn RuntimeSpawner>,
    ) -> Result<Self, PluginInitError> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.concurrency_support(true);
        let mut path =
            std::path::absolute(path.as_ref()).map_err(PluginInitError::PathResolutionFailed)?;
        path.pop();
        path.push("cache");
        let mut cache_config = CacheConfig::new();
        cache_config.with_directory(&path);
        let cache = Cache::new(cache_config).map_err(PluginInitError::CacheCreationFailed)?;
        config.cache(Some(cache));

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
            legacy_sync_reentry,
            store_spawner,
        })
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
                    verification.metadata.map(|m| {
                        wit::v0_1::pumpkin::plugin::context::MarketplaceMetadata {
                            marketplace_url: m.marketplace_url,
                            plugin_id: m.plugin_id,
                            plugin_name: m.plugin_name,
                            version: m.version,
                            dev_id: m.dev_id,
                            dev_name: m.dev_name,
                            is_paid: m.is_paid,
                            user_id: m.user_id,
                            license_key: m.license_key,
                            issued_at: m.issued_at,
                        }
                    })
                } else {
                    None
                }
            },
        )
        .flatten();

        let wasm_bytes = signature::strip_pumpkin_sections(&wasm_bytes).unwrap_or(wasm_bytes);

        let component = load_component(&self.engine, &wasm_bytes, &self.cache_dir)?;

        let instance_pre = self
            .linker
            .instantiate_pre(&component)
            .map_err(PluginInitError::ApiVersionMismatch)?;

        let (plugin_instance, store, metadata) = {
            let plugin_pre = wit::v0_1::prepare_plugin(&instance_pre)
                .map_err(PluginInitError::ApiVersionMismatch)?;

            wit::v0_1::init_plugin(&self.engine, plugin_pre, &self.legacy_sync_reentry).await?
        };

        let store = concurrent_store::start_legacy_store(
            store,
            self.legacy_sync_reentry.clone(),
            Arc::clone(&self.store_spawner),
        )
        .await
        .map_err(PluginInitError::InstantiationFailed)?;
        let wasm_plugin = Arc::new(WasmPlugin {
            plugin_instance: Arc::new(plugin_instance),
            store,
        });
        let weak_plugin = Arc::downgrade(&wasm_plugin);
        wasm_plugin
            .store
            .call(move |accessor| {
                Box::pin(async move {
                    accessor.with(|mut store| {
                        store.data_mut().plugin = Some(weak_plugin);
                        store.data_mut().marketplace_metadata = marketplace_metadata;
                    });
                    Ok(())
                })
            })
            .await
            .map_err(PluginInitError::InstantiationFailed)?;

        tracing::debug!(
            wasm_plugin_api = "0.1",
            wasm_plugin_policy = concurrent_store::LegacySyncReentry::NAME,
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

fn setup_linker(engine: &Engine) -> wasmtime::Result<Linker<PluginHostState>> {
    let mut linker = Linker::<PluginHostState>::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    wasmtime_wasi_http::p2::add_only_http_to_linker_async(&mut linker)?;
    wasmtime_wasi::p3::add_to_linker(&mut linker)?;
    wasmtime_wasi_http::p3::add_to_linker(&mut linker)?;
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

impl WasmPlugin {
    #[allow(clippy::too_many_lines)]
    pub async fn on_load(
        &self,
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
        builder.allow_udp(
            socket_policy.udp_send || socket_policy.udp_receive || socket_policy.udp_bind,
        );
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
        let function = match self.plugin_instance.as_ref() {
            PluginInstance::V0_1(plugin) => plugin.func_on_load(),
        };

        self.store
            .call_guest(move |mut guest| {
                Box::pin(async move {
                    let context_res = guest.with(|mut store| {
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
                        store.data_mut().add_context(context)
                    })?;

                    guest
                        .call(function, (context_res,))
                        .await
                        .map(|(result,)| result)
                })
            })
            .await
    }

    pub async fn on_unload(
        &self,
        context: Arc<Context>,
    ) -> Result<Result<(), String>, wasmtime::Error> {
        let loaded_plugin = self
            .store
            .call(|accessor| {
                Box::pin(async move {
                    Ok(accessor.with(|mut store| {
                        store
                            .data_mut()
                            .plugin
                            .as_ref()
                            .and_then(std::sync::Weak::upgrade)
                    }))
                })
            })
            .await?;

        if let Some(plugin) = loaded_plugin {
            context.server.task_scheduler.disable_plugin(&plugin);
        }

        let function = match self.plugin_instance.as_ref() {
            PluginInstance::V0_1(plugin) => plugin.func_on_unload(),
        };
        self.store
            .shutdown(move |accessor| {
                Box::pin(async move {
                    let (context_res, context_rep) = accessor.with(|mut store| {
                        let resource = store.data_mut().add_context(context)?;
                        let rep = resource.rep();
                        Ok::<_, wasmtime::Error>((resource, rep))
                    })?;
                    let result = function
                        .call_concurrent(accessor, (context_res,))
                        .await
                        .map(|(result,)| result);
                    accessor.with(|mut store| {
                        let _ = store.data_mut().resource_table.delete::<
                            crate::plugin::loader::wasm::wasm_host::state::ContextResource,
                        >(wasmtime::component::Resource::new_own(context_rep));
                    });
                    result
                })
            })
            .await
    }

    pub async fn handle_ipc_message(
        &self,
        sender: &str,
        message: &[u8],
    ) -> Result<Result<Vec<u8>, String>, wasmtime::Error> {
        let sender = sender.to_owned();
        let message = message.to_owned();
        let function = match self.plugin_instance.as_ref() {
            PluginInstance::V0_1(plugin) => plugin.func_handle_ipc_message(),
        };

        self.store
            .call_guest(move |mut context| {
                Box::pin(async move {
                    context
                        .call(function, (sender, message))
                        .await
                        .map(|(result,)| result)
                })
            })
            .await
    }
}

pub trait DowncastResourceExt<E> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a E;
    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut E;
    fn consume(self, state: &mut PluginHostState) -> E;
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    use wasmtime_wasi::sockets::SocketAddrUse;

    use super::{SocketPolicy, permissions, socket_policy_for_permissions};

    const PORT: u16 = 25_565;

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
}
