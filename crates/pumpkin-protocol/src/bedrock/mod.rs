pub mod client;
pub mod enum_as_str;
pub mod network_item;
pub mod packet_decoder;
pub mod packet_encoder;
pub mod server;
pub mod status;

pub const BEDROCK_GAME_PACKET: u8 = 0xfe;

#[repr(u16)]
pub enum SubClient {
    Main = 0,
    SubClient0 = 1,
    SubClient1 = 2,
    SubClietn2 = 3,
}
