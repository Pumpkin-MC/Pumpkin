use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::sync::Mutex;
use tracing::warn;

/// Manages dynamic port allocation for server instances.
///
/// The port allocator tracks which ports are currently in use and provides
/// methods to allocate free ports within a configurable range. This prevents
/// port conflicts when running multiple server instances concurrently.
#[derive(Debug)]
pub struct PortAllocator {
    allocated_ports: Mutex<Vec<u16>>,
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
            allocated_ports: Mutex::new(Vec::new()),
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
        if !self.range.contains(&port) {
            return Err(PortAllocatorError::OutOfRange(port, self.range.clone()));
        }

        let mut ports = self.allocated_ports.lock().unwrap();
        if ports.contains(&port) {
            return Err(PortAllocatorError::AlreadyAllocated(port));
        }
        ports.push(port);
        Ok(())
    }

    /// Allocates a specific port, falling back to any free port if it's taken.
    ///
    /// Returns the allocated port number.
    pub fn allocate_or_any(&self, preferred: u16) -> Result<u16, PortAllocatorError> {
        if !self.range.contains(&preferred) {
            warn!(
                "Preferred port {preferred} is outside allocator range {:?}, finding alternative",
                self.range
            );
            return self.allocate_any();
        }

        let mut ports = self.allocated_ports.lock().unwrap();
        if !ports.contains(&preferred) {
            ports.push(preferred);
            Ok(preferred)
        } else {
            for port in self.range.clone() {
                if !ports.contains(&port) {
                    ports.push(port);
                    return Ok(port);
                }
            }
            Err(PortAllocatorError::NoFreePorts(self.range.clone()))
        }
    }

    /// Allocates any free port within the configured range.
    ///
    /// Returns the allocated port number, or an error if no ports are available.
    pub fn allocate_any(&self) -> Result<u16, PortAllocatorError> {
        let mut ports = self.allocated_ports.lock().unwrap();
        for port in self.range.clone() {
            if !ports.contains(&port) {
                ports.push(port);
                return Ok(port);
            }
        }
        Err(PortAllocatorError::NoFreePorts(self.range.clone()))
    }

    /// Frees a previously allocated port, making it available for reuse.
    pub fn free(&self, port: u16) {
        let mut ports = self.allocated_ports.lock().unwrap();
        ports.retain(|p| *p != port);
    }

    /// Returns the list of currently allocated ports.
    pub fn allocated_ports(&self) -> Vec<u16> {
        self.allocated_ports.lock().unwrap().clone()
    }

    /// Checks whether a specific port is currently allocated.
    pub fn is_allocated(&self, port: u16) -> bool {
        self.allocated_ports.lock().unwrap().contains(&port)
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
}
