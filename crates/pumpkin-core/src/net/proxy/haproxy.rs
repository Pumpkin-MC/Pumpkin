use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

const V1_PREFIX: [u8; 6] = *b"PROXY ";
const V2_SIGNATURE: [u8; 12] = *b"\r\n\r\n\x00\r\nQUIT\n";

/// Maximum length of a v1 header is 107 bytes, including the 6-byte `"PROXY "` prefix.
const MAX_V1_BODY_LEN: usize = 101;

const V2_VERSION: u8 = 2;
const V2_CMD_LOCAL: u8 = 0x0;
const V2_CMD_PROXY: u8 = 0x1;

const V2_FAM_UNSPEC: u8 = 0x00;
const V2_FAM_INET_STREAM: u8 = 0x11;
const V2_FAM_INET_DGRAM: u8 = 0x12;
const V2_FAM_INET6_STREAM: u8 = 0x21;
const V2_FAM_INET6_DGRAM: u8 = 0x22;
const V2_FAM_UNIX_STREAM: u8 = 0x31;
const V2_FAM_UNIX_DGRAM: u8 = 0x32;

const IPV4_ADDR_LEN: usize = 12;
const IPV6_ADDR_LEN: usize = 36;
const UNIX_ADDR_LEN: usize = 216;

#[derive(Error, Debug)]
pub enum HAProxyError {
    #[error("I/O error while reading HAProxy header: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid HAProxy protocol signature")]
    InvalidSignature,
    #[error("Invalid HAProxy v1 header")]
    InvalidV1Header,
    #[error("Unsupported HAProxy v2 version: {0}")]
    UnsupportedVersion(u8),
    #[error("Invalid HAProxy v2 header")]
    InvalidV2Header,
}

/// Reads an `HAProxy` PROXY protocol (v1 or v2) header from the start of a stream.
///
/// Returns `Ok(Some(addr))` when the header carries a forwarded IPv4/IPv6 client address,
/// or `Ok(None)` when the connection uses `UNKNOWN` / `LOCAL` / `AF_UNSPEC` / `AF_UNIX`
/// (in which case the caller keeps the existing socket address).
pub async fn read_header<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<Option<SocketAddr>, HAProxyError> {
    let mut prefix = [0u8; 6];
    reader.read_exact(&mut prefix).await?;

    if prefix == V1_PREFIX {
        read_v1_header(reader).await
    } else if prefix == V2_SIGNATURE[..6] {
        read_v2_header(reader).await
    } else {
        Err(HAProxyError::InvalidSignature)
    }
}

async fn read_v1_header<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<Option<SocketAddr>, HAProxyError> {
    let mut buf = [0u8; MAX_V1_BODY_LEN];
    let mut len = 0;

    loop {
        if len >= MAX_V1_BODY_LEN {
            return Err(HAProxyError::InvalidV1Header);
        }
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte).await?;
        buf[len] = byte[0];
        len += 1;

        if byte[0] == b'\n' {
            if len < 2 || buf[len - 2] != b'\r' {
                return Err(HAProxyError::InvalidV1Header);
            }
            break;
        }
    }

    parse_v1_line(&buf[..len - 2])
}

fn parse_v1_line(line: &[u8]) -> Result<Option<SocketAddr>, HAProxyError> {
    if line.contains(&b'\r') {
        return Err(HAProxyError::InvalidV1Header);
    }
    let line = core::str::from_utf8(line).map_err(|_| HAProxyError::InvalidV1Header)?;

    if line == "UNKNOWN" || line.starts_with("UNKNOWN ") {
        return Ok(None);
    }

    let mut parts = line.split(' ');
    let protocol = parts.next().ok_or(HAProxyError::InvalidV1Header)?;
    let src_ip_str = parts.next().ok_or(HAProxyError::InvalidV1Header)?;
    let dst_ip_str = parts.next().ok_or(HAProxyError::InvalidV1Header)?;
    let src_port_str = parts.next().ok_or(HAProxyError::InvalidV1Header)?;
    let dst_port_str = parts.next().ok_or(HAProxyError::InvalidV1Header)?;
    if parts.next().is_some() {
        return Err(HAProxyError::InvalidV1Header);
    }

    let src_port = parse_v1_port(src_port_str)?;
    let _dst_port = parse_v1_port(dst_port_str)?;

    match protocol {
        "TCP4" => {
            let src_ip: Ipv4Addr = src_ip_str
                .parse()
                .map_err(|_| HAProxyError::InvalidV1Header)?;
            let _dst_ip: Ipv4Addr = dst_ip_str
                .parse()
                .map_err(|_| HAProxyError::InvalidV1Header)?;
            Ok(Some(SocketAddr::new(IpAddr::V4(src_ip), src_port)))
        }
        "TCP6" => {
            let src_ip: Ipv6Addr = src_ip_str
                .parse()
                .map_err(|_| HAProxyError::InvalidV1Header)?;
            let _dst_ip: Ipv6Addr = dst_ip_str
                .parse()
                .map_err(|_| HAProxyError::InvalidV1Header)?;
            Ok(Some(SocketAddr::new(IpAddr::V6(src_ip), src_port)))
        }
        _ => Err(HAProxyError::InvalidV1Header),
    }
}

