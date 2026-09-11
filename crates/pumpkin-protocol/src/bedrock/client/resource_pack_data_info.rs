use pumpkin_macros::packet;

use crate::serial::PacketWrite;

#[derive(PacketWrite)]
#[packet(82)]
pub struct CResourcePackDataInfo {
    pub resource_name: String,
    pub chunk_size: u32,
    pub number_of_chunks: u32,
    pub file_size: u64,
    pub file_hash: Vec<u8>,
    pub is_premium_pack: bool,
    pub pack_type: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `file_hash` is a Bedrock byte array, meaning a `VarUInt` length followed by the raw bytes.
    /// The derive adds that prefix for `Vec` fields, so pin the frame to catch a silent format
    /// change that would break resource pack delivery to clients.
    #[test]
    fn file_hash_is_length_prefixed() {
        let packet = CResourcePackDataInfo {
            resource_name: "pack_1.0.8".to_string(),
            chunk_size: 1_048_576,
            number_of_chunks: 2,
            file_size: 87_255,
            file_hash: vec![0xde, 0xad, 0xbe, 0xef],
            is_premium_pack: true,
            pack_type: 6,
        };

        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();

        assert_eq!(
            buf,
            vec![
                0x0a, b'p', b'a', b'c', b'k', b'_', b'1', b'.', b'0', b'.',
                b'8', // resource name
                0x00, 0x00, 0x10, 0x00, // chunk size
                0x02, 0x00, 0x00, 0x00, // number of chunks
                0xd7, 0x54, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, // file size
                0x04, 0xde, 0xad, 0xbe, 0xef, // file hash length and bytes
                0x01, // premium pack
                0x06, // pack type
            ]
        );
    }
}
