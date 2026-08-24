use std::fmt;

use crate::wit::pumpkin::plugin::event::ServerListPingAddress;

impl fmt::Display for ServerListPingAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.host.contains(':') {
            write!(f, "[{}]:{}", self.host, self.port)
        } else {
            write!(f, "{}:{}", self.host, self.port)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_ipv4_address() {
        let address = ServerListPingAddress {
            host: "127.0.0.1".into(),
            port: 25565,
        };

        assert_eq!(address.to_string(), "127.0.0.1:25565");
    }

    #[test]
    fn formats_ipv6_address() {
        let address = ServerListPingAddress {
            host: "::1".into(),
            port: 25565,
        };

        assert_eq!(address.to_string(), "[::1]:25565");
    }
}
