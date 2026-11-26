/// DNS resolution
pub const NETWORK_DNS: &str = "network.dns";

/// TCP Sockets
pub const NETWORK_TCP: &str = "network.tcp";

/// UDP Sockets
pub const NETWORK_UDP: &str = "network.udp";

/// TCP Connections
pub const NETWORK_TCP_CONNECT: &str = "network.tcp.connect";

/// Bind incoming connections over TCP
pub const NETWORK_TCP_BIND: &str = "network.tcp.bind";

/// Send / Receive UDP Packets
pub const NETWORK_UDP_CONNECT: &str = "network.udp.connect";

/// Bind UDP Sockets to ports
pub const NETWORK_UDP_BIND: &str = "network.udp.bind";

/// Send a datagram on a non-connected UDP socket
pub const NETWORK_UDP_OUTGOING_DATAGRAM: &str = "network.udp.outgoingdatagram";

/// Restrict to localhost only
pub const NETWORK_LOOPBACK: &str = "network.loopback";

/// Outbound TCP/UDP connections
/// > **Warning:** This is a powerful permission
pub const NETWORK_OUTBOUND: &str = "network.outbound";

/// Outbound HTTP connections
///
/// Separate from `network.outbound`; uses `wasi:http`
pub const HTTP_OUTBOUND: &str = "http.outbound";

/// Read files within its own data folder (`plugins/data/<name>`)
pub const FS_READ_DATA: &str = "fs.read.data";

/// Write (and read) files within its own data folder (`plugins/data/<name>`)
///
/// > This permission also implies `FS_READ_DATA`
pub const FS_WRITE_DATA: &str = "fs.write.data";

/// Read all environment variables
pub const SYS_ENV: &str = "sys.env";

/// Read specific environment variables
///
/// Used with a prefix like "sys.env.PATH"
pub const SYS_ENV_PREFIX: &str = "sys.env.";

/// Read system information (CPU, Memory, OS)
pub const SYS_INFO: &str = "sys.info";

/// Read CPU information
pub const SYS_INFO_CPU: &str = "sys.info.cpu";

/// Read RAM information
pub const SYS_INFO_RAM: &str = "sys.info.ram";

/// Read OS information
pub const SYS_INFO_OS: &str = "sys.info.os";
