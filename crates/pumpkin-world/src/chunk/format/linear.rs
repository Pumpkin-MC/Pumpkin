use rustc_hash::FxHashMap;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::chunk::io::{RegionFile, sync_file, write_file_atomically};
use crate::chunk::{ChunkReadingError, ChunkWritingError};
use bytes::{Buf, BufMut, Bytes};
use pumpkin_util::math::vector2::Vector2;
use ruzstd::decoding::StreamingDecoder;
use ruzstd::encoding::{CompressionLevel, compress_to_vec};
use tracing::{error, warn};
use xxhash_rust::xxh64::xxh64;

use super::anvil::CHUNK_COUNT;

/// The signature used as both header and footer.
/// <https://gist.github.com/Aaron2550/5701519671253d4c6190bde6706f9f98>
const SIGNATURE: [u8; 8] = u64::to_be_bytes(0xc3ff13183cca9d9a);

/// Allowed grid sizes for v2 bucket subdivision.
const VALID_GRID_SIZES: &[u8] = &[1, 2, 4, 8, 16, 32];

/// Default grid size.  2×2 = 4 buckets gives a reasonable write amplification
/// tradeoff for most worlds.
const DEFAULT_GRID_SIZE: u8 = 2;

// ---------------------------------------------------------------------------
// Superblock  (26 bytes)
// ---------------------------------------------------------------------------
//
//  0.. 8   uint64  Signature  (0xc3ff13183cca9d9a)
//  8.. 9   uint8   Version    (2)
//  9..17   uint64  Newest timestamp
// 17..18   int8    Grid size
// 18..22   int32   Region X
// 22..26   int32   Region Z

struct LinearV2Superblock {
    newest_timestamp: u64,
    /// One of {1, 2, 4, 8, 16, 32}.
    grid_size: u8,
    region_x: i32,
    region_z: i32,
}

impl LinearV2Superblock {
    const SIZE: usize = 26;

    fn from_bytes(buf: &[u8]) -> Result<Self, ChunkReadingError> {
        if buf.len() < Self::SIZE {
            return Err(ChunkReadingError::IoError(std::io::Error::from(
                ErrorKind::UnexpectedEof,
            )));
        }
        let mut b = buf;
        let mut sig = [0u8; 8];
        sig.copy_from_slice(&b[..8]);
        b = &b[8..];

        if sig != SIGNATURE {
            error!("Linear v2: invalid superblock signature");
            return Err(ChunkReadingError::InvalidHeader);
        }

        let version = b.get_u8();
        if version != 0x02 {
            error!("Linear v2: unexpected version byte {version:#x} in superblock");
            return Err(ChunkReadingError::InvalidHeader);
        }

        let newest_timestamp = b.get_u64();
        let grid_size = b.get_u8();
        let region_x = b.get_i32();
        let region_z = b.get_i32();

        if !VALID_GRID_SIZES.contains(&grid_size) {
            error!("Linear v2: invalid grid size {grid_size}");
            return Err(ChunkReadingError::InvalidHeader);
        }

        Ok(Self {
            newest_timestamp,
            grid_size,
            region_x,
            region_z,
        })
    }

    fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut out = [0u8; Self::SIZE];
        let mut b = out.as_mut_slice();
        b.put_slice(&SIGNATURE);
        b.put_u8(0x02);
        b.put_u64(self.newest_timestamp);
        b.put_u8(self.grid_size);
        b.put_i32(self.region_x);
        b.put_i32(self.region_z);
        out
    }
}

// ---------------------------------------------------------------------------
// Chunk existence bitmap  (128 bytes = 1024 bits)
// ---------------------------------------------------------------------------

struct ChunkBitmap([u8; 128]);

impl ChunkBitmap {
    const SIZE: usize = 128;

    const fn new() -> Self {
        Self([0u8; 128])
    }

    const fn set(&mut self, index: usize, exists: bool) {
        let byte = index / 8;
        let bit = index % 8;
        if exists {
            self.0[byte] |= 1 << bit;
        } else {
            self.0[byte] &= !(1 << bit);
        }
    }

    #[allow(dead_code)]
    const fn get(&self, index: usize) -> bool {
        let byte = index / 8;
        let bit = index % 8;
        (self.0[byte] >> bit) & 1 == 1
    }
}

