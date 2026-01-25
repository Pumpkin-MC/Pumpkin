use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use serde::{Deserialize, Serialize};

/// POI type identifier for nether portals
pub const POI_TYPE_NETHER_PORTAL: &str = "minecraft:nether_portal";

/// MCA format constants
const SECTOR_SIZE: usize = 4096;
const REGION_SIZE: usize = 32;
const CHUNK_COUNT: usize = REGION_SIZE * REGION_SIZE;
const HEADER_SIZE: usize = SECTOR_SIZE * 2; // Location table + timestamp table

/// Compression type for MCA format
const COMPRESSION_ZLIB: u8 = 2;

// Data version for 1.21
const DATA_VERSION: i32 = 3955;

/// A single Point of Interest entry (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoiEntry {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    #[serde(rename = "type")]
    pub poi_type: String,
    pub free_tickets: i32,
}

impl PoiEntry {
    pub fn new_portal(pos: BlockPos) -> Self {
        Self {
            x: pos.0.x,
            y: pos.0.y,
            z: pos.0.z,
            poi_type: POI_TYPE_NETHER_PORTAL.to_string(),
            free_tickets: 0,
        }
    }

    pub fn pos(&self) -> BlockPos {
        BlockPos(Vector3::new(self.x, self.y, self.z))
    }
}

/// POI section data (serializable) - vanilla format
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PoiSectionData {
    #[serde(default)]
    pub valid: i8,
    #[serde(default)]
    pub records: Vec<PoiEntry>,
}

/// POI chunk data (serializable) - vanilla format
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PoiChunkData {
    pub data_version: i32,
    /// Sections keyed by Y section coordinate (e.g., "-1", "0", "1", "4")
    pub sections: HashMap<String, PoiSectionData>,
}

/// POI data for a single region (32x32 chunks) using MCA format
#[derive(Debug, Default)]
pub struct PoiRegion {
    /// Entries indexed by position
    entries: HashMap<(i32, i32, i32), PoiEntry>,
    /// Track which chunks are dirty
    dirty_chunks: std::collections::HashSet<(i32, i32)>,
    dirty: bool,
}

impl PoiRegion {
    pub fn new() -> Self {
        Self::default()
    }

    fn pos_key(pos: &BlockPos) -> (i32, i32, i32) {
        (pos.0.x, pos.0.y, pos.0.z)
    }

    /// Get chunk index in MCA file (0-1023)
    fn chunk_index(chunk_x: i32, chunk_z: i32) -> usize {
        let local_x = chunk_x & 31;
        let local_z = chunk_z & 31;
        ((local_z << 5) | local_x) as usize
    }

    /// Returns section key as just the Y section coordinate (like vanilla)
    fn section_key(pos: &BlockPos) -> String {
        let section_y = pos.0.y >> 4;
        section_y.to_string()
    }

    pub fn add(&mut self, entry: PoiEntry) {
        let chunk_x = entry.x >> 4;
        let chunk_z = entry.z >> 4;
        self.dirty_chunks.insert((chunk_x, chunk_z));
        let key = (entry.x, entry.y, entry.z);
        self.entries.insert(key, entry);
        self.dirty = true;
    }

    pub fn remove(&mut self, pos: &BlockPos) -> bool {
        let key = Self::pos_key(pos);
        if self.entries.remove(&key).is_some() {
            let chunk_x = pos.0.x >> 4;
            let chunk_z = pos.0.z >> 4;
            self.dirty_chunks.insert((chunk_x, chunk_z));
            self.dirty = true;
            return true;
        }
        false
    }

    pub fn get_all(&self) -> Vec<&PoiEntry> {
        self.entries.values().collect()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
        self.dirty_chunks.clear();
    }

