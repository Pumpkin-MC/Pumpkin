use std::io::{Error, Read};

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketRead};

#[packet(135)]
pub struct SClientCacheBlobStatus {
    pub miss_hashes: Vec<u64>,
    pub hit_hashes: Vec<u64>,
}

impl PacketRead for SClientCacheBlobStatus {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let miss_count = VarUInt::read(reader)?.0 as usize;
        let mut miss_hashes = Vec::with_capacity(miss_count);
        for _ in 0..miss_count {
            let mut bytes = [0u8; 8];
            reader.read_exact(&mut bytes)?;
            miss_hashes.push(u64::from_le_bytes(bytes));
        }

        let hit_count = VarUInt::read(reader)?.0 as usize;
        let mut hit_hashes = Vec::with_capacity(hit_count);
        for _ in 0..hit_count {
            let mut bytes = [0u8; 8];
            reader.read_exact(&mut bytes)?;
            hit_hashes.push(u64::from_le_bytes(bytes));
        }

        Ok(Self {
            miss_hashes,
            hit_hashes,
        })
    }
}
