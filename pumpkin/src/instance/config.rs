use pumpkin_config::{AdvancedConfiguration, BasicConfiguration};
use std::path::PathBuf;

use super::port_allocator::{PortAllocator, PortAllocatorError, PortProtocol, PortReservation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(pub u64);

/// Configuration for creating a new server instance.
///
/// This struct allows specifying per-instance overrides for configuration paths,
/// world directories, plugin directories, and network settings. Any field set to
/// `None` will fall back to the corresponding value in the `BasicConfiguration` or
/// `AdvancedConfiguration`.
#[derive(Clone)]
pub struct InstanceConfig {
    /// The basic server configuration.
    pub basic: BasicConfiguration,
    /// The advanced server configuration.
    pub advanced: AdvancedConfiguration,
    /// The directory from which to load the configuration file.
    ///
    /// Defaults to the current working directory if `None`.
    pub config_dir: Option<PathBuf>,
    /// Override for the world directory path.
    ///
    /// If `None`, the world path is derived from `basic.default_level_name`.
    pub world_dir: Option<PathBuf>,
    /// Override for the plugin directory path.
    ///
    /// If `None`, defaults to `./plugins`.
    pub plugin_dir: Option<PathBuf>,
    /// Override for the data directory path.
    ///
    /// If `None`, defaults to `./data`.
    pub data_dir: Option<PathBuf>,
}

impl std::fmt::Debug for InstanceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // BasicConfiguration / AdvancedConfiguration do not implement Debug.
        f.debug_struct("InstanceConfig")
            .field("config_dir", &self.config_dir)
            .field("world_dir", &self.world_dir)
            .field("plugin_dir", &self.plugin_dir)
            .field("data_dir", &self.data_dir)
            .finish_non_exhaustive()
    }
}

impl InstanceConfig {
    pub fn new(basic: BasicConfiguration, advanced: AdvancedConfiguration) -> Self {
        Self {
            basic,
            advanced,
            config_dir: None,
            world_dir: None,
            plugin_dir: None,
            data_dir: None,
        }
    }

    pub fn with_config_dir(mut self, path: PathBuf) -> Self {
        self.config_dir = Some(path);
        self
    }

    pub fn with_world_dir(mut self, path: PathBuf) -> Self {
        self.world_dir = Some(path);
        self
    }

    pub fn with_plugin_dir(mut self, path: PathBuf) -> Self {
        self.plugin_dir = Some(path);
        self
    }

    pub fn with_data_dir(mut self, path: PathBuf) -> Self {
        self.data_dir = Some(path);
        self
    }

    pub fn world_path(&self) -> PathBuf {
        self.world_dir
            .clone()
            .unwrap_or_else(|| self.config_path().join(&self.basic.default_level_name))
    }

    pub fn plugin_path(&self) -> PathBuf {
        self.plugin_dir
            .clone()
            .unwrap_or_else(|| self.config_path().join("plugins"))
    }

    pub fn data_path(&self) -> PathBuf {
        self.data_dir
            .clone()
            .unwrap_or_else(|| self.config_path().join("data"))
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Reserves and applies all ports used by this instance.
    ///
    /// A configured port is retained when it is available. Port `0` and ports
    /// already reserved by another instance are replaced with a free port from
    /// the allocator's range.
    pub fn reserve_ports(
        &mut self,
        allocator: &PortAllocator,
    ) -> Result<Vec<PortReservation>, PortAllocatorError> {
        let mut reservations = Vec::new();

        let result = (|| {
            let mut reserve = |address: &mut std::net::SocketAddr, protocol: PortProtocol| {
                let port = allocator.allocate_or_any_for(protocol, address.port())?;
                address.set_port(port);
                reservations.push(PortReservation { protocol, port });
                Ok::<(), PortAllocatorError>(())
            };

            if self.advanced.networking.java.enabled {
                reserve(
                    &mut self.advanced.networking.java.address,
                    PortProtocol::Tcp,
                )?;

                if self.advanced.networking.query.enabled {
                    reserve(
                        &mut self.advanced.networking.query.address,
                        PortProtocol::Udp,
                    )?;
                }
            }

            if self.advanced.networking.bedrock.enabled {
                reserve(
                    &mut self.advanced.networking.bedrock.address,
                    PortProtocol::Udp,
                )?;
            }

            if self.advanced.networking.rcon.enabled {
                reserve(
                    &mut self.advanced.networking.rcon.address,
                    PortProtocol::Tcp,
                )?;
            }

            if self.advanced.networking.lan_broadcast.enabled {
                let configured_port = self.advanced.networking.lan_broadcast.port.unwrap_or(0);
                let mut address = std::net::SocketAddr::from(([0, 0, 0, 0], configured_port));
                reserve(&mut address, PortProtocol::Udp)?;
                self.advanced.networking.lan_broadcast.port = Some(address.port());
            }

            Ok::<(), PortAllocatorError>(())
        })();

        if let Err(error) = result {
            for reservation in reservations {
                allocator.free_for(reservation.protocol, reservation.port);
            }
            return Err(error);
        }

        Ok(reservations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::{PortAllocator, PortProtocol};
    use std::path::Path;

    #[test]
    fn explicit_config_root_is_used_for_default_paths() {
        let config = InstanceConfig::new(
            BasicConfiguration::default(),
            AdvancedConfiguration::default(),
        )
        .with_config_dir(Path::new("/tmp/pumpkin-instance").to_path_buf());

        assert_eq!(
            config.world_path(),
            Path::new("/tmp/pumpkin-instance/world")
        );
        assert_eq!(
            config.plugin_path(),
            Path::new("/tmp/pumpkin-instance/plugins")
        );
        assert_eq!(config.data_path(), Path::new("/tmp/pumpkin-instance/data"));
    }

    #[test]
    fn reserve_ports_updates_network_configuration() {
        let mut config = InstanceConfig::new(
            BasicConfiguration::default(),
            AdvancedConfiguration::default(),
        );
        let allocator = PortAllocator::new(30000..=30010);
        let reservations = config.reserve_ports(&allocator).unwrap();

        assert!(
            reservations
                .iter()
                .any(|reservation| reservation.protocol == PortProtocol::Tcp)
        );
        assert!(
            reservations
                .iter()
                .any(|reservation| reservation.protocol == PortProtocol::Udp)
        );
        assert_eq!(config.advanced.networking.java.address.port(), 25565);
        assert!(allocator.is_allocated_for(
            PortProtocol::Tcp,
            config.advanced.networking.java.address.port()
        ));
    }
}
