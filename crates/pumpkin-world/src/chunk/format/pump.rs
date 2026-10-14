use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::chunk::io::{RegionFile, sync_file, write_file_atomically};
use crate::chunk::{ChunkReadingError, ChunkWritingError};
use bytes::Bytes;
use pumpkin_util::math::vector2::Vector2;
use ruzstd::decoding::StreamingDecoder;
use ruzstd::encoding::{CompressionLevel, compress_to_vec};
use serde::{Deserialize, Serialize};

pub struct PumpFile {
    path: PathBuf,
    pub data: PumpData,
    staged: bool,
    /// The file was written without a sync since the last sync.
    unsynced: bool,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PumpData {
    pub x: i32,
    pub z: i32,
    pub chunks: BTreeMap<String, Bytes>,
}

impl PumpFile {
    fn parse(r: Bytes) -> Result<PumpData, ChunkReadingError> {
        let mut cursor = std::io::Cursor::new(r);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let nbt = pumpkin_nbt::Nbt::read_unnamed(&mut reader).map_err(|e| {
            ChunkReadingError::ParsingError(
                crate::chunk::ChunkParsingError::ErrorDeserializingChunk(e.to_string()),
            )
        })?;

        let x = nbt.get_int("x").unwrap_or(0);
        let z = nbt.get_int("z").unwrap_or(0);
        let mut chunks = BTreeMap::new();
        if let Some(chunks_tag) = nbt.get_compound("chunks") {
            for (k, v) in &chunks_tag.child_tags {
                if let pumpkin_nbt::tag::NbtTag::ByteArray(arr) = v {
                    let u8_vec: Vec<u8> = arr.iter().map(|&b| b as u8).collect();
                    chunks.insert(k.to_string(), u8_vec.into());
                }
            }
        }

        Ok(PumpData { x, z, chunks })
    }
}

impl RegionFile for PumpFile {
    type Config = ();
    type Encoded = Bytes;
    type Raw = Bytes;

    // Every flush rewrites the whole file, so saves are collected for a while first.
    const FLUSH_DELAY: Duration = Duration::from_secs(30);
    const CACHE_CAPACITY: usize = 32;

    fn file_name(region: Vector2<i32>) -> String {
        format!("r.{}.{}.pump", region.x, region.y)
    }

    fn open(path: &Path, region: Vector2<i32>) -> Result<Self, ChunkReadingError> {
        let data = match std::fs::read(path) {
            Ok(bytes) if !bytes.is_empty() => Self::parse(bytes.into())?,
            Ok(_) => PumpData::default(),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => PumpData::default(),
            Err(err) => return Err(ChunkReadingError::IoError(err)),
        };
        Ok(Self {
            path: path.to_path_buf(),
            data: PumpData {
                x: region.x,
                z: region.y,
                ..data
            },
            staged: false,
            unsynced: false,
        })
    }

    fn read_chunk(&self, index: usize) -> Result<Option<Bytes>, ChunkReadingError> {
        Ok(self.data.chunks.get(&index.to_string()).cloned())
    }

    fn decode_chunk(raw: Bytes) -> Result<Bytes, ChunkReadingError> {
        let mut decoder = StreamingDecoder::new(&raw[..])
            .map_err(|e| ChunkReadingError::IoError(std::io::Error::other(e.to_string())))?;
        let mut decompressed = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut decompressed)
            .map_err(ChunkReadingError::IoError)?;
        Ok(decompressed.into())
    }

    fn encode_chunk(nbt: Bytes, _config: &()) -> Result<Bytes, ChunkWritingError> {
        Ok(compress_to_vec(&nbt[..], CompressionLevel::Fastest).into())
    }

    fn stage_chunk(&mut self, index: usize, chunk: Bytes) {
        self.data.chunks.insert(index.to_string(), chunk);
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
        let mut root = pumpkin_nbt::compound::NbtCompound::new();
        root.put_int("x", self.data.x);
        root.put_int("z", self.data.z);
        let mut chunks_comp = pumpkin_nbt::compound::NbtCompound::new();
        for (k, v) in &self.data.chunks {
            let i8_vec: Vec<i8> = v.iter().map(|&b| b as i8).collect();
            chunks_comp.put(k, pumpkin_nbt::tag::NbtTag::ByteArray(i8_vec.into()));
        }
        root.put_compound("chunks", chunks_comp);
        let bytes = pumpkin_nbt::Nbt::from(root).write_unnamed();
        write_file_atomically(&self.path, &bytes, sync)?;
        self.staged = false;
        self.unsynced = !sync;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn pump_file_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("r.0.0.pump");

        let mut pump_file = PumpFile::open(&file_path, Vector2::new(0, 0)).unwrap();
        let nbt = Bytes::from_static(&[1, 2, 3]);
        pump_file.stage_chunk(0, PumpFile::encode_chunk(nbt.clone(), &()).unwrap());
        pump_file.flush(false).unwrap();

        let read_file = PumpFile::open(&file_path, Vector2::new(0, 0)).unwrap();
        assert_eq!(read_file.data.chunks.len(), 1);
        let raw = read_file.read_chunk(0).unwrap().unwrap();
        assert_eq!(PumpFile::decode_chunk(raw).unwrap(), nbt);
    }

    #[test]
    fn sync_flush_syncs_an_unsynced_write() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("r.0.0.pump");
        let mut pump_file = PumpFile::open(&file_path, Vector2::new(0, 0)).unwrap();
        let nbt = Bytes::from_static(&[1, 2, 3]);
        pump_file.stage_chunk(0, PumpFile::encode_chunk(nbt, &()).unwrap());
        pump_file.flush(false).unwrap();

        // The sync opens the written file, so it fails once the file is gone.
        std::fs::remove_file(&file_path).unwrap();
        assert!(pump_file.flush(true).is_err());
    }
}
