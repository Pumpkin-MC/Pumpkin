use crate::PumpkinServer;
use crate::instance::config::{InstanceConfig, InstanceId};
use crate::instance::port_allocator::{PortAllocator, PortReservation};
use crate::server::ServerPaths;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio_util::sync::CancellationToken;
use tracing::info;

#[derive(Debug, thiserror::Error)]
pub enum InstanceError {
    #[error("Instance not found: {0:?}")]
    NotFound(InstanceId),
    #[error("Instance {0:?} is already running")]
    AlreadyRunning(InstanceId),
    #[error("Instance {0:?} is not running")]
    NotRunning(InstanceId),
    #[error("Port allocation failed: {0}")]
    PortAllocation(String),
    #[error("Failed to bind network: {0}")]
    NetworkBind(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceState {
    Created,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

pub struct InstanceInfo {
    pub id: InstanceId,
    pub config: InstanceConfig,
    pub state: InstanceState,
    pub server: Option<Arc<PumpkinServer>>,
    pub ports: Vec<PortReservation>,
    /// Per-instance cancellation token used to stop this instance without
    /// affecting other concurrent instances.
    pub stop_token: CancellationToken,
    run_started: AtomicBool,
    run_finished: CancellationToken,
}

impl std::fmt::Debug for InstanceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstanceInfo")
            .field("id", &self.id)
            .field("config", &self.config)
            .field("state", &self.state)
            .field("has_server", &self.server.is_some())
            .field("ports", &self.ports)
            .finish()
    }
}

/// Manages multiple Pumpkin server instances.
pub struct InstanceManager {
    instances: HashMap<InstanceId, InstanceInfo>,
    port_allocator: Arc<PortAllocator>,
    next_id: AtomicU64,
}

impl InstanceManager {
    pub fn new() -> Self {
        Self::with_port_range(25565..=25665)
    }

    pub fn with_port_range(range: std::ops::RangeInclusive<u16>) -> Self {
        Self {
            instances: HashMap::new(),
            port_allocator: Arc::new(PortAllocator::new(range)),
            next_id: AtomicU64::new(1),
        }
    }

    fn generate_id(&self) -> InstanceId {
        InstanceId(self.next_id.fetch_add(1, Ordering::SeqCst))
    }

    /// Creates an instance and prepares its isolated directories.
    pub async fn create_instance(
        &mut self,
        config: InstanceConfig,
    ) -> Result<InstanceId, InstanceError> {
        let id = self.generate_id();

        for path in [
            config.config_path(),
            config.world_path(),
            config.plugin_path(),
            config.data_path(),
        ] {
            tokio::fs::create_dir_all(path).await?;
        }

        self.instances.insert(
            id,
            InstanceInfo {
                id,
                config,
                state: InstanceState::Created,
                server: None,
                ports: Vec::new(),
                stop_token: CancellationToken::new(),
                run_started: AtomicBool::new(false),
                run_finished: CancellationToken::new(),
            },
        );
        info!("Created instance {id:?}");
        Ok(id)
    }

    /// Starts an instance, including its accept loop.
    pub async fn start_instance(&mut self, id: InstanceId) -> Result<(), InstanceError> {
        let info = self
            .instances
            .get_mut(&id)
            .ok_or(InstanceError::NotFound(id))?;

        if matches!(info.state, InstanceState::Starting | InstanceState::Running) {
            return Err(InstanceError::AlreadyRunning(id));
        }

        info.state = InstanceState::Starting;
        info.stop_token = CancellationToken::new();

        let reservations = match info.config.reserve_ports(&self.port_allocator) {
            Ok(reservations) => reservations,
            Err(error) => {
                info.state = InstanceState::Failed;
                return Err(InstanceError::PortAllocation(error.to_string()));
            }
        };

        let paths = ServerPaths {
            world_dir: info.config.world_path(),
            data_dir: info.config.data_path(),
            plugin_dir: info.config.plugin_path(),
        };
        let server_result = PumpkinServer::new_with_result_and_paths(
            info.config.basic.clone(),
            info.config.advanced.clone(),
            crate::data::VanillaData::load_from(&paths.data_dir),
            paths,
            info.stop_token.clone(),
        )
        .await;

        let server = match server_result {
            Ok(server) => Arc::new(server),
            Err(error) => {
                for reservation in &reservations {
                    self.port_allocator
                        .free_for(reservation.protocol, reservation.port);
                }
                info.state = InstanceState::Failed;
                return Err(InstanceError::NetworkBind(error.to_string()));
            }
        };

        server.init_plugins().await;
        info.server = Some(server.clone());
        info.ports = reservations;
        info.state = InstanceState::Running;
        info.run_started.store(true, Ordering::Release);
        info.run_finished = CancellationToken::new();

        let stop_token = info.stop_token.clone();
        let run_finished = info.run_finished.clone();
        tokio::spawn(async move {
            server.start_with_token(stop_token).await;
            run_finished.cancel();
        });

        info!("Started instance {id:?}");
        Ok(())
    }

    /// Waits for a running instance to stop.
    pub async fn run_instance(&self, id: InstanceId) -> Result<(), InstanceError> {
        let info = self.instances.get(&id).ok_or(InstanceError::NotFound(id))?;
        if info.server.is_none() || !info.run_started.load(Ordering::Acquire) {
            return Err(InstanceError::NotRunning(id));
        }
        info.run_finished.cancelled().await;
        Ok(())
    }

    /// Stops an instance and waits for all of its server tasks to finish.
    pub async fn stop_instance(&mut self, id: InstanceId) -> Result<(), InstanceError> {
        let info = self
            .instances
            .get_mut(&id)
            .ok_or(InstanceError::NotFound(id))?;

        if !matches!(info.state, InstanceState::Starting | InstanceState::Running) {
            info.state = InstanceState::Stopped;
            return Ok(());
        }

        info.state = InstanceState::Stopping;
        info.stop_token.cancel();

        if info.run_started.load(Ordering::Acquire) {
            info.run_finished.cancelled().await;
        } else if let Some(server) = info.server.as_ref() {
            server.unload_plugins().await;
            server.server.shutdown().await;
        }

        info.server = None;
        info.run_started.store(false, Ordering::Release);
        for reservation in info.ports.drain(..) {
            self.port_allocator
                .free_for(reservation.protocol, reservation.port);
        }
        info.state = InstanceState::Stopped;
        info!("Stopped instance {id:?}");
        Ok(())
    }

    /// Stops and deletes an instance.
    pub async fn delete_instance(&mut self, id: InstanceId) -> Result<(), InstanceError> {
        self.stop_instance(id).await?;
        self.instances.remove(&id);
        info!("Deleted instance {id:?}");
        Ok(())
    }

    pub fn get_instance(&self, id: InstanceId) -> Option<&InstanceInfo> {
        self.instances.get(&id)
    }

    pub fn get_instance_mut(&mut self, id: InstanceId) -> Option<&mut InstanceInfo> {
        self.instances.get_mut(&id)
    }

    pub fn list_instances(&self) -> Vec<InstanceId> {
        let mut ids: Vec<_> = self.instances.keys().copied().collect();
        ids.sort_unstable_by_key(|id| id.0);
        ids
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    pub fn port_allocator(&self) -> &PortAllocator {
        &self.port_allocator
    }

    pub fn port_allocator_arc(&self) -> Arc<PortAllocator> {
        self.port_allocator.clone()
    }
}

impl Default for InstanceManager {
    fn default() -> Self {
        Self::new()
    }
}