fn parse_v1_port(s: &str) -> Result<u16, HAProxyError> {
    if s.is_empty() || s.starts_with('0') || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(HAProxyError::InvalidV1Header);
    }
    s.parse::<u16>().map_err(|_| HAProxyError::InvalidV1Header)
}

async fn read_v2_header<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<Option<SocketAddr>, HAProxyError> {
    let mut header = [0u8; 10];
    reader.read_exact(&mut header).await?;

    if header[..6] != V2_SIGNATURE[6..12] {
        return Err(HAProxyError::InvalidSignature);
    }

    let ver_cmd = header[6];
    let version = ver_cmd >> 4;
    if version != V2_VERSION {
        return Err(HAProxyError::UnsupportedVersion(version));
    }

    let command = ver_cmd & 0x0F;
    let fam_proto = header[7];
    let payload_len = u16::from_be_bytes([header[8], header[9]]) as usize;

    match command {
        V2_CMD_LOCAL => {
            skip_bytes(reader, payload_len).await?;
            Ok(None)
        }
        V2_CMD_PROXY => match fam_proto {
            V2_FAM_UNSPEC => {
                skip_bytes(reader, payload_len).await?;
                Ok(None)
            }
            V2_FAM_INET_STREAM | V2_FAM_INET_DGRAM => {
                if payload_len < IPV4_ADDR_LEN {
                    return Err(HAProxyError::InvalidV2Header);
                }
                let mut addr_buf = [0u8; IPV4_ADDR_LEN];
                reader.read_exact(&mut addr_buf).await?;
                skip_bytes(reader, payload_len - IPV4_ADDR_LEN).await?;

                let src_ip = Ipv4Addr::new(addr_buf[0], addr_buf[1], addr_buf[2], addr_buf[3]);
                let src_port = u16::from_be_bytes([addr_buf[8], addr_buf[9]]);
                Ok(Some(SocketAddr::new(IpAddr::V4(src_ip), src_port)))
            }
            V2_FAM_INET6_STREAM | V2_FAM_INET6_DGRAM => {
                if payload_len < IPV6_ADDR_LEN {
                    return Err(HAProxyError::InvalidV2Header);
                }
                let mut addr_buf = [0u8; IPV6_ADDR_LEN];
                reader.read_exact(&mut addr_buf).await?;
                skip_bytes(reader, payload_len - IPV6_ADDR_LEN).await?;

                let mut src_octets = [0u8; 16];
                src_octets.copy_from_slice(&addr_buf[..16]);
                let src_ip = Ipv6Addr::from(src_octets);
                let src_port = u16::from_be_bytes([addr_buf[32], addr_buf[33]]);
                Ok(Some(SocketAddr::new(IpAddr::V6(src_ip), src_port)))
            }
            V2_FAM_UNIX_STREAM | V2_FAM_UNIX_DGRAM => {
                if payload_len < UNIX_ADDR_LEN {
                    return Err(HAProxyError::InvalidV2Header);
                }
                skip_bytes(reader, payload_len).await?;
                Ok(None)
            }
            _ => Err(HAProxyError::InvalidV2Header),
        },
        _ => Err(HAProxyError::InvalidV2Header),
    }
}