    /// Group entries by chunk, then create chunk NBT data
    fn get_chunk_data(&self, chunk_x: i32, chunk_z: i32) -> Option<PoiChunkData> {
        let mut sections: HashMap<String, PoiSectionData> = HashMap::new();

        for entry in self.entries.values() {
            let entry_chunk_x = entry.x >> 4;
            let entry_chunk_z = entry.z >> 4;

            if entry_chunk_x != chunk_x || entry_chunk_z != chunk_z {
                continue;
            }

            let section_key = Self::section_key(&entry.pos());
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

    /// Compress chunk data to bytes
    fn compress_chunk_data(chunk_data: &PoiChunkData) -> std::io::Result<Vec<u8>> {
        let mut uncompressed = Vec::new();
        pumpkin_nbt::to_bytes_unnamed(chunk_data, &mut uncompressed)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&uncompressed)?;
        encoder.finish()
    }

    /// Decompress chunk data from bytes
    fn decompress_chunk_data(compressed: &[u8]) -> std::io::Result<PoiChunkData> {
        let mut decoder = ZlibDecoder::new(compressed);
        let mut uncompressed = Vec::new();
        decoder.read_to_end(&mut uncompressed)?;

        pumpkin_nbt::from_bytes_unnamed(Cursor::new(uncompressed))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }

    pub fn save(&mut self, path: &Path) -> std::io::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        if self.entries.is_empty() {
            // Don't save empty regions, delete the file if it exists
            if path.exists() {
                std::fs::remove_file(path)?;
            }
            self.dirty = false;
            self.dirty_chunks.clear();
            return Ok(());
        }

        // Build all chunk data
        let mut chunk_data_map: HashMap<usize, Vec<u8>> = HashMap::new();

        // Collect all unique chunks that have entries
        let mut chunks_with_data: std::collections::HashSet<(i32, i32)> =
            std::collections::HashSet::new();
        for entry in self.entries.values() {
            chunks_with_data.insert((entry.x >> 4, entry.z >> 4));
        }

        for (chunk_x, chunk_z) in &chunks_with_data {
            if let Some(chunk_data) = self.get_chunk_data(*chunk_x, *chunk_z) {
                let compressed = Self::compress_chunk_data(&chunk_data)?;
                let index = Self::chunk_index(*chunk_x, *chunk_z);
                chunk_data_map.insert(index, compressed);
            }
        }

        // Build MCA file
        let mut location_table = [0u32; CHUNK_COUNT];
        let mut timestamp_table = [0u32; CHUNK_COUNT];
        let mut sector_data: Vec<Vec<u8>> = Vec::new();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as u32)
            .unwrap_or(0);

        // Start after header (2 sectors)
        let mut current_sector: u32 = 2;

        for index in 0..CHUNK_COUNT {
            if let Some(compressed) = chunk_data_map.get(&index) {
                // Calculate sector count needed
                let data_len = compressed.len() + 5; // 4 bytes length + 1 byte compression + data
                let sector_count = data_len.div_ceil(SECTOR_SIZE) as u32;

                // Build padded sector data
                let mut padded = Vec::with_capacity(sector_count as usize * SECTOR_SIZE);
                let length = (compressed.len() + 1) as u32; // +1 for compression byte
                padded.extend_from_slice(&length.to_be_bytes());
                padded.push(COMPRESSION_ZLIB);
                padded.extend_from_slice(compressed);
                // Pad to sector boundary
                padded.resize(sector_count as usize * SECTOR_SIZE, 0);

                location_table[index] = (current_sector << 8) | sector_count;
                timestamp_table[index] = timestamp;
                sector_data.push(padded);

                current_sector += sector_count;
            }
        }

        // Write file
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::File::create(path)?;

        // Write location table
        for loc in &location_table {
            file.write_all(&loc.to_be_bytes())?;
        }

        // Write timestamp table
        for ts in &timestamp_table {
            file.write_all(&ts.to_be_bytes())?;
        }

        // Write chunk data
        for data in &sector_data {
            file.write_all(data)?;
        }

        self.dirty = false;
        self.dirty_chunks.clear();
        Ok(())
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let file_data = std::fs::read(path)?;
        if file_data.len() < HEADER_SIZE {
            return Ok(Self::new());
        }

        let mut region = Self::new();

        // Parse location table
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

            // Read chunk data
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

            match Self::decompress_chunk_data(compressed) {
                Ok(chunk_data) => {
                    for (_section_key, section) in chunk_data.sections {
                        for entry in section.records {
                            let key = (entry.x, entry.y, entry.z);
                            region.entries.insert(key, entry);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to parse POI chunk at index {}: {}", index, e);
                }
            }
        }

        region.dirty = false;
        Ok(region)
    }
}

/// Region-based POI storage using MCA format
pub struct PoiStorage {
    /// Path to the poi folder
    folder: PathBuf,
    /// Loaded regions, keyed by (region_x, region_z)
    regions: HashMap<(i32, i32), PoiRegion>,
}

impl PoiStorage {
    pub fn new(world_folder: &Path) -> Self {
        Self {
            folder: world_folder.join("poi"),
            regions: HashMap::new(),
        }
    }

    fn region_coords(pos: &BlockPos) -> (i32, i32) {
        let chunk_x = pos.0.x >> 4;
        let chunk_z = pos.0.z >> 4;
        (chunk_x >> 5, chunk_z >> 5)
    }

    fn region_path(&self, rx: i32, rz: i32) -> PathBuf {
        self.folder.join(format!("r.{}.{}.mca", rx, rz))
    }

