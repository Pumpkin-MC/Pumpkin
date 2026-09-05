use std::{
    collections::HashMap,
    io::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use tokio::{net::UdpSocket, sync::mpsc};
use tracing::trace;

use crate::{STOP_INTERRUPT, net::bedrock::status::IceSocket};

enum Command {
    Register {
        ufrag: String,
        internal: SocketAddr,
        candidates: Vec<SocketAddr>,
    },
    Remove {
        ufrag: String,
        internal: SocketAddr,
    },
}

struct Route {
    ufrag: String,
    remote: Option<SocketAddr>,
    candidates: Vec<SocketAddr>,
}

/// Routes every direct-IP peer through Pumpkin's one public Bedrock UDP socket.
pub(super) struct IceRouter {
    internal_addr: SocketAddr,
    public_addr: SocketAddr,
    commands: mpsc::UnboundedSender<Command>,
}

pub(super) struct Registration {
    ufrag: String,
    internal: SocketAddr,
    commands: mpsc::UnboundedSender<Command>,
}

impl Drop for Registration {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Remove {
            ufrag: self.ufrag.clone(),
            internal: self.internal,
        });
    }
}

impl IceRouter {
    pub(super) async fn bind(public: IceSocket) -> Result<Self, Error> {
        let public_addr = public.local_addr()?;
        let internal = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).await?;
        let internal_addr = internal.local_addr()?;
        let (commands, receiver) = mpsc::unbounded_channel();
        tokio::spawn(run(public, internal, receiver));
        Ok(Self {
            internal_addr,
            public_addr,
            commands,
        })
    }

    pub(super) const fn internal_addr(&self) -> SocketAddr {
        self.internal_addr
    }

    pub(super) const fn public_addr(&self) -> SocketAddr {
        self.public_addr
    }

    pub(super) fn register(
        &self,
        ufrag: String,
        internal: SocketAddr,
        candidates: Vec<SocketAddr>,
    ) -> Registration {
        let _ = self.commands.send(Command::Register {
            ufrag: ufrag.clone(),
            internal,
            candidates,
        });
        Registration {
            ufrag,
            internal,
            commands: self.commands.clone(),
        }
    }
}

async fn run(
    public: IceSocket,
    internal_socket: UdpSocket,
    mut commands: mpsc::UnboundedReceiver<Command>,
) {
    let mut routes = HashMap::<SocketAddr, Route>::new();
    let mut by_ufrag = HashMap::<String, SocketAddr>::new();
    let mut by_remote = HashMap::<SocketAddr, SocketAddr>::new();
    let mut public_buffer = vec![0; u16::MAX as usize];
    let mut internal_buffer = vec![0; u16::MAX as usize];

    loop {
        tokio::select! {
            () = STOP_INTERRUPT.cancelled() => break,
            Some(command) = commands.recv() => match command {
                Command::Register { ufrag, internal, candidates } => {
                    if let Some(old) = routes.insert(internal, Route {
                        ufrag: ufrag.clone(),
                        remote: None,
                        candidates,
                    }) {
                        by_ufrag.remove(&old.ufrag);
                        by_remote.retain(|_, route| *route != internal);
                    }
                    by_ufrag.insert(ufrag, internal);
                }
                Command::Remove { ufrag, internal } => {
                    if routes.get(&internal).is_some_and(|route| route.ufrag == ufrag) {
                        routes.remove(&internal);
                        by_ufrag.remove(&ufrag);
                        by_remote.retain(|_, route| *route != internal);
                    }
                }
            },
            result = public.recv_from(&mut public_buffer) => match result {
                Ok((length, remote)) => {
                    let packet = &public_buffer[..length];
                    let route = stun_username(packet)
                        .and_then(|username| {
                            username
                                .split(':')
                                .find_map(|ufrag| by_ufrag.get(ufrag).copied())
                        })
                        .or_else(|| by_remote.get(&remote).copied());
                    if let Some(internal) = route {
                        if let Some(route) = routes.get_mut(&internal) {
                            route.remote = Some(remote);
                        }
                        by_remote.insert(remote, internal);
                        if let Err(error) = internal_socket.send_to(packet, internal).await {
                            trace!(%remote, %internal, %error, "Failed to route inbound NetherNet ICE packet");
                        }
                    } else {
                        trace!(%remote, length, "Dropped unroutable NetherNet ICE packet");
                    }
                }
                Err(error) => trace!(%error, "Failed to receive NetherNet ICE packet"),
            },
            result = internal_socket.recv_from(&mut internal_buffer) => match result {
                Ok((length, internal)) => {
                    let target = routes.get(&internal).and_then(|route| {
                        route.remote.or_else(|| route.candidates.first().copied())
                    });
                    if let Some(target) = target {
                        if let Err(error) = public.send_to(&internal_buffer[..length], target).await {
                            trace!(%target, %internal, %error, "Failed to route outbound NetherNet ICE packet");
                        }
                    } else {
                        trace!(%internal, length, "Dropped NetherNet ICE packet without a remote route");
                    }
                }
                Err(error) => trace!(%error, "Failed to receive internal NetherNet ICE packet"),
            },
        }
    }
}