async fn skip_bytes<R: AsyncRead + Unpin>(
    reader: &mut R,
    mut remaining: usize,
) -> Result<(), std::io::Error> {
    let mut scratch = [0u8; 256];
    while remaining > 0 {
        let chunk = remaining.min(scratch.len());
        reader.read_exact(&mut scratch[..chunk]).await?;
        remaining -= chunk;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parses_v1_tcp4_and_preserves_trailing_stream() {
        let mut stream =
            &b"PROXY TCP4 203.0.113.195 198.51.100.2 56324 25565\r\nminecraft-payload"[..];

        let addr = read_header(&mut stream)
            .await
            .expect("valid v1 TCP4 header")
            .expect("forwarded address");

        assert_eq!(addr, "203.0.113.195:56324".parse::<SocketAddr>().unwrap());
        assert_eq!(stream, b"minecraft-payload");
    }

    #[tokio::test]
    async fn parses_v1_tcp6() {
        let mut stream = &b"PROXY TCP6 2001:db8::1 2001:db8::2 43210 25565\r\n"[..];

        let addr = read_header(&mut stream)
            .await
            .expect("valid v1 TCP6 header")
            .expect("forwarded address");

        assert_eq!(addr, "[2001:db8::1]:43210".parse::<SocketAddr>().unwrap());
    }

    #[tokio::test]
    async fn parses_v1_unknown_as_none() {
        let mut bare = &b"PROXY UNKNOWN\r\nnext"[..];
        assert_eq!(read_header(&mut bare).await.unwrap(), None);
        assert_eq!(bare, b"next");

        let mut with_addrs =
            &b"PROXY UNKNOWN ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff 65535 65535\r\nnext"[..];
        assert_eq!(read_header(&mut with_addrs).await.unwrap(), None);
        assert_eq!(with_addrs, b"next");
    }

    #[tokio::test]
    async fn rejects_malformed_v1_headers() {
        // Missing CRLF
        let mut bare_lf = &b"PROXY TCP4 203.0.113.1 198.51.100.1 12345 25565\n"[..];
        assert!(matches!(
            read_header(&mut bare_lf).await,
            Err(HAProxyError::InvalidV1Header)
        ));

        // Leading zero in port
        let mut leading_zero = &b"PROXY TCP4 203.0.113.1 198.51.100.1 01234 25565\r\n"[..];
        assert!(matches!(
            read_header(&mut leading_zero).await,
            Err(HAProxyError::InvalidV1Header)
        ));

        // Plus sign in port
        let mut plus_port = &b"PROXY TCP4 203.0.113.1 198.51.100.1 +1234 25565\r\n"[..];
        assert!(matches!(
            read_header(&mut plus_port).await,
            Err(HAProxyError::InvalidV1Header)
        ));

        // Direct Minecraft handshake without PROXY header
        let mut direct_mc = &b"\x10\x00\xfb\x05\tlocalhost\x63\xdd\x01"[..];
        assert!(matches!(
            read_header(&mut direct_mc).await,
            Err(HAProxyError::InvalidSignature)
        ));
    }

    #[tokio::test]
    async fn parses_v2_ipv4_with_tlvs_and_preserves_trailing_stream() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&V2_SIGNATURE);
        bytes.push(0x21); // v2, PROXY
        bytes.push(0x11); // AF_INET, STREAM
        // 12 bytes IPv4 + 4 bytes dummy TLV = 16 bytes
        bytes.extend_from_slice(&16u16.to_be_bytes());
        bytes.extend_from_slice(&[203, 0, 113, 42]); // src ip
        bytes.extend_from_slice(&[198, 51, 100, 1]); // dst ip
        bytes.extend_from_slice(&54321u16.to_be_bytes()); // src port
        bytes.extend_from_slice(&25565u16.to_be_bytes()); // dst port
        bytes.extend_from_slice(&[0x01, 0x00, 0x01, 0x02]); // TLV
        bytes.extend_from_slice(b"handshake");

        let mut slice = &bytes[..];
        let addr = read_header(&mut slice)
            .await
            .expect("valid v2 IPv4 header")
            .expect("forwarded address");

        assert_eq!(addr, "203.0.113.42:54321".parse::<SocketAddr>().unwrap());
        assert_eq!(slice, b"handshake");
    }

    #[tokio::test]
    async fn parses_v2_ipv6() {
        let src_ip: Ipv6Addr = "2001:db8::abcd".parse().unwrap();
        let dst_ip: Ipv6Addr = "2001:db8::1".parse().unwrap();

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&V2_SIGNATURE);
        bytes.push(0x21); // v2, PROXY
        bytes.push(0x21); // AF_INET6, STREAM
        bytes.extend_from_slice(&36u16.to_be_bytes());
        bytes.extend_from_slice(&src_ip.octets());
        bytes.extend_from_slice(&dst_ip.octets());
        bytes.extend_from_slice(&60000u16.to_be_bytes());
        bytes.extend_from_slice(&25565u16.to_be_bytes());

        let mut slice = &bytes[..];
        let addr = read_header(&mut slice)
            .await
            .expect("valid v2 IPv6 header")
            .expect("forwarded address");

        assert_eq!(addr, SocketAddr::new(IpAddr::V6(src_ip), 60000));
    }

    #[tokio::test]
    async fn parses_v2_local_as_none() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&V2_SIGNATURE);
        bytes.push(0x20); // v2, LOCAL
        bytes.push(0x00); // AF_UNSPEC
        bytes.extend_from_slice(&3u16.to_be_bytes());
        bytes.extend_from_slice(&[0xAA, 0xBB, 0xCC]);
        bytes.extend_from_slice(b"after");

        let mut slice = &bytes[..];
        assert_eq!(read_header(&mut slice).await.unwrap(), None);
        assert_eq!(slice, b"after");
    }

    #[tokio::test]
    async fn rejects_truncated_v2_address_payload() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&V2_SIGNATURE);
        bytes.push(0x21); // v2, PROXY
        bytes.push(0x11); // AF_INET, STREAM (requires >= 12 bytes)
        bytes.extend_from_slice(&8u16.to_be_bytes());
        bytes.extend_from_slice(&[0u8; 8]);

        let mut slice = &bytes[..];
        assert!(matches!(
            read_header(&mut slice).await,
            Err(HAProxyError::InvalidV2Header)
        ));
    }
}
