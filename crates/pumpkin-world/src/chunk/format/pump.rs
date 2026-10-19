use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

use bytes::{Buf, BufMut, Bytes};
use pumpkin_util::math::vector2::Vector2;
use rustc_hash::FxHashMap;
use ruzstd::{
    decoding::StreamingDecoder,
    encoding::{CompressionLevel, compress_to_vec},
};
use tracing::warn;
use xxhash_rust::xxh64::xxh64;

use crate::chunk::{
    ChunkReadingError, ChunkSerializingError, ChunkWritingError,
    format::anvil::{CHUNK_COUNT, SectorBitmap, SingleChunkDataSerializer},
    io::{RegionFile, read_exact_at, write_all_at},
};

/// Magic signature and format version (`"PUMP"` + version 1).
const SIGNATURE: [u8; 8] = *b"PUMP\x00\x00\x00\x01";

/// Magic prefix for a native `.pump` binary chunk payload.
pub const PUMP_CHUNK_MAGIC: [u8; 4] = *b"PCHK";

/// Per-chunk frame header: `u32` uncompressed byte length + `u64` `XXH64` checksum.
const CHUNK_FRAME_HEADER_BYTES: usize = 12;

/// Upper bound on a single decompressed chunk payload (64 MiB) to guard against corrupt headers.
const MAX_DECOMPRESSED_CHUNK_BYTES: usize = 64 * 1024 * 1024;

/// Sector alignment in bytes. 256-byte sectors keep 32-bit sector offsets able to address up to
/// 1 TiB per region file while wasting 16x less tail padding than Anvil's 4 KiB sectors.
const SECTOR_BYTES: usize = 256;

/// Sector 0 holds the 256-byte superblock; sectors 1..33 hold 1024 `(offset: u32, length: u32)`
/// entries (8192 bytes).
const HEADER_SECTORS: u32 = 33;
const HEADER_BYTES: usize = HEADER_SECTORS as usize * SECTOR_BYTES;

const fn sector_count(byte_len: u32) -> u32 {
    byte_len.div_ceil(SECTOR_BYTES as u32)
}

/// A `.pump` region file.
///
/// Only the 8 KiB offset/length table and sector bitmap live in memory. Staged chunks are
/// zstd-compressed in parallel on Rayon and written into free 256-byte sectors before the header
/// is updated at offset 0, so a crash at any point leaves every chunk readable in either its old
/// or its new version.
pub struct PumpFile {
    path: PathBuf,
    region: Vector2<i32>,
    file: Option<File>,
    file_len: u64,
    offsets: Box<[u32]>,
    lengths: Box<[u32]>,
    used_sectors: SectorBitmap,
    staged: FxHashMap<usize, Bytes>,
    unsynced: bool,
}

