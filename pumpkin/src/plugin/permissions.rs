/// Allows the plugin to perform DNS resolution (resolving hostnames to IP addresses).
///
/// Corresponds to the `wasi:sockets/ip-name-lookup` interface.
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
///
/// This gives the plugin full access to the host's network interfaces.
/// **Warning:** This is a powerful permission and should only be granted to trusted plugins.
pub const NETWORK_OUTBOUND: &str = "network.outbound";

/// Allows the plugin to read files from the server's file system outside of its data folder.
pub const FS_READ: &str = "fs.read";

/// Allows the plugin to write files to the server's file system outside of its data folder.
pub const FS_WRITE: &str = "fs.write";

/// Allows the plugin to read files within its own data folder (`plugins/<name>`).
/// This is granted by default if any other FS permission is not specified, but can be explicitly requested.
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

#[must_use]
pub fn get_permission_description(permission: &str) -> Option<&'static str> {
    match permission {
        NETWORK_DNS => Some(
            "Allows the plugin to perform DNS resolution (resolving hostnames to IP addresses).",
        ),
        NETWORK_TCP => Some("Allows the plugin to use TCP sockets."),
        NETWORK_UDP => Some("Allows the plugin to use UDP sockets."),
        NETWORK_TCP_CONNECT => Some("Allows the plugin to initiate TCP connections."),
        NETWORK_TCP_BIND => {
            Some("Allows the plugin to bind TCP listeners (accept inbound connections).")
        }
        NETWORK_UDP_CONNECT => {
            Some("Allows the plugin to send and receive UDP packets to specific destinations.")
        }
        NETWORK_UDP_BIND => Some("Allows the plugin to bind UDP sockets to local ports."),
        NETWORK_UDP_OUTGOING_DATAGRAM => {
            Some("Allows the plugin to send datagram on non-connected UDP socket.")
        }
        NETWORK_LOOPBACK => {
            Some("Restricts all networking permissions to loopback addresses (localhost) only.")
        }
        NETWORK_OUTBOUND => {
            Some("Allows the plugin to make outbound TCP/UDP connections. (POWERFUL)")
        }
        FS_READ => Some(
            "Allows the plugin to read files from the server's file system outside of its data folder.",
        ),
        FS_WRITE => Some(
            "Allows the plugin to write files to the server's file system outside of its data folder.",
        ),
        FS_READ_DATA => Some("Allows the plugin to read files within its own data folder."),
        FS_WRITE_DATA => Some("Allows the plugin to write files within its own data folder."),
        SYS_ENV => Some("Allows the plugin to read all environment variables."),
        SYS_INFO => Some("Allows the plugin to read system information (CPU, Memory, OS)."),
        SYS_INFO_CPU => Some("Allows the plugin to read CPU information."),
        SYS_INFO_RAM => Some("Allows the plugin to read RAM information."),
        SYS_INFO_OS => Some("Allows the plugin to read OS information."),
        p if p.starts_with(SYS_ENV_PREFIX) => {
            Some("Allows the plugin to read specific environment variables.")
        }
        _ => None,
    }
}
