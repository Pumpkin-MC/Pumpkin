mod server_data;

pub use server_data::{CONNECTION_TYPE_LAN_SIGNALING, ServerData, TRANSPORT_LAYER_NETHERNET};

use super::crypto::{self, CHECKSUM_SIZE};

const HEADER_PADDING: usize = 8;

const REQUEST_ID: u16 = 0x00;
const RESPONSE_ID: u16 = 0x01;
const MESSAGE_ID: u16 = 0x02;

pub enum DiscoveryPacket {
    Request,
    Response { application_data: Vec<u8> },
    Message { recipient_id: u64, data: String },
}

impl DiscoveryPacket {
    const fn id(&self) -> u16 {
        match self {
            Self::Request => REQUEST_ID,
            Self::Response { .. } => RESPONSE_ID,
            Self::Message { .. } => MESSAGE_ID,
        }
    }

    fn encode_payload(&self, out: &mut Vec<u8>) {
        match self {
            Self::Request => {}
            Self::Response { application_data } => {
                let hex = hex::encode(application_data);
                out.extend_from_slice(&(hex.len() as u32).to_le_bytes());
                out.extend_from_slice(hex.as_bytes());
            }
            Self::Message { recipient_id, data } => {
                out.extend_from_slice(&recipient_id.to_le_bytes());
                out.extend_from_slice(&(data.len() as u32).to_le_bytes());
                out.extend_from_slice(data.as_bytes());
            }
        }
    }

    fn decode_payload(id: u16, reader: &mut Reader) -> Option<Self> {
        match id {
            REQUEST_ID => Some(Self::Request),
            RESPONSE_ID => {
                let length = reader.read_u32()? as usize;
                let hex = reader.take(length)?;
                Some(Self::Response {
                    application_data: hex::decode(hex).ok()?,
                })
            }
            MESSAGE_ID => {
                let recipient_id = reader.read_u64()?;
                let length = reader.read_u32()? as usize;
                let data = reader.take(length)?;
                Some(Self::Message {
                    recipient_id,
                    data: String::from_utf8(data.to_vec()).ok()?,
                })
            }
            _ => None,
        }
    }
}

pub fn marshal(packet: &DiscoveryPacket, sender_id: u64) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&packet.id().to_le_bytes());
    body.extend_from_slice(&sender_id.to_le_bytes());
    body.extend_from_slice(&[0; HEADER_PADDING]);
    packet.encode_payload(&mut body);

    let mut payload = Vec::with_capacity(body.len() + 2);
    payload.extend_from_slice(&((body.len() + 2) as u16).to_le_bytes());
    payload.extend_from_slice(&body);

    let mut datagram = crypto::checksum(&payload).to_vec();
    datagram.extend_from_slice(&crypto::encrypt(&payload));
    datagram
}

pub fn unmarshal(datagram: &[u8]) -> Option<(DiscoveryPacket, u64)> {
    if datagram.len() < CHECKSUM_SIZE {
        return None;
    }
    let (checksum, ciphertext) = datagram.split_at(CHECKSUM_SIZE);
    let payload = crypto::decrypt(ciphertext)?;
    if crypto::checksum(&payload).as_slice() != checksum {
        return None;
    }

    let mut reader = Reader::new(&payload);
    reader.read_u16()?;
    let packet_id = reader.read_u16()?;
    let sender_id = reader.read_u64()?;
    reader.take(HEADER_PADDING)?;
    let packet = DiscoveryPacket::decode_payload(packet_id, &mut reader)?;
    Some((packet, sender_id))
}

struct Reader<'a> {
    data: &'a [u8],
}

impl<'a> Reader<'a> {
    const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    const fn take(&mut self, length: usize) -> Option<&'a [u8]> {
        if self.data.len() < length {
            return None;
        }
        let (taken, rest) = self.data.split_at(length);
        self.data = rest;
        Some(taken)
    }

    fn read_u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }

    fn read_u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn read_u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_round_trips() {
        let datagram = marshal(&DiscoveryPacket::Request, 42);
        let (packet, sender_id) = unmarshal(&datagram).unwrap();
        assert_eq!(sender_id, 42);
        assert!(matches!(packet, DiscoveryPacket::Request));
    }

    #[test]
    fn response_round_trips_hex_encoded_application_data() {
        let datagram = marshal(
            &DiscoveryPacket::Response {
                application_data: vec![0, 1, 2, 255],
            },
            7,
        );
        let (packet, _) = unmarshal(&datagram).unwrap();
        let DiscoveryPacket::Response { application_data } = packet else {
            panic!("expected a response packet");
        };
        assert_eq!(application_data, vec![0, 1, 2, 255]);
    }

    #[test]
    fn message_round_trips() {
        let datagram = marshal(
            &DiscoveryPacket::Message {
                recipient_id: u64::MAX,
                data: "CONNECTREQUEST 1 v=0".to_string(),
            },
            7,
        );
        let (packet, _) = unmarshal(&datagram).unwrap();
        let DiscoveryPacket::Message { recipient_id, data } = packet else {
            panic!("expected a message packet");
        };
        assert_eq!(recipient_id, u64::MAX);
        assert_eq!(data, "CONNECTREQUEST 1 v=0");
    }

    #[test]
    fn rejects_datagrams_with_a_broken_checksum() {
        let mut datagram = marshal(&DiscoveryPacket::Request, 1);
        datagram[0] ^= 0xff;
        assert!(unmarshal(&datagram).is_none());
        assert!(unmarshal(&[]).is_none());
    }
}
