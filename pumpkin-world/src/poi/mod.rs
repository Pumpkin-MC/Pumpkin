use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use serde::{Deserialize, Serialize};

/// POI type identifier for nether portals
pub const POI_TYPE_NETHER_PORTAL: &str = "minecraft:nether_portal";

/// A single Point of Interest entry (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoiEntry {
    pub x: i32,
    pub y: i32,
    pub z: i32,
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

/// POI section data (serializable) - keyed by Y section coordinate
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PoiSectionData {
    #[serde(default)]
    pub valid: bool,
    #[serde(default)]
    pub records: Vec<PoiEntry>,
}

/// POI chunk data (serializable) - sections keyed by Y coordinate like vanilla
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PoiChunkData {
    pub data_version: i32,
    /// Sections keyed by Y section coordinate (e.g., "-1", "0", "1", "4")
    pub sections: HashMap<String, PoiSectionData>,
}

/// POI region data (serializable) - chunks keyed by local chunk coords
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PoiRegionData {
    /// Chunks keyed by "localX,localZ" (0-31 each)
    pub chunks: HashMap<String, PoiChunkData>,
}

/// POI data for a single region (32x32 chunks)
#[derive(Debug, Default)]
pub struct PoiRegion {
    /// Entries indexed by position
    entries: HashMap<(i32, i32, i32), PoiEntry>,
    dirty: bool,
}

impl PoiRegion {
    pub fn new() -> Self {
        Self::default()
    }

    fn pos_key(pos: &BlockPos) -> (i32, i32, i32) {
        (pos.0.x, pos.0.y, pos.0.z)
    }

    /// Returns chunk key as "localX,localZ" (local coords 0-31 within region)
    fn chunk_key(pos: &BlockPos) -> String {
        let chunk_x = pos.0.x >> 4;
        let chunk_z = pos.0.z >> 4;
        format!("{},{}", chunk_x & 31, chunk_z & 31)
    }

    /// Returns section key as just the Y section coordinate (like vanilla)
    fn section_key(pos: &BlockPos) -> String {
        let section_y = pos.0.y >> 4;
        section_y.to_string()
    }

    pub fn add(&mut self, entry: PoiEntry) {
        let key = (entry.x, entry.y, entry.z);
        self.entries.insert(key, entry);
        self.dirty = true;
    }

    pub fn remove(&mut self, pos: &BlockPos) -> bool {
        let key = Self::pos_key(pos);
        if self.entries.remove(&key).is_some() {
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
    }

    fn to_region_data(&self) -> PoiRegionData {
        // Group entries by chunk, then by section Y
        let mut chunks: HashMap<String, PoiChunkData> = HashMap::new();

        for entry in self.entries.values() {
            let pos = entry.pos();
            let chunk_key = Self::chunk_key(&pos);
            let section_key = Self::section_key(&pos);

            let chunk = chunks.entry(chunk_key).or_insert_with(|| PoiChunkData {
                data_version: 3955, // 1.21
                sections: HashMap::new(),
            });

            let section = chunk.sections.entry(section_key).or_insert_with(|| PoiSectionData {
                valid: true,
                records: Vec::new(),
            });
            section.records.push(entry.clone());
        }

        PoiRegionData { chunks }
    }

    fn from_region_data(data: PoiRegionData) -> Self {
        let mut region = Self::new();

        for (_chunk_key, chunk) in data.chunks {
            for (_section_key, section) in chunk.sections {
                for entry in section.records {
                    let key = (entry.x, entry.y, entry.z);
                    region.entries.insert(key, entry);
                }
            }
        }

        region.dirty = false;
        region
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
            return Ok(());
        }

        let data = self.to_region_data();

        let mut uncompressed = Vec::new();
        pumpkin_nbt::to_bytes_unnamed(&data, &mut uncompressed)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&uncompressed)?;
        let compressed = encoder.finish()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, compressed)?;

        self.dirty = false;
        Ok(())
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let compressed = std::fs::read(path)?;
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut uncompressed = Vec::new();
        decoder.read_to_end(&mut uncompressed)?;

        let data: PoiRegionData = pumpkin_nbt::from_bytes_unnamed(Cursor::new(uncompressed))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        Ok(Self::from_region_data(data))
    }
}

/// Region-based POI storage
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
        self.folder.join(format!("r.{}.{}.poi", rx, rz))
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
    pub fn get_in_square(&mut self, center: BlockPos, radius: i32, poi_type: Option<&str>) -> Vec<BlockPos> {
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
                let path = self.folder.join(format!("r.{}.{}.poi", rx, rz));
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
    pub fn loaded_region_count(&self) -> usize {
        self.regions.len()
    }

    /// Get total POI count across all loaded regions
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
    fn test_poi_storage() {
        let dir = std::env::temp_dir().join("pumpkin_poi_test");
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

        let _ = std::fs::remove_dir_all(&dir);
    }
}
