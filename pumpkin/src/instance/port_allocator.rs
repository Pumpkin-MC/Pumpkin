use std::collections::HashSet;
use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::sync::Mutex;
use tracing::warn;

/// Network transport used by a port reservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortProtocol {
    Tcp,
    Udp,
}

/// A port reservation owned by an instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortReservation {
    pub protocol: PortProtocol,
    pub port: u16,
}

/// Manages dynamic port allocation for server instances.
///
/// The port allocator tracks which ports are currently in use and provides
/// methods to allocate free ports within a configurable range. This prevents
/// port conflicts when running multiple server instances concurrently.
#[derive(Debug)]
pub struct PortAllocator {
    allocated_ports: Mutex<HashSet<PortReservation>>,
    range: RangeInclusive<u16>,
}

#[derive(Debug, thiserror::Error)]
pub enum PortAllocatorError {
    #[error("Port {0} is already allocated")]
    AlreadyAllocated(u16),
    #[error("Port {0} is outside the allowed range {1:?}")]
    OutOfRange(u16, RangeInclusive<u16>),
    #[error("No free ports available in range {0:?}")]
    NoFreePorts(RangeInclusive<u16>),
}

impl PortAllocator {
    /// Creates a new port allocator with the given port range.
    ///
    /// # Arguments
    /// * `range` - The inclusive range of ports to allocate from.
    pub fn new(range: RangeInclusive<u16>) -> Self {
        Self {
            allocated_ports: Mutex::new(HashSet::new()),
            range,
        }
    }

    /// Creates a port allocator with a sensible default range for Minecraft servers.
    ///
    /// Default range: 25565..=25665 (100 ports for Java + Bedrock instances).
    pub fn default_range() -> Self {
        Self::new(25565..=25665)
    }

    /// Attempts to allocate a specific port.
    ///
    /// Returns `Ok(())` if the port was successfully allocated, or an error
    /// if the port is already in use or outside the allowed range.
    pub fn allocate(&self, port: u16) -> Result<(), PortAllocatorError> {
        self.allocate_for(PortProtocol::Tcp, port)
    }

    /// Reserves a specific port for a transport.
    pub fn allocate_for(
        &self,
        protocol: PortProtocol,
        port: u16,
    ) -> Result<(), PortAllocatorError> {
        if !self.range.contains(&port) {
            return Err(PortAllocatorError::OutOfRange(port, self.range.clone()));
        }

        let mut ports = self.allocated_ports.lock().unwrap();
        if !ports.insert(PortReservation { protocol, port }) {
            return Err(PortAllocatorError::AlreadyAllocated(port));
        }
        Ok(())
    }

    /// Allocates a specific port, falling back to any free port if it's taken.
    ///
    /// Returns the allocated port number.
    pub fn allocate_or_any(&self, preferred: u16) -> Result<u16, PortAllocatorError> {
        self.allocate_or_any_for(PortProtocol::Tcp, preferred)
    }

    /// Reserves a preferred port for a transport, falling back to the configured range.
    ///
    /// Explicit ports outside the automatic range are accepted when available. This
    /// preserves normal Minecraft configurations while still allowing automatic
    /// assignment for `0` and conflicting ports.
    pub fn allocate_or_any_for(
        &self,
        protocol: PortProtocol,
        preferred: u16,
    ) -> Result<u16, PortAllocatorError> {
        let reservation = PortReservation {
            protocol,
            port: preferred,
        };

        if preferred != 0 && !self.range.contains(&preferred) {
            let mut ports = self.allocated_ports.lock().unwrap();
            if ports.insert(reservation) {
                return Ok(preferred);
            }
            warn!(
                "Preferred port {preferred} is already allocated for {protocol:?}, finding alternative"
            );
            drop(ports);
            return self.allocate_any_for(protocol);
        }

        if preferred == 0 {
            return self.allocate_any_for(protocol);
        }

        let mut ports = self.allocated_ports.lock().unwrap();
        if ports.insert(reservation) {
            return Ok(preferred);
        }

        for port in self.range.clone() {
            if ports.insert(PortReservation { protocol, port }) {
                return Ok(port);
            }
        }
        Err(PortAllocatorError::NoFreePorts(self.range.clone()))
    }