// ---------------------------------------------------------------------------
// NBT features dictionary
// ---------------------------------------------------------------------------
//
// Repeated: u8 key_len, key bytes, u32 value
// Terminated by a single 0x00 byte.

struct NbtFeatures(FxHashMap<String, u32>);

impl NbtFeatures {
    fn empty() -> Self {
        Self(FxHashMap::default())
    }

    fn from_bytes(buf: &mut impl Buf) -> Result<Self, ChunkReadingError> {
        let mut map = FxHashMap::default();
        loop {
            if !buf.has_remaining() {
                return Err(ChunkReadingError::IoError(std::io::Error::from(
                    ErrorKind::UnexpectedEof,
                )));
            }
            let key_len = buf.get_u8();
            if key_len == 0 {
                break;
            }
            if buf.remaining() < key_len as usize + 4 {
                return Err(ChunkReadingError::IoError(std::io::Error::from(
                    ErrorKind::UnexpectedEof,
                )));
            }
            let key_bytes: Vec<u8> = (0..key_len).map(|_| buf.get_u8()).collect();
            let key = String::from_utf8(key_bytes).map_err(|_| ChunkReadingError::InvalidHeader)?;
            let value = buf.get_u32();
            map.insert(key, value);
        }
        Ok(Self(map))
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for (key, value) in &self.0 {
            let key_bytes = key.as_bytes();
            out.push(key_bytes.len() as u8);
            out.extend_from_slice(key_bytes);
            out.extend_from_slice(&value.to_be_bytes());
        }
        out.push(0); // end marker
        out
    }
}

// ---------------------------------------------------------------------------
// Bucket size entry  (13 bytes per bucket)
// ---------------------------------------------------------------------------
//
//  0..4    uint32   Bucket size (compressed)
//  4..5    int8     Compression level
//  5..13   uint64   xxhash64 of compressed bucket data

struct BucketSizeEntry {
    size: u32,
    compression_level: i8,
    xxhash: u64,
}

impl BucketSizeEntry {
    const SIZE: usize = 13;

    fn from_bytes(buf: &mut impl Buf) -> Result<Self, ChunkReadingError> {
        if buf.remaining() < Self::SIZE {
            return Err(ChunkReadingError::IoError(std::io::Error::from(
                ErrorKind::UnexpectedEof,
            )));
        }
        Ok(Self {
            size: buf.get_u32(),
            compression_level: buf.get_i8(),
            xxhash: buf.get_u64(),
        })
    }

    fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut out = [0u8; Self::SIZE];
        let mut b = out.as_mut_slice();
        b.put_u32(self.size);
        b.put_i8(self.compression_level);
        b.put_u64(self.xxhash);
        out
    }
}

/// A bucket's data as stored in the file, with the hash its size entry records.
#[derive(Clone, PartialEq, Debug)]
struct CompressedBucket {
    data: Box<[u8]>,
    compression_level: i8,
    xxhash: u64,
}

// ---------------------------------------------------------------------------
// Per-chunk record inside a decompressed bucket
// ---------------------------------------------------------------------------
//
//  0.. 4   uint32   Chunk size (0 = absent)
//  4..12   uint64   Chunk timestamp
// 12..     bytes    Chunk NBT data  (only when size > 0)

struct BucketChunkEntry {
    timestamp: u64,
    data: Option<Bytes>,
}

impl BucketChunkEntry {
    /// Number of bytes for the fixed part (size + timestamp).
    const FIXED_SIZE: usize = 12;

    fn write_into(&self, out: &mut Vec<u8>) {
        match &self.data {
            None => {
                out.put_u32(0);
                out.put_u64(self.timestamp);
            }
            Some(bytes) => {
                out.put_u32(bytes.len() as u32);
                out.put_u64(self.timestamp);
                out.extend_from_slice(bytes);
            }
        }
    }

    fn read_from(buf: &mut Bytes) -> Result<Self, ChunkReadingError> {
        if buf.remaining() < Self::FIXED_SIZE {
            return Err(ChunkReadingError::IoError(std::io::Error::from(
                ErrorKind::UnexpectedEof,
            )));
        }
        let size = buf.get_u32() as usize;
        let timestamp = buf.get_u64();
        if size == 0 {
            return Ok(Self {
                timestamp,
                data: None,
            });
        }
        if buf.remaining() < size {
            warn!(
                "Linear v2: not enough bytes for chunk (need {size}, have {})",
                buf.remaining()
            );
            return Err(ChunkReadingError::IoError(std::io::Error::from(
                ErrorKind::UnexpectedEof,
            )));
        }
        let data = buf.split_to(size);
        Ok(Self {
            timestamp,
            data: Some(data),
        })
    }
}