impl PumpFile {
    fn flush_with_sync(
        &mut self,
        sync: bool,
        sync_file: impl Fn(&File) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        if self.staged.is_empty() {
            if sync
                && self.unsynced
                && let Some(file) = &self.file
            {
                sync_file(file)?;
                self.unsynced = false;
            }
            return Ok(());
        }

        let mut used_sectors = self.used_sectors.clone();
        let mut free_runs = self.used_sectors.free_runs();
        let mut offsets = self.offsets.clone();
        let mut lengths = self.lengths.clone();
        let mut freed = Vec::new();
        let mut writes = Vec::with_capacity(self.staged.len());

        let mut staged: Vec<_> = self.staged.iter().collect();
        staged.sort_unstable_by_key(|(index, _)| **index);
        for (&index, chunk) in staged {
            let len = u32::try_from(chunk.len()).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "chunk too large")
            })?;
            if len == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "encoded chunk cannot be empty",
                ));
            }
            let count = sector_count(len);
            let padded_len = count as usize * SECTOR_BYTES;
            let mut record = Vec::with_capacity(padded_len);
            record.extend_from_slice(chunk);
            record.resize(padded_len, 0);

            let offset = free_runs.allocate(count);
            used_sectors.set(offset, count, true);
            if offsets[index] != 0 {
                freed.push((offsets[index], sector_count(lengths[index])));
            }
            offsets[index] = offset;
            lengths[index] = len;
            writes.push((offset, record));
        }

        writes.sort_unstable_by_key(|(offset, _)| *offset);
        let file = self.file()?;
        let mut file_end = 0;
        let mut run_start = 0;
        let mut run: Vec<u8> = Vec::new();
        for (offset, record) in writes {
            let position = u64::from(offset) * SECTOR_BYTES as u64;
            if !run.is_empty() && run_start + run.len() as u64 != position {
                write_all_at(file, &run, run_start)?;
                run.clear();
            }
            if run.is_empty() {
                run_start = position;
            }
            run.extend_from_slice(&record);
            file_end = position + record.len() as u64;
        }
        if !run.is_empty() {
            write_all_at(file, &run, run_start)?;
        }

        let old_offsets = std::mem::replace(&mut self.offsets, offsets);
        let old_lengths = std::mem::replace(&mut self.lengths, lengths);
        let header = self.header_bytes();
        let file = self.file()?;
        if let Err(err) = write_all_at(file, &header, 0) {
            self.offsets = old_offsets;
            self.lengths = old_lengths;
            return Err(err);
        }
        for (offset, count) in freed {
            used_sectors.set(offset, count, false);
        }
        self.file_len = self.file_len.max(file_end).max(HEADER_BYTES as u64);
        self.used_sectors = used_sectors;
        self.unsynced = true;
        if sync && let Some(file) = &self.file {
            sync_file(file)?;
            self.unsynced = false;
        }
        self.truncate_unused_tail();
        self.staged.clear();
        Ok(())
    }

    fn truncate_unused_tail(&mut self) {
        let used_len = u64::from(self.used_sectors.end()) * SECTOR_BYTES as u64;
        if used_len >= self.file_len {
            return;
        }
        let Some(file) = &self.file else {
            return;
        };
        match file.set_len(used_len) {
            Ok(()) => self.file_len = used_len,
            Err(err) => warn!("Failed to shrink {}: {err}", self.path.display()),
        }
    }

    fn header_bytes(&self) -> Vec<u8> {
        let mut header = Vec::with_capacity(HEADER_BYTES);
        header.put_slice(&SIGNATURE);
        header.put_i32(self.region.x);
        header.put_i32(self.region.y);
        header.resize(SECTOR_BYTES, 0);
        for index in 0..CHUNK_COUNT {
            header.put_u32(self.offsets[index]);
            header.put_u32(self.lengths[index]);
        }
        header
    }

    fn file(&mut self) -> std::io::Result<&File> {
        if self.file.is_none() {
            if let Some(parent) = self.path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&self.path)?;
            self.file_len = file.metadata()?.len();
            self.file = Some(file);
        }
        self.file
            .as_ref()
            .ok_or_else(|| std::io::Error::other("region file is not open"))
    }
}

impl RegionFile for PumpFile {
    type Config = ();
    type Encoded = Bytes;
    type Raw = Bytes;

    // Writes only touch the changed chunks, so there is nothing to gain from waiting.
    const FLUSH_DELAY: Duration = Duration::ZERO;
    const CACHE_CAPACITY: usize = 64;

    fn file_name(region: Vector2<i32>) -> String {
        format!("r.{}.{}.pump", region.x, region.y)
    }

    fn open(path: &Path, region: Vector2<i32>) -> Result<Self, ChunkReadingError> {
        let mut this = Self {
            path: path.to_path_buf(),
            region,
            file: None,
            file_len: 0,
            offsets: vec![0; CHUNK_COUNT].into_boxed_slice(),
            lengths: vec![0; CHUNK_COUNT].into_boxed_slice(),
            used_sectors: SectorBitmap::default(),
            staged: FxHashMap::default(),
            unsynced: false,
        };
        this.used_sectors.set(0, HEADER_SECTORS, true);

        let file = match OpenOptions::new().read(true).write(true).open(path) {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(this),
            Err(err) => return Err(ChunkReadingError::IoError(err)),
        };
        let file_len = file.metadata().map_err(ChunkReadingError::IoError)?.len();
        if file_len == 0 {
            this.file = Some(file);
            return Ok(this);
        }
        if file_len < HEADER_BYTES as u64 {
            return Err(ChunkReadingError::InvalidHeader);
        }

        let mut header = vec![0; HEADER_BYTES];
        read_exact_at(&file, &mut header, 0).map_err(ChunkReadingError::IoError)?;
        if header[..SIGNATURE.len()] != SIGNATURE {
            return Err(ChunkReadingError::InvalidHeader);
        }

        let mut table = &header[SECTOR_BYTES..];
        for index in 0..CHUNK_COUNT {
            let offset = table.get_u32();
            let len = table.get_u32();
            if offset == 0 && len == 0 {
                continue;
            }
            let count = sector_count(len);
            let end_byte = u64::from(offset) * SECTOR_BYTES as u64 + u64::from(len);
            if offset < HEADER_SECTORS || len == 0 || end_byte > file_len {
                warn!(
                    "Pump region {} has an invalid entry at index {index}: offset {offset}, len {len}",
                    path.display()
                );
                continue;
            }
            this.offsets[index] = offset;
            this.lengths[index] = len;
            this.used_sectors.set(offset, count, true);
        }

        this.file = Some(file);
        this.file_len = file_len;
        Ok(this)
    }