    /// Allocates any free port for a transport within the configured range.
    pub fn allocate_any_for(&self, protocol: PortProtocol) -> Result<u16, PortAllocatorError> {
        let mut ports = self.allocated_ports.lock().unwrap();
        for port in self.range.clone() {
            if ports.insert(PortReservation { protocol, port }) {
                return Ok(port);
            }
        }
        Err(PortAllocatorError::NoFreePorts(self.range.clone()))
    }

    /// Releases a transport-specific reservation.
    pub fn free_for(&self, protocol: PortProtocol, port: u16) {
        self.allocated_ports
            .lock()
            .unwrap()
            .remove(&PortReservation { protocol, port });
    }

    /// Checks whether a transport-specific port is reserved.
    pub fn is_allocated_for(&self, protocol: PortProtocol, port: u16) -> bool {
        self.allocated_ports
            .lock()
            .unwrap()
            .contains(&PortReservation { protocol, port })
    }

    /// Returns all reservations currently held by the allocator.
    pub fn reservations(&self) -> Vec<PortReservation> {
        self.allocated_ports
            .lock()
            .unwrap()
            .iter()
            .copied()
            .collect()
    }

    /// Allocates any free port within the configured range.
    ///
    /// Returns the allocated port number, or an error if no ports are available.
    pub fn allocate_any(&self) -> Result<u16, PortAllocatorError> {
        self.allocate_any_for(PortProtocol::Tcp)
    }

    /// Frees a previously allocated port, making it available for reuse.
    pub fn free(&self, port: u16) {
        self.free_for(PortProtocol::Tcp, port);
    }

    /// Returns the list of currently allocated ports.
    pub fn allocated_ports(&self) -> Vec<u16> {
        let mut ports: Vec<_> = self
            .allocated_ports
            .lock()
            .unwrap()
            .iter()
            .map(|reservation| reservation.port)
            .collect();
        ports.sort_unstable();
        ports.dedup();
        ports
    }

    /// Checks whether a specific port is currently allocated.
    pub fn is_allocated(&self, port: u16) -> bool {
        self.allocated_ports
            .lock()
            .unwrap()
            .iter()
            .any(|reservation| reservation.port == port)
    }

    /// Creates a `SocketAddr` from an allocated port with the given IP.
    pub fn make_addr(&self, ip: std::net::IpAddr, port: u16) -> SocketAddr {
        SocketAddr::new(ip, port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_and_free() {
        let allocator = PortAllocator::new(30000..=30100);
        assert!(allocator.allocate(30000).is_ok());
        assert!(allocator.is_allocated(30000));
        assert!(allocator.allocate(30000).is_err());
        allocator.free(30000);
        assert!(!allocator.is_allocated(30000));
        assert!(allocator.allocate(30000).is_ok());
    }

    #[test]
    fn test_allocate_any() {
        let allocator = PortAllocator::new(30000..=30100);
        let port = allocator.allocate_any().unwrap();
        assert!((30000..=30100).contains(&port));
        allocator.free(port);
    }

    #[test]
    fn test_out_of_range() {
        let allocator = PortAllocator::new(30000..=30100);
        assert!(allocator.allocate(29999).is_err());
        assert!(allocator.allocate(30101).is_err());
    }

    #[test]
    fn test_allocate_or_any_fallback() {
        let allocator = PortAllocator::new(30000..=30100);
        let preferred = 30005;
        let port1 = allocator.allocate_or_any(preferred).unwrap();
        assert_eq!(port1, preferred);

        // Now that preferred is taken, should get a different port
        let port2 = allocator.allocate_or_any(preferred).unwrap();
        assert_ne!(port2, preferred);
        assert!((30000..=30100).contains(&port2));
    }

    #[test]
    fn tcp_and_udp_can_share_a_port() {
        let allocator = PortAllocator::new(30000..=30010);
        assert!(allocator.allocate_for(PortProtocol::Tcp, 30000).is_ok());
        assert!(allocator.allocate_for(PortProtocol::Udp, 30000).is_ok());
        assert!(allocator.is_allocated_for(PortProtocol::Tcp, 30000));
        assert!(allocator.is_allocated_for(PortProtocol::Udp, 30000));
    }

    #[test]
    fn preferred_outside_range_is_kept_when_available() {
        let allocator = PortAllocator::new(30000..=30010);
        assert_eq!(
            allocator
                .allocate_or_any_for(PortProtocol::Tcp, 25565)
                .unwrap(),
            25565
        );
    }
}