// ---------------------------------------------------------------------------
// Main LinearV2File struct
// ---------------------------------------------------------------------------

pub struct LinearV2File {
    path: PathBuf,

    /// Region coordinates, needed to rebuild the superblock on write.
    region_x: i32,
    region_z: i32,

    /// Grid size (1, 2, 4, 8, 16, or 32).
    grid_size: u8,

    /// Per-chunk timestamps.
    timestamps: Box<[u64]>,

    /// Per-chunk NBT data (None = chunk absent).
    chunks_data: Box<[Option<Bytes>]>,

    /// The compressed bytes, compression level and xxhash64 of each bucket as last compressed.
    /// `None` marks a bucket with staged chunks, the only ones a flush compresses again.
    buckets: Vec<Option<CompressedBucket>>,
    staged: bool,
    /// The file was written without a sync since the last sync.
    unsynced: bool,
}

impl LinearV2File {
    fn empty(path: &Path, region: Vector2<i32>) -> Self {
        Self {
            path: path.to_path_buf(),
            region_x: region.x,
            region_z: region.y,
            grid_size: DEFAULT_GRID_SIZE,
            timestamps: vec![0; CHUNK_COUNT].into_boxed_slice(),
            chunks_data: vec![None; CHUNK_COUNT].into_boxed_slice(),
            buckets: vec![None; Self::bucket_count(DEFAULT_GRID_SIZE)],
            staged: false,
            unsynced: false,
        }
    }

    /// Number of buckets in the grid (`grid_size²`).
    const fn bucket_count(grid_size: u8) -> usize {
        (grid_size as usize) * (grid_size as usize)
    }

    /// Number of chunks in each bucket.
    const fn chunks_per_bucket(grid_size: u8) -> usize {
        CHUNK_COUNT / Self::bucket_count(grid_size)
    }

    /// Build the decompressed byte stream for bucket `bucket_idx`.
    fn serialise_bucket(&self, bucket_idx: usize) -> Vec<u8> {
        let cpb = Self::chunks_per_bucket(self.grid_size);
        let mut buf = Vec::new();

        for local in 0..cpb {
            // Recover the global chunk index from bucket + local.
            let chunk_index = Self::global_chunk_index(bucket_idx, local, self.grid_size);
            let entry = BucketChunkEntry {
                timestamp: self.timestamps[chunk_index],
                data: self.chunks_data[chunk_index].clone(),
            };
            entry.write_into(&mut buf);
        }
        buf
    }

    /// Inverse of `chunk_bucket_index` + `chunk_local_index`.
    const fn global_chunk_index(bucket_idx: usize, local: usize, grid_size: u8) -> usize {
        let stride = 32 / grid_size as usize;
        let bucket_row = bucket_idx / grid_size as usize;
        let bucket_col = bucket_idx % grid_size as usize;
        let local_row = local / stride;
        let local_col = local % stride;
        let cz = bucket_row * stride + local_row;
        let cx = bucket_col * stride + local_col;
        cz * 32 + cx
    }

    const fn chunk_bucket_index(chunk_index: usize, grid_size: u8) -> usize {
        let stride = 32 / grid_size as usize;
        let cx = chunk_index % 32;
        let cz = chunk_index / 32;
        (cz / stride) * grid_size as usize + (cx / stride)
    }

