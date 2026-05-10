/// Allows the plugin to perform DNS resolution.
pub const NETWORK_DNS: &str = "network.dns";

/// Allows the plugin to use TCP sockets.
pub const NETWORK_TCP: &str = "network.tcp";

/// Allows the plugin to use UDP sockets.
pub const NETWORK_UDP: &str = "network.udp";

/// Allows the plugin to initiate TCP connections.
pub const NETWORK_TCP_CONNECT: &str = "network.tcp.connect";

/// Allows the plugin to bind TCP listeners (accept inbound connections).
pub const NETWORK_TCP_BIND: &str = "network.tcp.bind";

/// Allows the plugin to send and receive UDP packets to specific destinations.
pub const NETWORK_UDP_CONNECT: &str = "network.udp.connect";

/// Allows the plugin to bind UDP sockets to local ports.
pub const NETWORK_UDP_BIND: &str = "network.udp.bind";

/// Allows the plugin to send datagram on non-connected UDP socket.
pub const NETWORK_UDP_OUTGOING_DATAGRAM: &str = "network.udp.outgoingdatagram";

/// Restricts all networking permissions to loopback addresses (localhost) only.
pub const NETWORK_LOOPBACK: &str = "network.loopback";

/// Allows the plugin to make outbound TCP/UDP connections.
/// **Warning:** This is a powerful permission.
pub const NETWORK_OUTBOUND: &str = "network.outbound";

/// Allows the plugin to read files from the server's file system outside of its data folder.
pub const FS_READ: &str = "fs.read";

/// Allows the plugin to write files to the server's file system outside of its data folder.
pub const FS_WRITE: &str = "fs.write";

/// Allows the plugin to read files within its own data folder (`plugins/<name>`).
pub const FS_READ_DATA: &str = "fs.read.data";

/// Allows the plugin to write files within its own data folder (`plugins/<name>`).
pub const FS_WRITE_DATA: &str = "fs.write.data";

/// Allows the plugin to read all environment variables.
pub const SYS_ENV: &str = "sys.env";

/// Allows the plugin to read specific environment variables.
/// Used with a prefix like "sys.env.PATH".
pub const SYS_ENV_PREFIX: &str = "sys.env.";

/// Allows the plugin to read system information (CPU, Memory, OS).
pub const SYS_INFO: &str = "sys.info";

/// Allows the plugin to read CPU information.
pub const SYS_INFO_CPU: &str = "sys.info.cpu";

/// Allows the plugin to read RAM information.
pub const SYS_INFO_RAM: &str = "sys.info.ram";

/// Allows the plugin to read OS information.
pub const SYS_INFO_OS: &str = "sys.info.os";
