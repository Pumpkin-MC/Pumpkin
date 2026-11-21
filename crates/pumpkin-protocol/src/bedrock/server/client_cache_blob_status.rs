// Last verified for v2169

use std::io::{Error, Read};

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketRead};

/// Docs: both id lists hold at most 4095 entries.
const MAX_BLOB_IDS: usize = 4095;

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
        if miss_count > MAX_BLOB_IDS {
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
        if hit_count > MAX_BLOB_IDS {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serial::PacketWrite;

    /// Both lists with the given counts, every id zero.
    fn payload(miss_count: usize, hit_count: usize) -> Vec<u8> {
        let mut buf = Vec::new();
        for count in [miss_count, hit_count] {
            VarUInt(count as u32).write(&mut buf).unwrap();
            buf.resize(buf.len() + count * size_of::<u64>(), 0);
        }
        buf
    }

    #[test]
    fn id_lists_hold_at_most_4095_entries() {
        let full =
            SClientCacheBlobStatus::read(&mut payload(MAX_BLOB_IDS, MAX_BLOB_IDS).as_slice())
                .unwrap();
        assert_eq!(full.miss_hashes.len(), MAX_BLOB_IDS);
        assert_eq!(full.hit_hashes.len(), MAX_BLOB_IDS);

        for (miss_count, hit_count) in [(MAX_BLOB_IDS + 1, 0), (0, MAX_BLOB_IDS + 1)] {
            let err = SClientCacheBlobStatus::read(&mut payload(miss_count, hit_count).as_slice())
                .err()
                .unwrap();
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        }
    }
}
