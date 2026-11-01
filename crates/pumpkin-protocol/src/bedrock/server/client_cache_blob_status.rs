// Last verified for v2169

use std::io::{Error, Read};

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketRead};

/// Sent by the client to report cache hits and misses for chunk blobs requested by the server.
#[packet(135)]
pub struct SClientCacheBlobStatus {
    /// Hashes of chunk blobs not found in the client cache, which the server must send.
    pub miss_hashes: Vec<u64>,
    /// Hashes of chunk blobs successfully retrieved from the client cache.
    pub hit_hashes: Vec<u64>,
}

impl PacketRead for SClientCacheBlobStatus {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let miss_count = VarUInt::read(reader)?.0 as usize;
        if miss_count > 4096 {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "miss_count exceeds limit",
            ));
        }
        let mut miss_hashes = Vec::with_capacity(miss_count.min(256));
        for _ in 0..miss_count {
            miss_hashes.push(u64::read(reader)?);
        }

        let hit_count = VarUInt::read(reader)?.0 as usize;
        if hit_count > 4096 {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "hit_count exceeds limit",
            ));
        }
        let mut hit_hashes = Vec::with_capacity(hit_count.min(256));
        for _ in 0..hit_count {
            hit_hashes.push(u64::read(reader)?);
        }

        Ok(Self {
            miss_hashes,
            hit_hashes,
        })
    }
}