    fn read_chunk(&self, index: usize) -> Result<Option<Bytes>, ChunkReadingError> {
        if let Some(staged) = self.staged.get(&index) {
            return Ok(Some(staged.clone()));
        }
        let offset = self.offsets[index];
        let len = self.lengths[index];
        let Some(file) = &self.file else {
            return Ok(None);
        };
        if offset == 0 || len == 0 {
            return Ok(None);
        }

        let start = u64::from(offset) * SECTOR_BYTES as u64;
        let mut buf = vec![0; len as usize];
        read_exact_at(file, &mut buf, start).map_err(ChunkReadingError::IoError)?;
        Ok(Some(buf.into()))
    }

    fn decode_chunk(raw: Bytes) -> Result<Bytes, ChunkReadingError> {
        if raw.len() < CHUNK_FRAME_HEADER_BYTES {
            return Err(ChunkReadingError::IoError(std::io::Error::from(
                std::io::ErrorKind::UnexpectedEof,
            )));
        }
        let mut header = &raw[..CHUNK_FRAME_HEADER_BYTES];
        let uncompressed_len = header.get_u32() as usize;
        let expected_checksum = header.get_u64();
        if uncompressed_len > MAX_DECOMPRESSED_CHUNK_BYTES {
            return Err(ChunkReadingError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("pump chunk uncompressed length {uncompressed_len} exceeds limit"),
            )));
        }

        let mut decoder = StreamingDecoder::new(&raw[CHUNK_FRAME_HEADER_BYTES..])
            .map_err(|e| ChunkReadingError::IoError(std::io::Error::other(e.to_string())))?;
        let mut decompressed = Vec::with_capacity(uncompressed_len);
        decoder
            .read_to_end(&mut decompressed)
            .map_err(ChunkReadingError::IoError)?;
        if decompressed.len() != uncompressed_len || xxh64(&decompressed, 0) != expected_checksum {
            return Err(ChunkReadingError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "pump chunk checksum or length mismatch",
            )));
        }
        Ok(decompressed.into())
    }

    fn encode_chunk(nbt: Bytes, _config: &()) -> Result<Bytes, ChunkWritingError> {
        let uncompressed_len = u32::try_from(nbt.len()).map_err(|_| {
            ChunkWritingError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "chunk payload too large",
            ))
        })?;
        let checksum = xxh64(&nbt, 0);
        let compressed = compress_to_vec(&nbt[..], CompressionLevel::Fastest);
        let mut out = Vec::with_capacity(CHUNK_FRAME_HEADER_BYTES + compressed.len());
        out.put_u32(uncompressed_len);
        out.put_u64(checksum);
        out.extend_from_slice(&compressed);
        Ok(out.into())
    }

    fn serialize_chunk<D: SingleChunkDataSerializer>(
        chunk: &D,
    ) -> Result<Bytes, ChunkSerializingError> {
        chunk.to_pump_bytes()
    }

    fn stage_chunk(&mut self, index: usize, chunk: Bytes) {
        self.staged.insert(index, chunk);
    }

    fn has_staged(&self) -> bool {
        !self.staged.is_empty()
    }

    fn flush(&mut self, sync: bool) -> std::io::Result<()> {
        self.flush_with_sync(sync, File::sync_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn noise(seed: u64, len: usize) -> Bytes {
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state as u8
            })
            .collect()
    }

    fn assert_no_overlap(region: &PumpFile) {
        let mut taken = vec![false; region.used_sectors.end() as usize];
        for index in 0..CHUNK_COUNT {
            let offset = region.offsets[index];
            let len = region.lengths[index];
            if offset == 0 {
                continue;
            }
            for sector in offset..offset + sector_count(len) {
                assert!(sector >= HEADER_SECTORS, "chunk inside the header");
                assert!(!taken[sector as usize], "sector {sector} used twice");
                taken[sector as usize] = true;
            }
        }
    }

    #[test]
    fn pump_file_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("r.0.0.pump");

        let mut pump_file = PumpFile::open(&file_path, Vector2::new(0, 0)).unwrap();
        let nbt = Bytes::from_static(&[1, 2, 3]);
        pump_file.stage_chunk(0, PumpFile::encode_chunk(nbt.clone(), &()).unwrap());
        pump_file.flush(false).unwrap();

        let read_file = PumpFile::open(&file_path, Vector2::new(0, 0)).unwrap();
        let raw = read_file.read_chunk(0).unwrap().unwrap();
        assert_eq!(PumpFile::decode_chunk(raw).unwrap(), nbt);
        assert!(read_file.read_chunk(1).unwrap().is_none());
    }

    #[test]
    fn corrupt_chunk_payload_fails_checksum() {
        let payload = Bytes::from(vec![42u8; 256]);
        let mut encoded = PumpFile::encode_chunk(payload, &()).unwrap().to_vec();
        encoded[4] ^= 0xFF;
        assert!(PumpFile::decode_chunk(encoded.into()).is_err());
    }

    #[test]
    fn a_chunk_pointing_into_the_header_is_skipped() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("r.0.0.pump");

        let mut header = Vec::with_capacity(HEADER_BYTES + SECTOR_BYTES);
        header.put_slice(&SIGNATURE);
        header.resize(SECTOR_BYTES, 0);
        header.put_u32(HEADER_SECTORS - 1);
        header.put_u32(16);
        header.resize(HEADER_BYTES + SECTOR_BYTES, 0);
        std::fs::write(&file_path, header).unwrap();

        let region = PumpFile::open(&file_path, Vector2::new(0, 0)).unwrap();
        assert!(region.read_chunk(0).unwrap().is_none());
    }

    #[test]
    fn sync_failure_keeps_live_sectors_allocated_for_retry() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("r.0.0.pump");
        let mut region = PumpFile::open(&path, Vector2::new(0, 0)).unwrap();
        region.stage_chunk(
            0,
            PumpFile::encode_chunk(Bytes::from(vec![1; 64]), &()).unwrap(),
        );
        region.flush(false).unwrap();
        let expected = Bytes::from(vec![2; 64]);
        region.stage_chunk(0, PumpFile::encode_chunk(expected.clone(), &()).unwrap());

        assert!(
            region
                .flush_with_sync(true, |_| Err(std::io::Error::other("sync failed")))
                .is_err()
        );
        assert!(region.has_staged());
        region.flush(false).unwrap();
        region.stage_chunk(
            1,
            PumpFile::encode_chunk(Bytes::from(vec![3; 64]), &()).unwrap(),
        );
        region.flush(false).unwrap();

        let reopened = PumpFile::open(&path, Vector2::new(0, 0)).unwrap();
        assert_eq!(
            PumpFile::decode_chunk(reopened.read_chunk(0).unwrap().unwrap()).unwrap(),
            expected
        );
        assert_no_overlap(&reopened);
    }

    #[test]
    fn rewritten_chunks_reuse_sectors_and_survive_reopening() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("r.-1.2.pump");
        let mut expected = vec![None; CHUNK_COUNT];
        let mut region = PumpFile::open(&path, Vector2::new(-1, 2)).unwrap();

        for round in 0..6u64 {
            for index in (0..CHUNK_COUNT).step_by(37) {
                let len = 100 + ((index as u64 * 7919 + round * 104_729) % 4_000) as usize;
                let nbt = noise(index as u64 ^ (round << 32), len);
                region.stage_chunk(index, PumpFile::encode_chunk(nbt.clone(), &()).unwrap());
                expected[index] = Some(nbt);
            }
            region.flush(false).unwrap();
            assert_no_overlap(&region);
        }
        let file_len = std::fs::metadata(&path).unwrap().len();

        let reopened = PumpFile::open(&path, Vector2::new(-1, 2)).unwrap();
        for (index, nbt) in expected.iter().enumerate() {
            let read = reopened
                .read_chunk(index)
                .unwrap()
                .map(|raw| PumpFile::decode_chunk(raw).unwrap());
            assert_eq!(read.as_ref(), nbt.as_ref(), "chunk {index}");
        }
        let live: u64 = expected.iter().flatten().map(|nbt| nbt.len() as u64).sum();
        assert!(
            file_len < live * 3,
            "file {file_len} bytes for {live} live bytes"
        );
    }
}
