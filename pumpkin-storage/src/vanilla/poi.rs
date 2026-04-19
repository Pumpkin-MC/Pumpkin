//! Vanilla POI backend — MCA region files under `<world_dir>/poi/r.X.Z.mca`,
//! zlib-compressed NBT chunk payloads.

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use pumpkin_util::math::position::BlockPos;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::StorageError;
use crate::poi::{PoiEntry, PoiStorage};
use crate::vanilla::VanillaStorage;

const SECTOR_SIZE: usize = 4096;
const REGION_SIZE: usize = 32;
const CHUNK_COUNT: usize = REGION_SIZE * REGION_SIZE;
const HEADER_SIZE: usize = SECTOR_SIZE * 2;
const COMPRESSION_ZLIB: u8 = 2;
const DATA_VERSION: i32 = 3955;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PoiSectionData {
    #[serde(default)]
    valid: i8,
    #[serde(default)]
    records: Vec<PoiEntry>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PoiChunkData {
    data_version: i32,
    sections: HashMap<String, PoiSectionData>,
}

#[derive(Debug, Default)]
struct PoiRegion {
    entries: HashMap<(i32, i32, i32), PoiEntry>,
    dirty: bool,
}

impl PoiRegion {
    fn add(&mut self, entry: PoiEntry) {
        let key = (entry.x, entry.y, entry.z);
        self.entries.insert(key, entry);
        self.dirty = true;
    }

    fn remove(&mut self, pos: BlockPos) -> bool {
        if self.entries.remove(&(pos.0.x, pos.0.y, pos.0.z)).is_some() {
            self.dirty = true;
            return true;
        }
        false
    }

    fn entries(&self) -> impl Iterator<Item = &PoiEntry> {
        self.entries.values()
    }

    fn load(path: &Path) -> std::io::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let file_data = std::fs::read(path)?;
        if file_data.len() < HEADER_SIZE {
            return Ok(Self::default());
        }

        let mut region = Self::default();

        for index in 0..CHUNK_COUNT {
            let offset = index * 4;
            let location = u32::from_be_bytes([
                file_data[offset],
                file_data[offset + 1],
                file_data[offset + 2],
                file_data[offset + 3],
            ]);
            let sector_offset = (location >> 8) as usize;
            let sector_count = (location & 0xFF) as usize;
            if sector_offset == 0 || sector_count == 0 {
                continue;
            }
            let byte_offset = sector_offset * SECTOR_SIZE;
            let byte_end = byte_offset + sector_count * SECTOR_SIZE;
            if byte_end > file_data.len() {
                continue;
            }
            let chunk_bytes = &file_data[byte_offset..byte_end];
            if chunk_bytes.len() < 5 {
                continue;
            }
            let length = u32::from_be_bytes([
                chunk_bytes[0],
                chunk_bytes[1],
                chunk_bytes[2],
                chunk_bytes[3],
            ]) as usize;
            let compression = chunk_bytes[4];
            if compression != COMPRESSION_ZLIB || length < 1 || length > chunk_bytes.len() - 4 {
                continue;
            }
            let compressed = &chunk_bytes[5..5 + length - 1];

            match decompress_chunk(compressed) {
                Ok(chunk_data) => {
                    for (_k, section) in chunk_data.sections {
                        for entry in section.records {
                            let key = (entry.x, entry.y, entry.z);
                            region.entries.insert(key, entry);
                        }
                    }
                }
                Err(e) => warn!("Failed to parse POI chunk at index {index}: {e}"),
            }
        }

        region.dirty = false;
        Ok(region)
    }

    fn save(&mut self, path: &Path) -> std::io::Result<()> {
        if !self.dirty {
            return Ok(());
        }
        if self.entries.is_empty() {
            if path.exists() {
                std::fs::remove_file(path)?;
            }
            self.dirty = false;
            return Ok(());
        }

        let mut chunks_with_data: HashSet<(i32, i32)> = HashSet::new();
        for entry in self.entries.values() {
            chunks_with_data.insert((entry.x >> 4, entry.z >> 4));
        }

        let mut chunk_data_map: HashMap<usize, Vec<u8>> = HashMap::new();
        for (chunk_x, chunk_z) in &chunks_with_data {
            if let Some(chunk_data) = self.build_chunk_data(*chunk_x, *chunk_z) {
                let compressed = compress_chunk(&chunk_data)?;
                let index = chunk_index(*chunk_x, *chunk_z);
                chunk_data_map.insert(index, compressed);
            }
        }

        let mut location_table = [0u32; CHUNK_COUNT];
        let mut timestamp_table = [0u32; CHUNK_COUNT];
        let mut sector_data: Vec<Vec<u8>> = Vec::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as u32);
        let mut current_sector: u32 = 2;

        for index in 0..CHUNK_COUNT {
            if let Some(compressed) = chunk_data_map.get(&index) {
                let data_len = compressed.len() + 5;
                let sector_count = data_len.div_ceil(SECTOR_SIZE) as u32;
                let mut padded = Vec::with_capacity(sector_count as usize * SECTOR_SIZE);
                let length = (compressed.len() + 1) as u32;
                padded.extend_from_slice(&length.to_be_bytes());
                padded.push(COMPRESSION_ZLIB);
                padded.extend_from_slice(compressed);
                padded.resize(sector_count as usize * SECTOR_SIZE, 0);
                location_table[index] = (current_sector << 8) | sector_count;
                timestamp_table[index] = timestamp;
                sector_data.push(padded);
                current_sector += sector_count;
            }
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(path)?;
        for loc in &location_table {
            file.write_all(&loc.to_be_bytes())?;
        }
        for ts in &timestamp_table {
            file.write_all(&ts.to_be_bytes())?;
        }
        for data in &sector_data {
            file.write_all(data)?;
        }

        self.dirty = false;
        Ok(())
    }

    fn build_chunk_data(&self, chunk_x: i32, chunk_z: i32) -> Option<PoiChunkData> {
        let mut sections: HashMap<String, PoiSectionData> = HashMap::new();
        for entry in self.entries.values() {
            if entry.x >> 4 != chunk_x || entry.z >> 4 != chunk_z {
                continue;
            }
            let section_key = (entry.y >> 4).to_string();
            let section = sections
                .entry(section_key)
                .or_insert_with(|| PoiSectionData {
                    valid: 1,
                    records: Vec::new(),
                });
            section.records.push(entry.clone());
        }
        if sections.is_empty() {
            None
        } else {
            Some(PoiChunkData {
                data_version: DATA_VERSION,
                sections,
            })
        }
    }
}

