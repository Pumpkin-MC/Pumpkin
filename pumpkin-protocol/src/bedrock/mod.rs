pub mod ack;
pub mod client;
pub mod frame_set;
pub mod packet_decoder;
pub mod packet_encoder;
pub mod server;

pub const UDP_HEADER_SIZE: u16 = 28;

pub const RAKNET_MAGIC: [u8; 16] = [
    0x00, 0xff, 0xff, 0x0, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

pub const RAKNET_VALID: u8 = 0x80;
pub const RAKNET_ACK: u8 = 0xC0;
pub const RAKNET_NACK: u8 = 0xA0;

pub const RAKNET_GAME_PACKET: i32 = 0xfe;

pub const RAKNET_SPLIT: u8 = 0x10;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
pub enum RakReliability {
    Unreliable,
    UnreliableSequenced,
    Reliable,
    #[default]
    ReliableOrdered,
    ReliableSequenced,
    UnreliableWithAckReceipt,
    ReliableWithAckReceipt,
    ReliableOrderedWithAckReceipt,
}

impl RakReliability {
    pub fn is_reliable(&self) -> bool {
        matches!(
            self,
            RakReliability::Reliable
                | RakReliability::ReliableOrdered
                | RakReliability::ReliableSequenced
                | RakReliability::ReliableWithAckReceipt
                | RakReliability::ReliableOrderedWithAckReceipt
        )
    }

    pub fn is_sequenced(&self) -> bool {
        matches!(
            self,
            RakReliability::ReliableSequenced | RakReliability::UnreliableSequenced
        )
    }

    pub fn is_ordered(&self) -> bool {
        matches!(
            self,
            RakReliability::UnreliableSequenced
                | RakReliability::ReliableOrdered
                | RakReliability::ReliableSequenced
                | RakReliability::ReliableOrderedWithAckReceipt
        )
    }

    pub fn is_order_exclusive(&self) -> bool {
        matches!(
            self,
            RakReliability::ReliableOrdered | RakReliability::ReliableOrderedWithAckReceipt
        )
    }

    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(RakReliability::Unreliable),
            1 => Some(RakReliability::UnreliableSequenced),
            2 => Some(RakReliability::Reliable),
            3 => Some(RakReliability::ReliableOrdered),
            4 => Some(RakReliability::ReliableSequenced),
            5 => Some(RakReliability::UnreliableWithAckReceipt),
            6 => Some(RakReliability::ReliableWithAckReceipt),
            7 => Some(RakReliability::ReliableOrderedWithAckReceipt),
            _ => None,
        }
    }

    pub fn to_id(&self) -> u8 {
        match self {
            RakReliability::Unreliable => 0,
            RakReliability::UnreliableSequenced => 1,
            RakReliability::Reliable => 2,
            RakReliability::ReliableOrdered => 3,
            RakReliability::ReliableSequenced => 4,
            RakReliability::UnreliableWithAckReceipt => 5,
            RakReliability::ReliableWithAckReceipt => 6,
            RakReliability::ReliableOrderedWithAckReceipt => 7,
        }
    }
}
