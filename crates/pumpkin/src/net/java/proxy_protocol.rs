use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use pumpkin_config::networking::java::ProxyProtocolConfig;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

const HEADER_LENGTH: usize = 16;
const SIGNATURE_LENGTH: usize = 12;
const VERSION_COMMAND_INDEX: usize = 12;
const FAMILY_PROTOCOL_INDEX: usize = 13;
const LENGTH_INDEX: usize = 14;
const IPV4_ADDRESS_BLOCK_LENGTH: usize = 12;
const IPV6_ADDRESS_BLOCK_LENGTH: usize = 36;
const IPV6_BYTES_LENGTH: usize = 16;
const UNIX_ADDRESS_BLOCK_LENGTH: usize = 216;
const DISCARD_BUFFER_LENGTH: usize = 512;

const LOCAL_COMMAND: u8 = 0x20;
const PROXY_COMMAND: u8 = 0x21;
const UNSPECIFIED: u8 = 0x00;
const INET_STREAM: u8 = 0x11;
const INET_DATAGRAM: u8 = 0x12;
const INET6_STREAM: u8 = 0x21;
const INET6_DATAGRAM: u8 = 0x22;
const UNIX_STREAM: u8 = 0x31;
const UNIX_DATAGRAM: u8 = 0x32;

const SIGNATURE: &[u8; SIGNATURE_LENGTH] = b"\r\n\r\n\0\r\nQUIT\n";

#[derive(Debug, PartialEq, Eq)]
pub struct Endpoints {
    pub source: SocketAddr,
    pub destination: SocketAddr,
}