    fn parse(raw_file: Bytes, path: &Path) -> Result<Self, ChunkReadingError> {
        let mut buf = raw_file;

        if buf.remaining() < LinearV2Superblock::SIZE {
            return Err(ChunkReadingError::IoError(std::io::Error::from(
                ErrorKind::UnexpectedEof,
            )));
        }
        let superblock_bytes = buf.split_to(LinearV2Superblock::SIZE);
        let superblock = LinearV2Superblock::from_bytes(&superblock_bytes)?;
        let grid_size = superblock.grid_size;
        let bucket_count = Self::bucket_count(grid_size);

        if buf.remaining() < ChunkBitmap::SIZE {
            return Err(ChunkReadingError::IoError(std::io::Error::from(
                ErrorKind::UnexpectedEof,
            )));
        }
        // We read the bitmap but per the spec it is not yet reliable, so we
        // do not use it to short-circuit — actual presence is determined by
        // the per-chunk size field inside each bucket.
        let mut bitmap_raw = [0u8; ChunkBitmap::SIZE];
        bitmap_raw.copy_from_slice(&buf.split_to(ChunkBitmap::SIZE));
        //let _bitmap = ChunkBitmap(bitmap_raw);

        let _features = NbtFeatures::from_bytes(&mut buf)?;

        let total_bucket_meta = bucket_count * BucketSizeEntry::SIZE;
        if buf.remaining() < total_bucket_meta {
            return Err(ChunkReadingError::IoError(std::io::Error::from(
                ErrorKind::UnexpectedEof,
            )));
        }
        let mut bucket_entries: Vec<BucketSizeEntry> = Vec::with_capacity(bucket_count);
        for _ in 0..bucket_count {
            bucket_entries.push(BucketSizeEntry::from_bytes(&mut buf)?);
        }

        // Work out where the footer starts so we can validate it without
        // consuming the bucket data bytes yet.
        let total_compressed: usize = bucket_entries.iter().map(|e| e.size as usize).sum();
        if buf.remaining() < total_compressed + SIGNATURE.len() {
            return Err(ChunkReadingError::IoError(std::io::Error::from(
                ErrorKind::UnexpectedEof,
            )));
        }
        let footer_offset = total_compressed;
        {
            let footer = &buf[footer_offset..footer_offset + SIGNATURE.len()];
            if footer != SIGNATURE {
                error!("Linear v2: invalid footer signature");
                return Err(ChunkReadingError::InvalidHeader);
            }
        }

        let mut timestamps = vec![0u64; CHUNK_COUNT].into_boxed_slice();
        let mut chunks_data: Box<[Option<Bytes>]> = vec![None; CHUNK_COUNT].into_boxed_slice();
        let mut buckets = vec![None; bucket_count];

        for (bucket_idx, entry) in bucket_entries.iter().enumerate() {
            let compressed_size = entry.size as usize;

            let compressed_slice = &buf[..compressed_size];
            // A corrupt bucket keeps its bytes and stored hash, so a flush writes it back
            // unchanged until one of its chunks is saved.
            buckets[bucket_idx] = Some(CompressedBucket {
                data: Box::from(compressed_slice),
                compression_level: entry.compression_level,
                xxhash: entry.xxhash,
            });

            // xxhash64 integrity check.
            let actual_hash = xxh64(compressed_slice, 0);
            if actual_hash != entry.xxhash {
                error!(
                    "Linear v2: xxhash mismatch for bucket {bucket_idx} \
                     (expected {:#x}, got {:#x})",
                    entry.xxhash, actual_hash
                );
                // Skip the corrupted bucket rather than aborting the whole
                // file — the remaining buckets may be intact.
                buf.advance(compressed_size);
                continue;
            }

            let mut decompressed = Vec::new();
            {
                let mut decoder = StreamingDecoder::new(compressed_slice)
                    .map_err(|_| ChunkReadingError::RegionIsInvalid)?;
                decoder
                    .read_to_end(&mut decompressed)
                    .map_err(ChunkReadingError::IoError)?
            };
            buf.advance(compressed_size);

            let mut bucket_buf: Bytes = decompressed.into();
            let cpb = Self::chunks_per_bucket(grid_size);

            for local in 0..cpb {
                let chunk_index = Self::global_chunk_index(bucket_idx, local, grid_size);
                let record = BucketChunkEntry::read_from(&mut bucket_buf)?;
                timestamps[chunk_index] = record.timestamp;
                chunks_data[chunk_index] = record.data;
            }
        }

        Ok(Self {
            path: path.to_path_buf(),
            region_x: superblock.region_x,
            region_z: superblock.region_z,
            grid_size,
            timestamps,
            chunks_data,
            buckets,
            staged: false,
            unsynced: false,
        })
    }
}

impl RegionFile for LinearV2File {
    type Config = ();
    type Encoded = Bytes;
    type Raw = Bytes;

