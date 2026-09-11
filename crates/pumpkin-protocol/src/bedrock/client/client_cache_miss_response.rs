// Last verified for v2169

use pumpkin_macros::packet;

use crate::serial::PacketWrite;

#[derive(PacketWrite, Clone, Debug)]
pub struct MissingBlobData {
    pub blob_id: u64,
    pub blob_data: Vec<u8>,
}

#[derive(PacketWrite)]
#[packet(136)]
pub struct CClientCacheMissResponse {
    pub missing_blobs: Vec<MissingBlobData>,
}
