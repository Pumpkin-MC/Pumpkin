pub mod ack;
pub mod client;
pub mod frame_set;
pub mod network_item;
pub mod packet_decoder;
pub mod packet_encoder;
pub mod server;

pub const UDP_HEADER_SIZE: u16 = 28;
pub const MTU: usize = 1400;

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
    #[must_use]
    pub fn is_reliable(&self) -> bool {
        matches!(
            self,
            Self::Reliable
                | Self::ReliableOrdered
                | Self::ReliableSequenced
                | Self::ReliableWithAckReceipt
                | Self::ReliableOrderedWithAckReceipt
        )
    }

    #[must_use]
    pub fn is_sequenced(&self) -> bool {
        matches!(self, Self::ReliableSequenced | Self::UnreliableSequenced)
    }

    #[must_use]
    pub fn is_ordered(&self) -> bool {
        matches!(
            self,
            Self::UnreliableSequenced
                | Self::ReliableOrdered
                | Self::ReliableSequenced
                | Self::ReliableOrderedWithAckReceipt
        )
    }

    #[must_use]
    pub fn is_order_exclusive(&self) -> bool {
        matches!(
            self,
            Self::ReliableOrdered | Self::ReliableOrderedWithAckReceipt
        )
    }

    #[must_use]
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::Unreliable),
            1 => Some(Self::UnreliableSequenced),
            2 => Some(Self::Reliable),
            3 => Some(Self::ReliableOrdered),
            4 => Some(Self::ReliableSequenced),
            5 => Some(Self::UnreliableWithAckReceipt),
            6 => Some(Self::ReliableWithAckReceipt),
            7 => Some(Self::ReliableOrderedWithAckReceipt),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_id(&self) -> u8 {
        match self {
            Self::Unreliable => 0,
            Self::UnreliableSequenced => 1,
            Self::Reliable => 2,
            Self::ReliableOrdered => 3,
            Self::ReliableSequenced => 4,
            Self::UnreliableWithAckReceipt => 5,
            Self::ReliableWithAckReceipt => 6,
            Self::ReliableOrderedWithAckReceipt => 7,
        }
    }
}

#[repr(u16)]
pub enum SubClient {
    Main = 0,
    SubClient0 = 1,
    SubClient1 = 2,
    SubClietn2 = 3,
}
