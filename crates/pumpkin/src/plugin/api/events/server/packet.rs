use crate::net::user::User;
use bytes::{Buf, Bytes};
use pumpkin_macros::{Event, cancellable};
use pumpkin_protocol::{ConnectionState, MAX_PACKET_DATA_SIZE, RawPacket, codec::var_int::VarInt};
use std::{io, sync::Arc};

tokio::task_local! { static PACKET_CALLBACK: (); }

pub(crate) fn in_packet_callback() -> bool {
    PACKET_CALLBACK.try_with(|()| ()).is_ok() || pumpkin_plugin_runtime::is_guest_call_active()
}

/// Body bytes exclude the packet ID, framing, compression and encryption.
#[cancellable]
#[derive(Event, Clone)]
pub struct PacketReceivedEvent {
    pub user: Arc<User>,
    pub state: ConnectionState,
    pub protocol_version: Option<i32>,
    pub version: pumpkin_util::version::JavaMinecraftVersion,
    pub packet_id: i32,
    pub payload: Bytes,
}

impl PacketReceivedEvent {
    pub fn new(user: Arc<User>, packet_id: i32, payload: Bytes) -> Self {
        Self {
            state: user.decoder_state.load(),
            protocol_version: user.protocol_version.load(),
            version: user.java_version.load(),
            user,
            packet_id,
            payload,
            cancelled: false,
        }
    }

    pub async fn filter(user: &Arc<User>, packet: RawPacket) -> Option<RawPacket> {
        let server = user.server.upgrade()?;
        if !server.plugin_manager.has_handlers::<Self>() {
            return Some(packet);
        }
        let mut event = Self::new(user.clone(), packet.id, packet.payload);
        PACKET_CALLBACK
            .scope((), server.plugin_manager.fire(&server, &mut event))
            .await;
        if event.cancelled || event.packet_id < 0 || event.payload.len() > MAX_PACKET_DATA_SIZE {
            return None;
        }
        Some(RawPacket {
            id: event.packet_id,
            payload: event.payload,
        })
    }
}

/// Runs before writing a packet, not after delivery to the client.
#[cancellable]
#[derive(Event, Clone)]
pub struct PacketSentEvent {
    pub user: Arc<User>,
    pub state: ConnectionState,
    pub protocol_version: Option<i32>,
    pub version: pumpkin_util::version::JavaMinecraftVersion,
    pub packet_id: i32,
    pub payload: Bytes,
}

impl PacketSentEvent {
    pub fn new(user: Arc<User>, packet_id: i32, payload: Bytes) -> Self {
        Self {
            state: user.encoder_state.load(),
            protocol_version: user.protocol_version.load(),
            version: user.java_version.load(),
            user,
            packet_id,
            payload,
            cancelled: false,
        }
    }

    pub async fn filter(user: &Arc<User>, packet: RawPacket, silent: bool) -> Option<RawPacket> {
        let server = user.server.upgrade()?;
        if silent || !server.plugin_manager.has_handlers::<Self>() {
            return Some(packet);
        }
        let mut event = Self::new(user.clone(), packet.id, packet.payload);
        PACKET_CALLBACK
            .scope((), server.plugin_manager.fire(&server, &mut event))
            .await;
        if event.cancelled || event.packet_id < 0 || event.payload.len() > MAX_PACKET_DATA_SIZE {
            return None;
        }
        Some(RawPacket {
            id: event.packet_id,
            payload: event.payload,
        })
    }
}

pub(crate) fn decode_packet(mut data: Bytes) -> io::Result<RawPacket> {
    let mut body = data.as_ref();
    let id = VarInt::decode(&mut body)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?
        .0;
    if id < 0 || body.len() > MAX_PACKET_DATA_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid packet ID or size",
        ));
    }
    let header_len = data.len() - body.len();
    data.advance(header_len);
    Ok(RawPacket { id, payload: data })
}

pub(crate) fn encode_packet(id: i32, body: &[u8]) -> io::Result<Bytes> {
    if id < 0 || body.len() > MAX_PACKET_DATA_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid packet ID or size",
        ));
    }
    let mut data = Vec::with_capacity(VarInt(id).written_size() + body.len());
    VarInt(id).encode(&mut data).map_err(io::Error::other)?;
    data.extend_from_slice(body);
    Ok(data.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::user::Edition;
    use std::sync::Weak;

    #[test]
    fn packet_body_round_trip_and_state_snapshot() {
        for id in [0, 127, 128, 16_384, i32::MAX] {
            let packet = decode_packet(encode_packet(id, &[4, 5, 6]).unwrap()).unwrap();
            assert_eq!(packet.id, id);
            assert_eq!(packet.payload.as_ref(), &[4, 5, 6]);
        }
        for bytes in [&[][..], &[0x80], &[0xff; 5]] {
            assert!(decode_packet(Bytes::copy_from_slice(bytes)).is_err());
        }
        assert!(encode_packet(-1, &[]).is_err());
        let user = User::new(
            "127.0.0.1:25565".parse().unwrap(),
            Edition::Java,
            Weak::new(),
        );
        user.decoder_state.store(ConnectionState::Login);
        user.encoder_state.store(ConnectionState::Config);
        let received = PacketReceivedEvent::new(user.clone(), 0, Bytes::new());
        let sent = PacketSentEvent::new(user.clone(), 0, Bytes::new());
        user.decoder_state.store(ConnectionState::Play);
        user.encoder_state.store(ConnectionState::Play);
        assert_eq!(received.state, ConnectionState::Login);
        assert_eq!(sent.state, ConnectionState::Config);
        assert!(user.player().is_none());
        assert!(Arc::ptr_eq(&received.user, &sent.user));
    }
}