const fn chunk_index(chunk_x: i32, chunk_z: i32) -> usize {
    let local_x = chunk_x & 31;
    let local_z = chunk_z & 31;
    ((local_z << 5) | local_x) as usize
}

const fn region_coords(pos: BlockPos) -> (i32, i32) {
    let chunk_x = pos.0.x >> 4;
    let chunk_z = pos.0.z >> 4;
    (chunk_x >> 5, chunk_z >> 5)
}

fn compress_chunk(chunk_data: &PoiChunkData) -> std::io::Result<Vec<u8>> {
    let mut uncompressed = Vec::new();
    pumpkin_nbt::to_bytes_unnamed(chunk_data, &mut uncompressed)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&uncompressed)?;
    encoder.finish()
}

fn decompress_chunk(compressed: &[u8]) -> std::io::Result<PoiChunkData> {
    let mut decoder = ZlibDecoder::new(compressed);
    let mut uncompressed = Vec::new();
    decoder.read_to_end(&mut uncompressed)?;
    pumpkin_nbt::from_bytes_unnamed(Cursor::new(uncompressed))
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
}

#[derive(Debug, Default)]
pub(crate) struct PoiInner {
    regions: HashMap<(i32, i32), PoiRegion>,
}

impl VanillaStorage {
    fn poi_folder(&self) -> PathBuf {
        self.world_dir().join("poi")
    }

    fn poi_region_path(&self, rx: i32, rz: i32) -> PathBuf {
        self.poi_folder().join(format!("r.{rx}.{rz}.mca"))
    }
}

impl PoiInner {
    fn get_or_load_region(&mut self, rx: i32, rz: i32, path: &Path) -> &mut PoiRegion {
        self.regions.entry((rx, rz)).or_insert_with(|| {
            PoiRegion::load(path).unwrap_or_else(|e| {
                if path.exists() {
                    warn!("Failed to load POI region {}: {}", path.display(), e);
                }
                PoiRegion::default()
            })
        })
    }
}

#[async_trait]
impl PoiStorage for VanillaStorage {
    async fn add(&self, pos: BlockPos, poi_type: &str) -> Result<(), StorageError> {
        let (rx, rz) = region_coords(pos);
        let path = self.poi_region_path(rx, rz);
        let mut guard = self.poi_inner.lock().await;
        let region = guard.get_or_load_region(rx, rz, &path);
        region.add(PoiEntry {
            x: pos.0.x,
            y: pos.0.y,
            z: pos.0.z,
            poi_type: poi_type.to_string(),
            free_tickets: 0,
        });
        Ok(())
    }

    async fn remove(&self, pos: BlockPos) -> Result<bool, StorageError> {
        let (rx, rz) = region_coords(pos);
        let path = self.poi_region_path(rx, rz);
        let mut guard = self.poi_inner.lock().await;
        let region = guard.get_or_load_region(rx, rz, &path);
        Ok(region.remove(pos))
    }

    async fn get_in_square(
        &self,
        center: BlockPos,
        radius: i32,
        poi_type: Option<&str>,
    ) -> Result<Vec<BlockPos>, StorageError> {
        let min_x = center.0.x - radius;
        let max_x = center.0.x + radius;
        let min_z = center.0.z - radius;
        let max_z = center.0.z + radius;
        let min_rx = (min_x >> 4) >> 5;
        let max_rx = (max_x >> 4) >> 5;
        let min_rz = (min_z >> 4) >> 5;
        let max_rz = (max_z >> 4) >> 5;

        let mut guard = self.poi_inner.lock().await;
        let mut results = Vec::new();
        for rx in min_rx..=max_rx {
            for rz in min_rz..=max_rz {
                let path = self.poi_region_path(rx, rz);
                let region = guard.get_or_load_region(rx, rz, &path);
                for entry in region.entries() {
                    if let Some(filter) = poi_type
                        && entry.poi_type != filter
                    {
                        continue;
                    }
                    let dx = (entry.x - center.0.x).abs();
                    let dz = (entry.z - center.0.z).abs();
                    if dx <= radius && dz <= radius {
                        results.push(entry.pos());
                    }
                }
            }
        }
        Ok(results)
    }

    async fn save_all(&self) -> Result<(), StorageError> {
        let folder = self.poi_folder();
        std::fs::create_dir_all(&folder).map_err(|e| StorageError::io_at(&folder, e))?;

        let mut guard = self.poi_inner.lock().await;
        for ((rx, rz), region) in guard.regions.iter_mut() {
            if region.dirty {
                let path = folder.join(format!("r.{rx}.{rz}.mca"));
                region.save(&path).map_err(|e| StorageError::io_at(&path, e))?;
            }
        }
        Ok(())
    }
}