    fn get_or_load_region(&mut self, rx: i32, rz: i32) -> &mut PoiRegion {
        if !self.regions.contains_key(&(rx, rz)) {
            let path = self.region_path(rx, rz);
            let region = PoiRegion::load(&path).unwrap_or_else(|e| {
                if path.exists() {
                    log::warn!("Failed to load POI region {:?}: {}", path, e);
                }
                PoiRegion::new()
            });
            self.regions.insert((rx, rz), region);
        }
        self.regions.get_mut(&(rx, rz)).unwrap()
    }

    pub fn add(&mut self, pos: BlockPos, poi_type: &str) {
        let (rx, rz) = Self::region_coords(&pos);
        let region = self.get_or_load_region(rx, rz);
        region.add(PoiEntry {
            x: pos.0.x,
            y: pos.0.y,
            z: pos.0.z,
            poi_type: poi_type.to_string(),
            free_tickets: 0,
        });
    }

    pub fn add_portal(&mut self, pos: BlockPos) {
        self.add(pos, POI_TYPE_NETHER_PORTAL);
    }

    pub fn remove(&mut self, pos: &BlockPos) -> bool {
        let (rx, rz) = Self::region_coords(pos);
        let region = self.get_or_load_region(rx, rz);
        region.remove(pos)
    }

    /// Get all POI positions within a square radius (for portal search)
    pub fn get_in_square(
        &mut self,
        center: BlockPos,
        radius: i32,
        poi_type: Option<&str>,
    ) -> Vec<BlockPos> {
        let min_x = center.0.x - radius;
        let max_x = center.0.x + radius;
        let min_z = center.0.z - radius;
        let max_z = center.0.z + radius;

        // Calculate which regions we need to check
        let min_rx = (min_x >> 4) >> 5;
        let max_rx = (max_x >> 4) >> 5;
        let min_rz = (min_z >> 4) >> 5;
        let max_rz = (max_z >> 4) >> 5;

        let mut results = Vec::new();

        for rx in min_rx..=max_rx {
            for rz in min_rz..=max_rz {
                let region = self.get_or_load_region(rx, rz);
                for entry in region.get_all() {
                    if let Some(filter_type) = poi_type
                        && entry.poi_type != filter_type
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

        results
    }

    pub fn save_all(&mut self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.folder)?;

        let mut saved = 0;
        for ((rx, rz), region) in &mut self.regions {
            if region.is_dirty() {
                let path = self.folder.join(format!("r.{}.{}.mca", rx, rz));
                region.save(&path)?;
                saved += 1;
            }
        }

        if saved > 0 {
            log::info!("Saved {} POI region(s)", saved);
        }
        Ok(())
    }

    /// Get count of loaded regions
    #[must_use]
    pub fn loaded_region_count(&self) -> usize {
        self.regions.len()
    }

    /// Get total POI count across all loaded regions
    #[must_use]
    pub fn total_poi_count(&self) -> usize {
        self.regions.values().map(|r| r.get_all().len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poi_entry() {
        let entry = PoiEntry::new_portal(BlockPos(Vector3::new(100, 64, 200)));
        assert_eq!(entry.x, 100);
        assert_eq!(entry.y, 64);
        assert_eq!(entry.z, 200);
        assert_eq!(entry.poi_type, POI_TYPE_NETHER_PORTAL);
    }

    #[test]
    fn test_poi_region() {
        let mut region = PoiRegion::new();
        region.add(PoiEntry::new_portal(BlockPos(Vector3::new(100, 64, 200))));
        region.add(PoiEntry::new_portal(BlockPos(Vector3::new(101, 64, 200))));

        assert_eq!(region.get_all().len(), 2);
        assert!(region.is_dirty());

        region.remove(&BlockPos(Vector3::new(100, 64, 200)));
        assert_eq!(region.get_all().len(), 1);
    }

    #[test]
    fn test_poi_storage_mca() {
        let dir = std::env::temp_dir().join("pumpkin_poi_mca_test");
        let _ = std::fs::remove_dir_all(&dir);

        let mut storage = PoiStorage::new(&dir);

        storage.add_portal(BlockPos(Vector3::new(100, 64, 100)));
        storage.add_portal(BlockPos(Vector3::new(110, 64, 100)));
        storage.add_portal(BlockPos(Vector3::new(1000, 64, 1000))); // Different region

        let results = storage.get_in_square(
            BlockPos(Vector3::new(105, 64, 100)),
            16,
            Some(POI_TYPE_NETHER_PORTAL),
        );
        assert_eq!(results.len(), 2);

        storage.save_all().unwrap();

        // Verify .mca file was created
        let mca_path = dir.join("poi").join("r.0.0.mca");
        assert!(mca_path.exists());

        // Reload and verify
        let mut storage2 = PoiStorage::new(&dir);
        let results2 = storage2.get_in_square(
            BlockPos(Vector3::new(105, 64, 100)),
            16,
            Some(POI_TYPE_NETHER_PORTAL),
        );
        assert_eq!(results2.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
