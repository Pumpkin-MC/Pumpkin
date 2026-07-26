use crate::instance::config::{InstanceConfig, InstanceId};
use crate::instance::port_allocator::PortAllocator;
use crate::PumpkinServer;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
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
    #[error("Server creation failed: {0}")]
    Creation(String),
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
    /// Per-instance cancellation token used to stop this instance without
    /// affecting other concurrent instances.
    pub stop_token: CancellationToken,
}

impl std::fmt::Debug for InstanceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstanceInfo")
            .field("id", &self.id)
            .field("config", &self.config)
            .field("state", &self.state)
            .field("has_server", &self.server.is_some())
            .finish()
    }
}

/// Manages multiple Pumpkin server instances.
///
/// The `InstanceManager` provides a centralized system for creating, starting,
/// stopping, and managing multiple isolated server instances. It handles port
/// allocation, resource isolation, and lifecycle management.
pub struct InstanceManager {
    instances: HashMap<InstanceId, InstanceInfo>,
    port_allocator: Arc<PortAllocator>,
    next_id: AtomicU64,
}

impl InstanceManager {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            port_allocator: Arc::new(PortAllocator::default_range()),
            next_id: AtomicU64::new(1),
        }
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

    /// Creates a new server instance with the given configuration.
    ///
    /// The instance is created in the `Created` state. Call `start_instance`
    /// to actually begin running it.
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid or if required
    /// directories cannot be created.
    pub async fn create_instance(
        &mut self,
        config: InstanceConfig,
    ) -> Result<InstanceId, InstanceError> {
        let id = self.generate_id();

        let config_dir = config.config_path();
        if !config_dir.exists() {
            tokio::fs::create_dir_all(&config_dir).await?;
        }

        let world_dir = config.world_path();
        if !world_dir.exists() {
            tokio::fs::create_dir_all(&world_dir).await?;
        }

        let plugin_dir = config.plugin_path();
        if !plugin_dir.exists() {
            tokio::fs::create_dir_all(&plugin_dir).await?;
        }

        let info = InstanceInfo {
            id,
            config,
            state: InstanceState::Created,
            server: None,
            stop_token: CancellationToken::new(),
        };

        self.instances.insert(id, info);
        info!("Created instance {id:?}");
        Ok(id)
    }

    /// Starts a created instance.
    ///
    /// This loads vanilla data, creates the `PumpkinServer`, binds network ports,
    /// and initializes plugins. Call [`Self::run_instance`] to begin the accept loop.
    ///
    /// # Errors
    /// Returns an error if the instance is not found, already running, or
    /// if network binding fails.
    pub async fn start_instance(&mut self, id: InstanceId) -> Result<(), InstanceError> {
        let info = self
            .instances
            .get_mut(&id)
            .ok_or(InstanceError::NotFound(id))?;

        if info.state == InstanceState::Running || info.state == InstanceState::Starting {
            return Err(InstanceError::AlreadyRunning(id));
        }

        info.state = InstanceState::Starting;
        // Fresh token for this start cycle so a previous stop does not leave it cancelled.
        info.stop_token = CancellationToken::new();

        let plugin_dir = info.config.plugin_path();

        // Take ownership of the configs from the instance info.
        // We replace with defaults temporarily so we can move the values out.
        // Note: BasicConfiguration / AdvancedConfiguration are not Clone.
        let basic_config = std::mem::take(&mut info.config.basic);
        let advanced_config = std::mem::take(&mut info.config.advanced);

        let vanilla_data = crate::data::VanillaData::load();

        let server = match PumpkinServer::new_with_result(
            basic_config,
            advanced_config,
            vanilla_data,
        )
        .await
        {
            Ok(server) => server,
            Err(e) => {
                info.state = InstanceState::Failed;
                return Err(InstanceError::NetworkBind(e.to_string()));
            }
        };

        server.server.plugin_manager.set_plugin_dir(plugin_dir).await;
        server.init_plugins().await;

        info.server = Some(Arc::new(server));
        info.state = InstanceState::Running;

        info!("Started instance {id:?}");
        Ok(())
    }

    /// Starts the main loop for a running instance.
    ///
    /// This method blocks until the instance is stopped. It should typically
    /// be spawned as a task if you want to run multiple instances concurrently.
    ///
    /// # Errors
    /// Returns an error if the instance is not found or not running.
    pub async fn run_instance(&self, id: InstanceId) -> Result<(), InstanceError> {
        let info = self.instances.get(&id).ok_or(InstanceError::NotFound(id))?;
        let server = info
            .server
            .clone()
            .ok_or(InstanceError::NotRunning(id))?;
        let stop_token = info.stop_token.clone();

        server.start_with_token(stop_token).await;
        Ok(())
    }

    /// Stops a running instance gracefully.
    ///
    /// Cancels the instance's cancellation token so any active
    /// [`Self::run_instance`] call can exit. Does not use the global
    /// `stop_server()` path, so other instances keep running.
    ///
    /// # Errors
    /// Returns an error if the instance is not found.
    pub async fn stop_instance(&mut self, id: InstanceId) -> Result<(), InstanceError> {
        let info = self
            .instances
            .get_mut(&id)
            .ok_or(InstanceError::NotFound(id))?;

        if info.state != InstanceState::Running && info.state != InstanceState::Starting {
            info.state = InstanceState::Stopped;
            return Ok(());
        }

        info.state = InstanceState::Stopping;
        info.stop_token.cancel();

        if let Some(server) = info.server.take() {
            server.unload_plugins().await;
            drop(server);
        }

        info.state = InstanceState::Stopped;
        info!("Stopped instance {id:?}");
        Ok(())
    }

    /// Deletes an instance, stopping it first if it's running.
    ///
    /// After deletion, the instance ID is no longer valid.
    ///
    /// # Errors
    /// Returns an error if the instance is not found.
    pub async fn delete_instance(&mut self, id: InstanceId) -> Result<(), InstanceError> {
        {
            let info = self
                .instances
                .get_mut(&id)
                .ok_or(InstanceError::NotFound(id))?;

            if info.state == InstanceState::Running || info.state == InstanceState::Starting {
                info.state = InstanceState::Stopping;
                info.stop_token.cancel();
                if let Some(server) = info.server.take() {
                    server.unload_plugins().await;
                    drop(server);
                }
            }
        }

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
        self.instances.keys().copied().collect()
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