fn stun_username(packet: &[u8]) -> Option<&str> {
    if packet.len() < 20 || packet.get(4..8)? != [0x21, 0x12, 0xa4, 0x42] {
        return None;
    }
    let attributes_length = usize::from(u16::from_be_bytes(packet.get(2..4)?.try_into().ok()?));
    let end = 20usize.checked_add(attributes_length)?;
    if end > packet.len() {
        return None;
    }

    let mut offset = 20usize;
    while offset.checked_add(4)? <= end {
        let attribute = u16::from_be_bytes(packet.get(offset..offset + 2)?.try_into().ok()?);
        let length = usize::from(u16::from_be_bytes(
            packet.get(offset + 2..offset + 4)?.try_into().ok()?,
        ));
        let value_start = offset + 4;
        let value_end = value_start.checked_add(length)?;
        if value_end > end {
            return None;
        }
        if attribute == 0x0006 {
            return std::str::from_utf8(packet.get(value_start..value_end)?).ok();
        }
        offset = value_start.checked_add(length.next_multiple_of(4))?;
    }
    None
}

pub(super) fn proxy_offer(sdp: &str, proxy: SocketAddr) -> (String, Vec<SocketAddr>) {
    let mut candidates = Vec::new();
    let sdp = rewrite_sdp_candidates(sdp, |fields| {
        if is_component_one_udp(fields) {
            if let Some(candidate) = candidate_address(fields) {
                candidates.push(candidate);
            }
            fields[4] = proxy.ip().to_string();
            fields[5] = proxy.port().to_string();
        }
    });
    (sdp, candidates)
}

pub(super) fn proxy_answer(
    sdp: &str,
    public_port: u16,
) -> Result<(String, String, SocketAddr), String> {
    let ufrag = sdp
        .lines()
        .find_map(|line| line.strip_prefix("a=ice-ufrag:"))
        .map(str::to_owned)
        .ok_or_else(|| "WebRTC answer has no ICE username fragment".to_string())?;
    let candidate = sdp
        .lines()
        .filter_map(|line| line.strip_prefix("a=candidate:"))
        .map(|candidate| {
            candidate
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .find(|fields| is_component_one_udp(fields))
        .and_then(|fields| candidate_address(&fields))
        .ok_or_else(|| "WebRTC answer has no UDP host candidate".to_string())?;
    let internal = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), candidate.port());
    let answer = rewrite_sdp_candidates(sdp, |fields| {
        if is_component_one_udp(fields) {
            fields[5] = public_port.to_string();
        }
    });
    Ok((answer, ufrag, internal))
}

fn rewrite_sdp_candidates(sdp: &str, mut rewrite: impl FnMut(&mut Vec<String>)) -> String {
    let mut rewritten = String::with_capacity(sdp.len());
    for line in sdp.lines() {
        if let Some(candidate) = line.strip_prefix("a=candidate:") {
            let mut fields = candidate
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            rewrite(&mut fields);
            rewritten.push_str("a=candidate:");
            rewritten.push_str(&fields.join(" "));
        } else {
            rewritten.push_str(line);
        }
        rewritten.push_str("\r\n");
    }
    rewritten
}

fn is_component_one_udp(fields: &[String]) -> bool {
    fields.len() >= 8 && fields[1] == "1" && fields[2].eq_ignore_ascii_case("udp")
}

fn candidate_address(fields: &[String]) -> Option<SocketAddr> {
    Some(SocketAddr::new(
        fields.get(4)?.parse().ok()?,
        fields.get(5)?.parse().ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_stun_username() {
        let username = b"server:client";
        let mut packet = vec![0, 1, 0, 20, 0x21, 0x12, 0xa4, 0x42];
        packet.extend_from_slice(&[0; 12]);
        packet.extend_from_slice(&0x0006u16.to_be_bytes());
        packet.extend_from_slice(&(username.len() as u16).to_be_bytes());
        packet.extend_from_slice(username);
        packet.resize(packet.len().next_multiple_of(4), 0);
        assert_eq!(stun_username(&packet), Some("server:client"));
    }

    #[test]
    fn proxies_offer_and_answer_candidates() {
        let offer =
            "v=0\r\na=ice-ufrag:client\r\na=candidate:1 1 udp 123 192.168.1.7 50000 typ host\r\n";
        let proxy = "127.0.0.1:40000".parse().unwrap();
        let (offer, candidates) = proxy_offer(offer, proxy);
        assert!(offer.contains("127.0.0.1 40000 typ host"));
        assert_eq!(
            candidates,
            ["192.168.1.7:50000".parse::<SocketAddr>().unwrap()]
        );

        let answer =
            "v=0\r\na=ice-ufrag:server\r\na=candidate:2 1 udp 456 192.168.1.8 51000 typ host\r\n";
        let (answer, ufrag, internal) = proxy_answer(answer, 19132).unwrap();
        assert!(answer.contains("192.168.1.8 19132 typ host"));
        assert_eq!(ufrag, "server");
        assert_eq!(internal, "127.0.0.1:51000".parse::<SocketAddr>().unwrap());
    }
}