    // Every flush rewrites the whole file, so saves are collected for a while first.
    const FLUSH_DELAY: Duration = Duration::from_secs(30);
    // An open region holds all of its chunks uncompressed.
    const CACHE_CAPACITY: usize = 16;

    fn file_name(region: Vector2<i32>) -> String {
        format!("r.{}.{}.linear", region.x, region.y)
    }

    fn open(path: &Path, region: Vector2<i32>) -> Result<Self, ChunkReadingError> {
        match std::fs::read(path) {
            Ok(bytes) if bytes.is_empty() => Ok(Self::empty(path, region)),
            Ok(bytes) => Self::parse(bytes.into(), path),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(Self::empty(path, region)),
            Err(err) => Err(ChunkReadingError::IoError(err)),
        }
    }

    fn read_chunk(&self, index: usize) -> Result<Option<Bytes>, ChunkReadingError> {
        Ok(self.chunks_data[index].clone())
    }

    fn decode_chunk(raw: Bytes) -> Result<Bytes, ChunkReadingError> {
        Ok(raw)
    }

    fn encode_chunk(nbt: Bytes, _config: &()) -> Result<Bytes, ChunkWritingError> {
        Ok(nbt)
    }

    fn stage_chunk(&mut self, index: usize, chunk: Bytes) {
        self.timestamps[index] = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        self.chunks_data[index] = Some(chunk);
        self.buckets[Self::chunk_bucket_index(index, self.grid_size)] = None;
        self.staged = true;
    }

    fn has_staged(&self) -> bool {
        self.staged
    }

    fn flush(&mut self, sync: bool) -> std::io::Result<()> {
        if !self.staged {
            if sync && self.unsynced {
                sync_file(&self.path)?;
                self.unsynced = false;
            }
            return Ok(());
        }
        for bucket_idx in 0..self.buckets.len() {
            if self.buckets[bucket_idx].is_none() {
                let raw = self.serialise_bucket(bucket_idx);
                let data =
                    compress_to_vec(raw.as_slice(), CompressionLevel::Fastest).into_boxed_slice();
                let xxhash = xxh64(&data, 0);
                self.buckets[bucket_idx] = Some(CompressedBucket {
                    data,
                    compression_level: 1,
                    xxhash,
                });
            }
        }

        let superblock = LinearV2Superblock {
            newest_timestamp: self.timestamps.iter().copied().max().unwrap_or(0),
            grid_size: self.grid_size,
            region_x: self.region_x,
            region_z: self.region_z,
        };
        let mut bitmap = ChunkBitmap::new();
        for (index, chunk) in self.chunks_data.iter().enumerate() {
            if chunk.is_some() {
                bitmap.set(index, true);
            }
        }

        let mut out = Vec::new();
        out.extend_from_slice(&superblock.to_bytes());
        out.extend_from_slice(&bitmap.0);
        out.extend_from_slice(&NbtFeatures::empty().to_bytes());
        for bucket in self.buckets.iter().flatten() {
            let entry = BucketSizeEntry {
                size: bucket.data.len() as u32,
                compression_level: bucket.compression_level,
                xxhash: bucket.xxhash,
            };
            out.extend_from_slice(&entry.to_bytes());
        }
        for bucket in self.buckets.iter().flatten() {
            out.extend_from_slice(&bucket.data);
        }
        out.extend_from_slice(&SIGNATURE);

        write_file_atomically(&self.path, &out, sync)?;
        self.staged = false;
        self.unsynced = !sync;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a tiny chunk payload.
    fn fake_nbt(seed: u8) -> Bytes {
        Bytes::from(vec![seed; 64])
    }

    #[test]
    fn flush_recompresses_only_changed_buckets() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("r.3.-4.linear");
        let mut file = LinearV2File::open(&path, Vector2::new(3, -4)).unwrap();
        for index in (0..CHUNK_COUNT).step_by(9) {
            file.stage_chunk(index, fake_nbt(index as u8));
        }
        file.flush(false).unwrap();

        let mut file = LinearV2File::open(&path, Vector2::new(3, -4)).unwrap();
        let untouched = file.buckets[3].clone();
        file.stage_chunk(0, fake_nbt(0xEE));
        assert!(file.buckets[0].is_none());
        file.flush(false).unwrap();
        assert_eq!(file.buckets[3], untouched);

        let file = LinearV2File::open(&path, Vector2::new(3, -4)).unwrap();
        assert_eq!(file.read_chunk(0).unwrap().unwrap(), fake_nbt(0xEE));
        for index in (9..CHUNK_COUNT).step_by(9) {
            assert_eq!(
                file.read_chunk(index).unwrap().unwrap(),
                fake_nbt(index as u8)
            );
        }
        assert!(file.read_chunk(1).unwrap().is_none());
    }