#[derive(Debug, Error)]
pub enum ProxyProtocolError {
    #[error("untrusted TCP peer")]
    Untrusted,
    #[error("header deadline exceeded")]
    Timeout,
    #[error("server shutting down")]
    Cancelled,
    #[error("invalid signature")]
    Signature,
    #[error("invalid version or command")]
    VersionCommand,
    #[error("reserved address family or transport protocol")]
    FamilyProtocol,
    #[error("address payload too short")]
    Length,
    #[error("incomplete header or socket read failure")]
    Read(#[from] std::io::Error),
}

pub async fn accept<R: AsyncRead + Unpin>(
    stream: &mut R,
    peer: SocketAddr,
    config: &ProxyProtocolConfig,
    shutdown: &CancellationToken,
) -> Result<Option<Endpoints>, ProxyProtocolError> {
    if !config.enabled {
        return Ok(None);
    }
    if !config.trusts(peer.ip()) {
        return Err(ProxyProtocolError::Untrusted);
    }
    tokio::select! {
        biased;
        () = shutdown.cancelled() => Err(ProxyProtocolError::Cancelled),
        result = timeout(
            Duration::from_millis(config.header_timeout_ms), read_header(stream),
        ) => result.map_err(|_| ProxyProtocolError::Timeout)?,
    }
}

async fn read_header<R: AsyncRead + Unpin>(
    stream: &mut R,
) -> Result<Option<Endpoints>, ProxyProtocolError> {
    let mut header = [0; HEADER_LENGTH];
    stream.read_exact(&mut header).await?;
    if &header[..SIGNATURE_LENGTH] != SIGNATURE {
        return Err(ProxyProtocolError::Signature);
    }
    if !matches!(header[VERSION_COMMAND_INDEX], LOCAL_COMMAND | PROXY_COMMAND) {
        return Err(ProxyProtocolError::VersionCommand);
    }
    let length = usize::from(u16::from_be_bytes([
        header[LENGTH_INDEX],
        header[LENGTH_INDEX + 1],
    ]));
    // Validate the fixed address block for every legal transport, but decode
    // only TCP/IPv4 and TCP/IPv6. Other valid transports are opaque to Java.
    let (required_address_length, parsed_address_length) =
        if header[VERSION_COMMAND_INDEX] == LOCAL_COMMAND {
            (0, 0)
        } else {
            match header[FAMILY_PROTOCOL_INDEX] {
                UNSPECIFIED => (0, 0),
                INET_STREAM => (IPV4_ADDRESS_BLOCK_LENGTH, IPV4_ADDRESS_BLOCK_LENGTH),
                INET_DATAGRAM => (IPV4_ADDRESS_BLOCK_LENGTH, 0),
                INET6_STREAM => (IPV6_ADDRESS_BLOCK_LENGTH, IPV6_ADDRESS_BLOCK_LENGTH),
                INET6_DATAGRAM => (IPV6_ADDRESS_BLOCK_LENGTH, 0),
                UNIX_STREAM | UNIX_DATAGRAM => (UNIX_ADDRESS_BLOCK_LENGTH, 0),
                _ => return Err(ProxyProtocolError::FamilyProtocol),
            }
        };
    if length < required_address_length {
        return Err(ProxyProtocolError::Length);
    }
    let mut address = [0; IPV6_ADDRESS_BLOCK_LENGTH];
    stream
        .read_exact(&mut address[..parsed_address_length])
        .await?;
    let endpoints = match parsed_address_length {
        IPV4_ADDRESS_BLOCK_LENGTH => Some(Endpoints {
            source: SocketAddr::new(
                Ipv4Addr::new(address[0], address[1], address[2], address[3]).into(),
                u16::from_be_bytes([address[8], address[9]]),
            ),
            destination: SocketAddr::new(
                Ipv4Addr::new(address[4], address[5], address[6], address[7]).into(),
                u16::from_be_bytes([address[10], address[11]]),
            ),
        }),
        IPV6_ADDRESS_BLOCK_LENGTH => {
            let mut source = [0; IPV6_BYTES_LENGTH];
            let mut destination = [0; IPV6_BYTES_LENGTH];
            source.copy_from_slice(&address[..IPV6_BYTES_LENGTH]);
            destination.copy_from_slice(&address[IPV6_BYTES_LENGTH..2 * IPV6_BYTES_LENGTH]);
            Some(Endpoints {
                source: SocketAddr::new(
                    Ipv6Addr::from(source).into(),
                    u16::from_be_bytes([address[32], address[33]]),
                ),
                destination: SocketAddr::new(
                    Ipv6Addr::from(destination).into(),
                    u16::from_be_bytes([address[34], address[35]]),
                ),
            })
        }
        _ => None,
    };
    let mut remaining = length - parsed_address_length;
    // The length covers the address block and optional TLVs. Consume the
    // remainder before Minecraft reads from the stream.
    let mut discard = [0; DISCARD_BUFFER_LENGTH];
    while remaining > 0 {
        let count = remaining.min(discard.len());
        stream.read_exact(&mut discard[..count]).await?;
        remaining -= count;
    }
    Ok(endpoints)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    fn config() -> ProxyProtocolConfig {
        ProxyProtocolConfig {
            enabled: true,
            trusted_proxies: vec!["127.0.0.0/8".parse().unwrap()],
            header_timeout_ms: 100,
        }
    }

    fn peer() -> SocketAddr {
        "127.0.0.1:40000".parse().unwrap()
    }

    fn header(command: u8, family: u8, payload: &[u8]) -> Vec<u8> {
        let mut bytes = SIGNATURE.to_vec();
        bytes.extend([command, family]);
        bytes.extend(u16::try_from(payload.len()).unwrap().to_be_bytes());
        bytes.extend(payload);
        bytes
    }

    fn ipv4() -> Vec<u8> {
        header(
            PROXY_COMMAND,
            INET_STREAM,
            &[192, 0, 2, 42, 203, 0, 113, 5, 0x9c, 0x40, 0x63, 0xdd],
        )
    }

    async fn parse(bytes: &[u8]) -> Result<Option<Endpoints>, ProxyProtocolError> {
        let mut stream = bytes;
        accept(&mut stream, peer(), &config(), &CancellationToken::new()).await
    }

    #[tokio::test]
    async fn ipv4_and_ipv6_endpoints() {
        assert_eq!(
            parse(&ipv4()).await.unwrap().unwrap(),
            Endpoints {
                source: "192.0.2.42:40000".parse().unwrap(),
                destination: "203.0.113.5:25565".parse().unwrap(),
            }
        );
        let mut payload = Vec::new();
        payload.extend("2001:db8::1234".parse::<Ipv6Addr>().unwrap().octets());
        payload.extend("2001:db8:1::5678".parse::<Ipv6Addr>().unwrap().octets());
        payload.extend([0xff, 0xff, 0, 0]);
        assert_eq!(
            parse(&header(PROXY_COMMAND, INET6_STREAM, &payload))
                .await
                .unwrap()
                .unwrap(),
            Endpoints {
                source: "[2001:db8::1234]:65535".parse().unwrap(),
                destination: "[2001:db8:1::5678]:0".parse().unwrap(),
            }
        );
    }

    #[tokio::test]
    async fn local_and_unsupported_preserve_stream_boundary() {
        for (command, family, length) in [
            (LOCAL_COMMAND, 0xff, 0),
            (LOCAL_COMMAND, 0xff, 1024),
            (PROXY_COMMAND, UNSPECIFIED, 0),
            (PROXY_COMMAND, INET_DATAGRAM, 12),
            (PROXY_COMMAND, INET6_DATAGRAM, 36),
            (PROXY_COMMAND, UNIX_STREAM, 216),
            (PROXY_COMMAND, UNIX_DATAGRAM, 216),
            (LOCAL_COMMAND, INET_STREAM, 65535),
        ] {
            let mut bytes = header(command, family, &vec![0x55; length]);
            bytes.extend([1, 0]);
            let mut stream = bytes.as_slice();
            assert_eq!(
                accept(&mut stream, peer(), &config(), &CancellationToken::new())
                    .await
                    .unwrap(),
                None
            );
            assert_eq!(stream, [1, 0]);
        }
    }

    #[tokio::test]
    async fn malformed_and_truncated_headers() {
        let mut bytes = ipv4();
        bytes[0] = 0;
        assert!(matches!(
            parse(&bytes).await,
            Err(ProxyProtocolError::Signature)
        ));
        for command in [0x11, 0x22, 0x2f, 0x30] {
            assert!(matches!(
                parse(&header(command, 0, &[])).await,
                Err(ProxyProtocolError::VersionCommand)
            ));
        }
        for family in [0x10, 0x01, 0x20, 0x30, 0x40, 0x13, 0x03, 0xf1, 0xff] {
            assert!(matches!(
                parse(&header(PROXY_COMMAND, family, &[])).await,
                Err(ProxyProtocolError::FamilyProtocol)
            ));
        }
        for (family, length) in [
            (INET_STREAM, 11),
            (INET_DATAGRAM, 11),
            (INET6_STREAM, 35),
            (INET6_DATAGRAM, 35),
            (UNIX_STREAM, 215),
            (UNIX_DATAGRAM, 215),
        ] {
            assert!(matches!(
                parse(&header(PROXY_COMMAND, family, &vec![0; length])).await,
                Err(ProxyProtocolError::Length)
            ));
        }
        let bytes = ipv4();
        for length in 0..bytes.len() {
            assert!(
                matches!(parse(&bytes[..length]).await, Err(ProxyProtocolError::Read(error)) if error.kind() == std::io::ErrorKind::UnexpectedEof)
            );
        }
        let bytes = header(LOCAL_COMMAND, UNSPECIFIED, &[1, 2, 3]);
        assert!(matches!(
            parse(&bytes[..18]).await,
            Err(ProxyProtocolError::Read(_))
        ));
        assert!(
            parse(b"PROXY TCP4 127.0.0.1 127.0.0.1 1 2\r\n")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn fragmented_header_and_extensions() {
        let (mut sender, mut receiver) = tokio::io::duplex(1);
        let mut bytes = ipv4();
        bytes[15] = 15;
        bytes.extend([0xea, 0, 0, 1, 0]);
        let writer = tokio::spawn(async move {
            for byte in bytes {
                sender.write_all(&[byte]).await.unwrap();
                tokio::task::yield_now().await;
            }
        });
        let endpoints = accept(&mut receiver, peer(), &config(), &CancellationToken::new())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            endpoints.source,
            "192.0.2.42:40000".parse::<SocketAddr>().unwrap()
        );
        let mut minecraft = [0; 2];
        receiver.read_exact(&mut minecraft).await.unwrap();
        assert_eq!(minecraft, [1, 0]);
        writer.await.unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn silent_and_trickling_connections_share_one_deadline() {
        for trickle in [false, true] {
            let (mut sender, mut receiver) = tokio::io::duplex(64);
            let writer = tokio::spawn(async move {
                if trickle {
                    for byte in ipv4() {
                        if sender.write_all(&[byte]).await.is_err() {
                            return;
                        }
                        tokio::time::sleep(Duration::from_millis(30)).await;
                    }
                } else {
                    std::future::pending::<()>().await;
                }
            });
            let start = tokio::time::Instant::now();
            assert!(matches!(
                accept(&mut receiver, peer(), &config(), &CancellationToken::new()).await,
                Err(ProxyProtocolError::Timeout)
            ));
            assert_eq!(start.elapsed(), Duration::from_millis(100));
            writer.abort();
            let _ = writer.await;
        }
    }

    #[tokio::test]
    async fn disabled_untrusted_and_cancelled_do_not_read() {
        let mut stream = &b"ordinary Minecraft bytes"[..];
        let original = stream;
        assert_eq!(
            accept(
                &mut stream,
                peer(),
                &ProxyProtocolConfig::default(),
                &CancellationToken::new()
            )
            .await
            .unwrap(),
            None
        );
        assert_eq!(stream, original);
        assert!(matches!(
            accept(
                &mut stream,
                "192.0.2.1:1".parse().unwrap(),
                &config(),
                &CancellationToken::new()
            )
            .await,
            Err(ProxyProtocolError::Untrusted)
        ));
        assert_eq!(stream, original);
        let shutdown = CancellationToken::new();
        shutdown.cancel();
        assert!(matches!(
            accept(&mut stream, peer(), &config(), &shutdown).await,
            Err(ProxyProtocolError::Cancelled)
        ));
        assert_eq!(stream, original);
        assert!(
            accept(&mut stream, peer(), &config(), &CancellationToken::new())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn loopback_propagates_address_and_decodes_minecraft() {
        use crate::data::{banlist_serializer::BannedIpEntry, banned_ip::BannedIpList};
        use crate::net::{ClientPlatform, java::JavaClient};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut sender = tokio::net::TcpStream::connect(listener.local_addr().unwrap())
            .await
            .unwrap();
        let (mut receiver, transport_peer) = listener.accept().await.unwrap();
        let mut bytes = ipv4();
        bytes.extend([1, 0]);
        sender.write_all(&bytes).await.unwrap();
        let forwarded = accept(
            &mut receiver,
            transport_peer,
            &config(),
            &CancellationToken::new(),
        )
        .await
        .unwrap()
        .unwrap();
        let mut pending = super::super::pending::PendingConnection::new(
            receiver,
            transport_peer,
            1,
            crate::net::PacketRateLimiter::from_config(
                &pumpkin_config::PacketLimiterConfig::default(),
            ),
        );
        pending.address = forwarded.source;
        let packet = timeout(Duration::from_secs(1), pending.get_packet())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(packet.id, 0);
        let profile = crate::net::GameProfile {
            id: uuid::Uuid::nil(),
            name: "ProxyTest".into(),
            properties: arc_swap::ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        };
        let client =
            JavaClient::from_pending(pending, profile, crate::net::PlayerConfig::default());
        assert_eq!(client.transport_peer, transport_peer);
        let platform = ClientPlatform::Java(client);
        let address = platform.address();
        assert_eq!(address, forwarded.source);
        let mut bans = BannedIpList {
            banned_ips: vec![BannedIpEntry::new(
                address.ip(),
                "test".into(),
                None,
                "test".into(),
            )],
        };
        assert!(bans.get_entry(&address.ip()).is_some());
        assert!(bans.get_entry(&transport_peer.ip()).is_none());
    }

    #[tokio::test]
    async fn loopback_direct_connections_require_explicit_disabled_mode() {
        for enabled in [false, true] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let mut sender = tokio::net::TcpStream::connect(listener.local_addr().unwrap())
                .await
                .unwrap();
            let (mut receiver, transport_peer) = listener.accept().await.unwrap();
            sender.write_all(&[1, 0]).await.unwrap();
            sender.shutdown().await.unwrap();
            let config = ProxyProtocolConfig {
                enabled,
                ..config()
            };
            let result = accept(
                &mut receiver,
                transport_peer,
                &config,
                &CancellationToken::new(),
            )
            .await;
            if enabled {
                assert!(matches!(result, Err(ProxyProtocolError::Read(_))));
            } else {
                assert_eq!(result.unwrap(), None);
                let mut client = super::super::pending::PendingConnection::new(
                    receiver,
                    transport_peer,
                    1,
                    crate::net::PacketRateLimiter::from_config(
                        &pumpkin_config::PacketLimiterConfig::default(),
                    ),
                );
                assert_eq!(client.get_packet().await.unwrap().id, 0);
                assert_eq!(client.address, transport_peer);
            }
        }
    }
}