    #[test]
    fn failed_flush_keeps_changes_pending_for_retry() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("r.0.0.linear");
        let mut file = LinearV2File::open(&path, Vector2::new(0, 0)).unwrap();
        file.flush(true).unwrap();
        assert!(!path.exists());
        file.stage_chunk(0, fake_nbt(1));
        file.flush(false).unwrap();
        file.stage_chunk(0, fake_nbt(2));

        std::fs::create_dir(path.with_extension("tmp")).unwrap();
        assert!(file.flush(false).is_err());
        assert!(file.has_staged());
        std::fs::remove_dir(path.with_extension("tmp")).unwrap();
        file.flush(true).unwrap();

        let reopened = LinearV2File::open(&path, Vector2::new(0, 0)).unwrap();
        assert_eq!(reopened.read_chunk(0).unwrap().unwrap(), fake_nbt(2));
    }

    #[test]
    fn corrupt_bucket_survives_flushes() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("r.0.0.linear");
        let mut file = LinearV2File::open(&path, Vector2::new(0, 0)).unwrap();
        file.stage_chunk(0, fake_nbt(1));
        file.stage_chunk(CHUNK_COUNT - 1, fake_nbt(2));
        file.flush(true).unwrap();

        // Damage the stored hash of bucket 0, which holds chunk 0. It follows the empty
        // features dictionary (1 byte) and the bucket's size and compression level (5 bytes).
        let hash_offset = LinearV2Superblock::SIZE + ChunkBitmap::SIZE + 1 + 5;
        let mut bytes = std::fs::read(&path).unwrap();
        let hash_bytes = &mut bytes[hash_offset..hash_offset + 8];
        let stored_hash: [u8; 8] = hash_bytes.try_into().unwrap();
        hash_bytes.copy_from_slice(&0u64.to_be_bytes());
        std::fs::write(&path, &bytes).unwrap();

        let mut file = LinearV2File::open(&path, Vector2::new(0, 0)).unwrap();
        assert!(file.read_chunk(0).unwrap().is_none());
        assert!(!file.has_staged());
        file.flush(true).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), bytes);

        file.stage_chunk(CHUNK_COUNT - 1, fake_nbt(3));
        file.flush(true).unwrap();
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[hash_offset..hash_offset + 8].copy_from_slice(&stored_hash);
        std::fs::write(&path, &bytes).unwrap();

        let file = LinearV2File::open(&path, Vector2::new(0, 0)).unwrap();
        assert_eq!(file.read_chunk(0).unwrap().unwrap(), fake_nbt(1));
        assert_eq!(
            file.read_chunk(CHUNK_COUNT - 1).unwrap().unwrap(),
            fake_nbt(3)
        );
    }

    #[test]
    fn sync_flush_syncs_an_unsynced_write() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("r.0.0.linear");
        let mut file = LinearV2File::open(&path, Vector2::new(0, 0)).unwrap();
        file.stage_chunk(0, fake_nbt(1));
        file.flush(false).unwrap();

        // The sync opens the written file, so it fails once the file is gone.
        std::fs::remove_file(&path).unwrap();
        assert!(file.flush(true).is_err());
    }

    #[test]
    fn bitmap_round_trip() {
        let mut bm = ChunkBitmap::new();
        bm.set(0, true);
        bm.set(7, true);
        bm.set(1023, true);
        assert!(bm.get(0));
        assert!(!bm.get(1));
        assert!(bm.get(7));
        assert!(!bm.get(8));
        assert!(bm.get(1023));
        assert!(!bm.get(1022));
    }

    #[test]
    fn nbt_features_empty_round_trip() {
        let f = NbtFeatures::empty();
        let encoded = f.to_bytes();
        assert_eq!(encoded, vec![0u8]); // just the end marker

        let mut buf: Bytes = encoded.into();
        let decoded = NbtFeatures::from_bytes(&mut buf).unwrap();
        assert!(decoded.0.is_empty());
    }

    #[test]
    fn nbt_features_round_trip() {
        let mut f = NbtFeatures::empty();
        f.0.insert("version".into(), 42);
        let encoded = f.to_bytes();
        let mut buf: Bytes = encoded.into();
        let decoded = NbtFeatures::from_bytes(&mut buf).unwrap();
        assert_eq!(decoded.0.get("version"), Some(&42u32));
    }

    fn chunk_bucket_index(i: usize, gs: u8) -> usize {
        let cx = i % 32;
        let cz = i / 32;
        let stride = 32 / gs as usize;
        (cz / stride) * gs as usize + (cx / stride)
    }

    fn chunk_local_index(i: usize, gs: u8) -> usize {
        let cx = i % 32;
        let cz = i / 32;
        let stride = 32 / gs as usize;
        (cz % stride) * stride + (cx % stride)
    }

    fn global_chunk_index(bucket: usize, local: usize, gs: u8) -> usize {
        let stride = 32 / gs as usize;
        let brow = bucket / gs as usize;
        let bcol = bucket % gs as usize;
        let lrow = local / stride;
        let lcol = local % stride;
        let cz = brow * stride + lrow;
        let cx = bcol * stride + lcol;
        cz * 32 + cx
    }

    #[test]
    fn index_round_trip_grid2() {
        for i in 0..CHUNK_COUNT {
            let b = chunk_bucket_index(i, 2);
            let l = chunk_local_index(i, 2);
            let recovered = global_chunk_index(b, l, 2);
            assert_eq!(i, recovered, "chunk index {i} round-trip failed");
        }
    }

    #[test]
    fn index_round_trip_grid4() {
        for i in 0..CHUNK_COUNT {
            let b = chunk_bucket_index(i, 4);
            let l = chunk_local_index(i, 4);
            let recovered = global_chunk_index(b, l, 4);
            assert_eq!(i, recovered, "chunk index {i} round-trip failed");
        }
    }

    #[test]
    fn index_round_trip_grid1() {
        for i in 0..CHUNK_COUNT {
            let b = chunk_bucket_index(i, 1);
            let l = chunk_local_index(i, 1);
            let recovered = global_chunk_index(b, l, 1);
            assert_eq!(i, recovered, "chunk index {i} round-trip failed");
        }
    }

    #[test]
    fn superblock_round_trip() {
        let sb = LinearV2Superblock {
            newest_timestamp: 0xDEADBEEFCAFEBABE,
            grid_size: 4,
            region_x: -7,
            region_z: 13,
        };
        let encoded = sb.to_bytes();
        let decoded = LinearV2Superblock::from_bytes(&encoded).unwrap();
        assert_eq!(decoded.newest_timestamp, sb.newest_timestamp);
        assert_eq!(decoded.grid_size, 4);
        assert_eq!(decoded.region_x, -7);
        assert_eq!(decoded.region_z, 13);
    }

    #[test]
    fn superblock_rejects_bad_signature() {
        let mut bytes = LinearV2Superblock {
            newest_timestamp: 0,
            grid_size: 2,
            region_x: 0,
            region_z: 0,
        }
        .to_bytes();
        bytes[0] ^= 0xFF; // corrupt the signature
        assert!(LinearV2Superblock::from_bytes(&bytes).is_err());
    }

    #[test]
    fn bucket_entry_present_round_trip() {
        let entry = BucketChunkEntry {
            timestamp: 12345,
            data: Some(fake_nbt(0xAB)),
        };
        let mut buf = Vec::new();
        entry.write_into(&mut buf);
        let mut b: Bytes = buf.into();
        let decoded = BucketChunkEntry::read_from(&mut b).unwrap();
        assert_eq!(decoded.timestamp, 12345);
        assert_eq!(decoded.data.unwrap().as_ref(), &[0xABu8; 64][..]);
    }

    #[test]
    fn bucket_entry_absent_round_trip() {
        let entry = BucketChunkEntry {
            timestamp: 0,
            data: None,
        };
        let mut buf = Vec::new();
        entry.write_into(&mut buf);
        let mut b: Bytes = buf.into();
        let decoded = BucketChunkEntry::read_from(&mut b).unwrap();
        assert!(decoded.data.is_none());
    }
}
