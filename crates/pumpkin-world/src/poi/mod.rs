use rustc_hash::FxHashMap;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use pumpkin_data::poi_type::PoiType;
use pumpkin_data::tag::{PointOfInterestType, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

mod record;
mod section_pos;

pub use record::{Occupancy, PoiRecord};
pub use section_pos::SectionPos;

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
    #[must_use]
    pub fn new_portal(pos: BlockPos) -> Self {
        Self {
            x: pos.0.x,
            y: pos.0.y,
            z: pos.0.z,
            poi_type: POI_TYPE_NETHER_PORTAL.to_string(),
            free_tickets: 0,
        }
    }

    #[must_use]
    pub const fn pos(&self) -> BlockPos {
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
    pub sections: FxHashMap<String, PoiSectionData>,
}

/// POI data for a single region (32x32 chunks) using MCA format
#[derive(Debug, Default)]
pub struct PoiRegion {
    /// Entries indexed by section, then by block position
    sections: FxHashMap<SectionPos, FxHashMap<(i32, i32, i32), PoiEntry>>,
    /// Track which chunks are dirty
    dirty_chunks: rustc_hash::FxHashSet<(i32, i32)>,
    dirty: bool,
}

impl PoiRegion {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    const fn pos_key(pos: &BlockPos) -> (i32, i32, i32) {
        (pos.0.x, pos.0.y, pos.0.z)
    }

    /// Get chunk index in MCA file (0-1023)
    const fn chunk_index(chunk_x: i32, chunk_z: i32) -> usize {
        let local_x = chunk_x & 31;
        let local_z = chunk_z & 31;
        ((local_z << 5) | local_x) as usize
    }

    fn mark_dirty(&mut self, chunk_x: i32, chunk_z: i32) {
        self.dirty_chunks.insert((chunk_x, chunk_z));
        self.dirty = true;
    }

    fn section_entries(
        &self,
        section: SectionPos,
    ) -> Option<&FxHashMap<(i32, i32, i32), PoiEntry>> {
        self.sections.get(&section)
    }

    fn entry_at(&self, pos: &BlockPos) -> Option<&PoiEntry> {
        self.section_entries(SectionPos::of(pos))?
            .get(&Self::pos_key(pos))
    }

    fn entry_at_mut(&mut self, pos: &BlockPos) -> Option<&mut PoiEntry> {
        let section = SectionPos::of(pos);
        let key = Self::pos_key(pos);
        if !self.sections.get(&section)?.contains_key(&key) {
            return None;
        }
        self.mark_dirty(pos.0.x >> 4, pos.0.z >> 4);
        self.sections.get_mut(&section)?.get_mut(&key)
    }

    pub fn add(&mut self, entry: PoiEntry) {
        self.mark_dirty(entry.x >> 4, entry.z >> 4);
        let section = SectionPos::of(&entry.pos());
        let key = (entry.x, entry.y, entry.z);
        self.sections.entry(section).or_default().insert(key, entry);
    }

    pub fn remove(&mut self, pos: &BlockPos) -> bool {
        let section = SectionPos::of(pos);
        let key = Self::pos_key(pos);
        let removed = self
            .sections
            .get_mut(&section)
            .and_then(|entries| entries.remove(&key))
            .is_some();
        if removed {
            self.mark_dirty(pos.0.x >> 4, pos.0.z >> 4);
        }
        removed
    }

    #[must_use]
    pub fn get_all(&self) -> Vec<&PoiEntry> {
        self.sections
            .values()
            .flat_map(|entries| entries.values())
            .collect()
    }

    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
        self.dirty_chunks.clear();
    }

    /// Group entries by chunk, then create chunk NBT data
    fn get_chunk_data(&self, chunk_x: i32, chunk_z: i32) -> Option<PoiChunkData> {
        let mut sections: FxHashMap<String, PoiSectionData> = FxHashMap::default();

        for (section_pos, entries) in &self.sections {
            if section_pos.x != chunk_x || section_pos.z != chunk_z {
                continue;
            }

            let section_key = section_pos.y.to_string();
            let section = sections
                .entry(section_key)
                .or_insert_with(|| PoiSectionData {
                    valid: 1,
                    records: Vec::new(),
                });
            section.records.extend(entries.values().cloned());
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
        let mut root = pumpkin_nbt::compound::NbtCompound::new();
        root.put_int("DataVersion", chunk_data.data_version);

        let mut sections_comp = pumpkin_nbt::compound::NbtCompound::new();
        for (sec_key, sec_data) in &chunk_data.sections {
            let mut sec_comp = pumpkin_nbt::compound::NbtCompound::new();
            sec_comp.put_byte("Valid", sec_data.valid);
            let mut rec_list = Vec::new();
            for rec in &sec_data.records {
                let mut rec_comp = pumpkin_nbt::compound::NbtCompound::new();
                rec_comp.put_int("x", rec.x);
                rec_comp.put_int("y", rec.y);
                rec_comp.put_int("z", rec.z);
                rec_comp.put_string("type", rec.poi_type.clone());
                rec_comp.put_int("free_tickets", rec.free_tickets);
                rec_list.push(pumpkin_nbt::tag::NbtTag::Compound(rec_comp));
            }
            sec_comp.put_list("Records", rec_list);
            sections_comp.put_compound(sec_key, sec_comp);
        }
        root.put_compound("Sections", sections_comp);

        let uncompressed = pumpkin_nbt::Nbt::from(root).write();
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&uncompressed)?;
        encoder.finish()
    }

    /// Decompress chunk data from bytes
    fn decompress_chunk_data(compressed: &[u8]) -> std::io::Result<PoiChunkData> {
        let mut decoder = ZlibDecoder::new(compressed);
        let mut uncompressed = Vec::new();
        decoder.read_to_end(&mut uncompressed)?;

        let mut cursor = Cursor::new(uncompressed);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let nbt = pumpkin_nbt::Nbt::read(&mut reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

        let data_version = nbt.get_int("DataVersion").unwrap_or(DATA_VERSION);
        let mut sections = FxHashMap::default();

        if let Some(sec_tag) = nbt.get_compound("Sections") {
            for (sec_key, tag) in &sec_tag.child_tags {
                if let pumpkin_nbt::tag::NbtTag::Compound(sec_comp) = tag {
                    let valid = sec_comp.get_byte("Valid").unwrap_or(1);
                    let mut records = Vec::new();
                    if let Some(pumpkin_nbt::tag::NbtTag::List(rec_list)) = sec_comp.get("Records")
                    {
                        for rec_t in rec_list {
                            if let pumpkin_nbt::tag::NbtTag::Compound(rc) = rec_t {
                                records.push(PoiEntry {
                                    x: rc.get_int("x").unwrap_or(0),
                                    y: rc.get_int("y").unwrap_or(0),
                                    z: rc.get_int("z").unwrap_or(0),
                                    poi_type: rc
                                        .get_string("type")
                                        .unwrap_or(POI_TYPE_NETHER_PORTAL)
                                        .to_string(),
                                    free_tickets: rc.get_int("free_tickets").unwrap_or(0),
                                });
                            }
                        }
                    }
                    sections.insert(sec_key.to_string(), PoiSectionData { valid, records });
                }
            }
        }

        Ok(PoiChunkData {
            data_version,
            sections,
        })
    }

    pub fn save(&mut self, path: &Path) -> std::io::Result<()> {
        if !self.dirty {
            return Ok(());
        }

        if self.sections.values().all(FxHashMap::is_empty) {
            // Don't save empty regions, delete the file if it exists
            if path.exists() {
                std::fs::remove_file(path)?;
            }
            self.dirty = false;
            self.dirty_chunks.clear();
            return Ok(());
        }

        // Build all chunk data
        let mut chunk_data_map: FxHashMap<usize, Vec<u8>> = FxHashMap::default();

        // Collect all unique chunks that have entries
        let mut chunks_with_data: rustc_hash::FxHashSet<(i32, i32)> =
            rustc_hash::FxHashSet::default();
        for section in self.sections.keys() {
            chunks_with_data.insert((section.x, section.z));
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
            .map_or(0, |d| d.as_secs() as u32);

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
                            let section_pos = SectionPos::of(&entry.pos());
                            let key = (entry.x, entry.y, entry.z);
                            region
                                .sections
                                .entry(section_pos)
                                .or_default()
                                .insert(key, entry);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse POI chunk at index {index}: {e}");
                }
            }
        }

        region.dirty = false;
        Ok(region)
    }
}

fn poi_type_string(poi_type: PoiType) -> String {
    format!("minecraft:{}", poi_type.name())
}

fn typed_record(entry: &PoiEntry) -> Option<PoiRecord> {
    Some(PoiRecord {
        pos: entry.pos(),
        poi_type: PoiType::from_name(&entry.poi_type)?,
        free_tickets: entry.free_tickets,
    })
}

fn distance_sq(center: BlockPos, pos: BlockPos) -> i64 {
    let dx = i64::from(pos.0.x) - i64::from(center.0.x);
    let dy = i64::from(pos.0.y) - i64::from(center.0.y);
    let dz = i64::from(pos.0.z) - i64::from(center.0.z);
    dx * dx + dy * dy + dz * dz
}

fn sort_records(records: &mut [PoiRecord], center: BlockPos) {
    records.sort_unstable_by_key(|record| {
        (
            distance_sq(center, record.pos),
            record.pos.0.x,
            record.pos.0.y,
            record.pos.0.z,
        )
    });
}

fn sections_at_distance(section: SectionPos, distance: i32) -> Vec<SectionPos> {
    if distance == 0 {
        return vec![section];
    }

    let mut result = Vec::new();
    for dx in -distance..=distance {
        let dx_abs = dx.abs();
        for dy in -distance..=distance {
            let remaining = distance - dx_abs - dy.abs();
            if remaining < 0 {
                continue;
            }
            if remaining == 0 {
                result.push(SectionPos::new(section.x + dx, section.y + dy, section.z));
            } else {
                result.push(SectionPos::new(
                    section.x + dx,
                    section.y + dy,
                    section.z + remaining,
                ));
                result.push(SectionPos::new(
                    section.x + dx,
                    section.y + dy,
                    section.z - remaining,
                ));
            }
        }
    }
    result
}

/// Region-based POI storage using MCA format
pub struct PoiStorage {
    /// Path to the poi folder
    folder: PathBuf,
    /// Loaded regions, keyed by (`region_x`, `region_z`)
    regions: FxHashMap<(i32, i32), PoiRegion>,
}

impl PoiStorage {
    #[must_use]
    pub fn new(poi_folder: PathBuf) -> Self {
        Self {
            folder: poi_folder,
            regions: FxHashMap::default(),
        }
    }

    const fn region_coords(pos: &BlockPos) -> (i32, i32) {
        let chunk_x = pos.0.x >> 4;
        let chunk_z = pos.0.z >> 4;
        (chunk_x >> 5, chunk_z >> 5)
    }

    fn region_path(&self, rx: i32, rz: i32) -> PathBuf {
        self.folder.join(format!("r.{rx}.{rz}.mca"))
    }

    fn get_or_load_region(&mut self, rx: i32, rz: i32) -> &mut PoiRegion {
        let path = self.region_path(rx, rz);
        self.regions.entry((rx, rz)).or_insert_with(|| {
            PoiRegion::load(&path).unwrap_or_else(|e| {
                if path.exists() {
                    warn!("Failed to load POI region {}: {}", path.display(), e);
                }
                PoiRegion::new()
            })
        })
    }

    fn records_in_square(
        &mut self,
        predicate: &dyn Fn(PoiType) -> bool,
        center: BlockPos,
        radius: i32,
        occupancy: Occupancy,
    ) -> Vec<PoiRecord> {
        let min_x = center.0.x - radius;
        let max_x = center.0.x + radius;
        let min_z = center.0.z - radius;
        let max_z = center.0.z + radius;
        let min_section_x = min_x >> 4;
        let max_section_x = max_x >> 4;
        let min_section_z = min_z >> 4;
        let max_section_z = max_z >> 4;

        let mut records = Vec::new();
        for rx in (min_section_x >> 5)..=(max_section_x >> 5) {
            for rz in (min_section_z >> 5)..=(max_section_z >> 5) {
                let region = self.get_or_load_region(rx, rz);
                for (section, entries) in &region.sections {
                    if section.x < min_section_x
                        || section.x > max_section_x
                        || section.z < min_section_z
                        || section.z > max_section_z
                    {
                        continue;
                    }
                    for entry in entries.values() {
                        if entry.x < min_x || entry.x > max_x || entry.z < min_z || entry.z > max_z
                        {
                            continue;
                        }
                        let Some(record) = typed_record(entry) else {
                            continue;
                        };
                        if predicate(record.poi_type) && occupancy.test(&record) {
                            records.push(record);
                        }
                    }
                }
            }
        }
        records
    }

    fn is_village_center(&mut self, section: SectionPos) -> bool {
        let region = self.get_or_load_region(section.x >> 5, section.z >> 5);
        region.section_entries(section).is_some_and(|entries| {
            entries.values().any(|entry| {
                typed_record(entry).is_some_and(|record| {
                    record
                        .poi_type
                        .has_tag(&PointOfInterestType::MINECRAFT_VILLAGE)
                        && record.is_occupied()
                })
            })
        })
    }

    pub fn add(&mut self, pos: BlockPos, poi_type: &str) {
        self.add_with_free_tickets(pos, poi_type, 0);
    }

    pub fn add_with_free_tickets(&mut self, pos: BlockPos, poi_type: &str, free_tickets: i32) {
        let (rx, rz) = Self::region_coords(&pos);
        let region = self.get_or_load_region(rx, rz);
        region.add(PoiEntry {
            x: pos.0.x,
            y: pos.0.y,
            z: pos.0.z,
            poi_type: poi_type.to_string(),
            free_tickets,
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

    pub fn add_poi(&mut self, pos: BlockPos, poi_type: PoiType) -> bool {
        let (rx, rz) = Self::region_coords(&pos);
        let region = self.get_or_load_region(rx, rz);
        if region
            .entry_at(&pos)
            .is_some_and(|entry| PoiType::from_name(&entry.poi_type) == Some(poi_type))
        {
            return false;
        }
        region.add(PoiEntry {
            x: pos.0.x,
            y: pos.0.y,
            z: pos.0.z,
            poi_type: poi_type_string(poi_type),
            free_tickets: poi_type.max_tickets(),
        });
        true
    }

    pub fn get_type(&mut self, pos: &BlockPos) -> Option<PoiType> {
        let (rx, rz) = Self::region_coords(pos);
        let region = self.get_or_load_region(rx, rz);
        let entry = region.entry_at(pos)?;
        PoiType::from_name(&entry.poi_type)
    }

    pub fn exists(&mut self, pos: &BlockPos, predicate: &dyn Fn(PoiType) -> bool) -> bool {
        self.get_type(pos).is_some_and(predicate)
    }

    pub fn get_records_in_square(
        &mut self,
        predicate: &dyn Fn(PoiType) -> bool,
        center: BlockPos,
        radius: i32,
        occupancy: Occupancy,
    ) -> Vec<PoiRecord> {
        let mut records = self.records_in_square(predicate, center, radius, occupancy);
        sort_records(&mut records, center);
        records
    }

    pub fn get_records_in_range(
        &mut self,
        predicate: &dyn Fn(PoiType) -> bool,
        center: BlockPos,
        radius: i32,
        occupancy: Occupancy,
    ) -> Vec<PoiRecord> {
        let radius_sqr = i64::from(radius) * i64::from(radius);
        let mut records = self.records_in_square(predicate, center, radius, occupancy);
        records.retain(|record| distance_sq(center, record.pos) <= radius_sqr);
        sort_records(&mut records, center);
        records
    }

    pub fn find_closest(
        &mut self,
        predicate: &dyn Fn(PoiType) -> bool,
        filter: &dyn Fn(BlockPos) -> bool,
        center: BlockPos,
        radius: i32,
        occupancy: Occupancy,
    ) -> Option<BlockPos> {
        self.get_records_in_range(predicate, center, radius, occupancy)
            .into_iter()
            .find(|record| filter(record.pos))
            .map(|record| record.pos)
    }

    pub fn find_closest_with_type(
        &mut self,
        predicate: &dyn Fn(PoiType) -> bool,
        center: BlockPos,
        radius: i32,
        occupancy: Occupancy,
    ) -> Option<(PoiType, BlockPos)> {
        self.get_records_in_range(predicate, center, radius, occupancy)
            .into_iter()
            .next()
            .map(|record| (record.poi_type, record.pos))
    }

    pub fn find_all_closest_first_with_type(
        &mut self,
        predicate: &dyn Fn(PoiType) -> bool,
        filter: &dyn Fn(BlockPos) -> bool,
        center: BlockPos,
        radius: i32,
        occupancy: Occupancy,
    ) -> Vec<(PoiType, BlockPos)> {
        self.get_records_in_range(predicate, center, radius, occupancy)
            .into_iter()
            .filter(|record| filter(record.pos))
            .map(|record| (record.poi_type, record.pos))
            .collect()
    }

    pub fn take(
        &mut self,
        predicate: &dyn Fn(PoiType) -> bool,
        filter: &dyn Fn(PoiType, BlockPos) -> bool,
        center: BlockPos,
        radius: i32,
    ) -> Option<BlockPos> {
        let record = self
            .get_records_in_range(predicate, center, radius, Occupancy::HasSpace)
            .into_iter()
            .find(|record| filter(record.poi_type, record.pos))?;
        self.acquire_ticket(&record.pos).then_some(record.pos)
    }

    pub fn get_random<R: rand::Rng + ?Sized>(
        &mut self,
        predicate: &dyn Fn(PoiType) -> bool,
        filter: &dyn Fn(BlockPos) -> bool,
        occupancy: Occupancy,
        center: BlockPos,
        radius: i32,
        rng: &mut R,
    ) -> Option<BlockPos> {
        let mut records = self.get_records_in_range(predicate, center, radius, occupancy);
        records.shuffle(rng);
        records
            .into_iter()
            .find(|record| filter(record.pos))
            .map(|record| record.pos)
    }

    pub fn release(&mut self, pos: &BlockPos) -> bool {
        let (rx, rz) = Self::region_coords(pos);
        let region = self.get_or_load_region(rx, rz);
        let Some(entry) = region.entry_at(pos) else {
            return false;
        };
        let Some(poi_type) = PoiType::from_name(&entry.poi_type) else {
            return false;
        };
        if entry.free_tickets >= poi_type.max_tickets() {
            return false;
        }
        let Some(entry) = region.entry_at_mut(pos) else {
            return false;
        };
        entry.free_tickets += 1;
        true
    }

    pub fn acquire_ticket(&mut self, pos: &BlockPos) -> bool {
        let (rx, rz) = Self::region_coords(pos);
        let region = self.get_or_load_region(rx, rz);
        let Some(entry) = region.entry_at(pos) else {
            return false;
        };
        if PoiType::from_name(&entry.poi_type).is_none() || entry.free_tickets <= 0 {
            return false;
        }
        let Some(entry) = region.entry_at_mut(pos) else {
            return false;
        };
        entry.free_tickets -= 1;
        true
    }

    pub const MAX_VILLAGE_DISTANCE: i32 = 6;

    pub fn sections_to_village(&mut self, section: SectionPos) -> i32 {
        for distance in 0..=Self::MAX_VILLAGE_DISTANCE {
            if sections_at_distance(section, distance)
                .into_iter()
                .any(|candidate| self.is_village_center(candidate))
            {
                return distance;
            }
        }
        Self::MAX_VILLAGE_DISTANCE + 1
    }

    pub fn refresh_section(&mut self, section: SectionPos, found: &[(BlockPos, PoiType)]) {
        let region = self.get_or_load_region(section.x >> 5, section.z >> 5);

        let found_positions: FxHashMap<(i32, i32, i32), PoiType> = found
            .iter()
            .map(|(pos, poi_type)| ((pos.0.x, pos.0.y, pos.0.z), *poi_type))
            .collect();

        let stale: Vec<(i32, i32, i32)> = region
            .section_entries(section)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|(key, entry)| {
                        if PoiType::from_name(&entry.poi_type).is_none()
                            || found_positions.contains_key(key)
                        {
                            None
                        } else {
                            Some(*key)
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        for key in stale {
            region.remove(&BlockPos(Vector3::new(key.0, key.1, key.2)));
        }

        for (pos, poi_type) in found {
            let unchanged = region
                .entry_at(pos)
                .is_some_and(|entry| PoiType::from_name(&entry.poi_type) == Some(*poi_type));
            if unchanged {
                continue;
            }
            region.add(PoiEntry {
                x: pos.0.x,
                y: pos.0.y,
                z: pos.0.z,
                poi_type: poi_type_string(*poi_type),
                free_tickets: poi_type.max_tickets(),
            });
        }
    }

    /// Get all POI positions within a square radius (for portal search)
    #[expect(clippy::similar_names)]
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

    /// Finds the closest POI whose type matches `matches`, considering
    /// entries within `radius` blocks of `center` on the x/z axes (like
    /// vanilla's `PoiManager.findClosestWithType`: a chebyshev square gather
    /// followed by picking the smallest 3D squared distance).
    ///
    /// Returns the entry's position together with its type.
    pub fn find_closest_matching(
        &mut self,
        center: BlockPos,
        radius: i32,
        matches: impl Fn(&str) -> bool,
    ) -> Option<(BlockPos, String)> {
        self.find_closest_where(center, radius, |poi_type, _| matches(poi_type))
    }

    pub fn find_closest_where(
        &mut self,
        center: BlockPos,
        radius: i32,
        matches: impl Fn(&str, BlockPos) -> bool,
    ) -> Option<(BlockPos, String)> {
        let min_rx = ((center.0.x - radius) >> 4) >> 5;
        let max_rx = ((center.0.x + radius) >> 4) >> 5;
        let min_rz = ((center.0.z - radius) >> 4) >> 5;
        let max_rz = ((center.0.z + radius) >> 4) >> 5;

        let mut best: Option<(BlockPos, String, i64)> = None;

        for rx in min_rx..=max_rx {
            for rz in min_rz..=max_rz {
                let region = self.get_or_load_region(rx, rz);
                for entry in region.get_all() {
                    if (entry.x - center.0.x).abs() > radius
                        || (entry.z - center.0.z).abs() > radius
                        || !matches(&entry.poi_type, entry.pos())
                    {
                        continue;
                    }

                    let dx = i64::from(entry.x - center.0.x);
                    let dy = i64::from(entry.y - center.0.y);
                    let dz = i64::from(entry.z - center.0.z);
                    let distance_sq = dx * dx + dy * dy + dz * dz;

                    if best.as_ref().is_none_or(|(_, _, d)| distance_sq < *d) {
                        best = Some((entry.pos(), entry.poi_type.clone(), distance_sq));
                    }
                }
            }
        }

        best.map(|(pos, poi_type, _)| (pos, poi_type))
    }

    pub fn save_all(&mut self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.folder)?;

        let mut saved = 0;
        for ((rx, rz), region) in &mut self.regions {
            if region.is_dirty() {
                let path = self.folder.join(format!("r.{rx}.{rz}.mca"));
                region.save(&path)?;
                saved += 1;
            }
        }

        if saved > 0 {
            info!("Saved {saved} POI region(s)");
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
    fn poi_entry() {
        let entry = PoiEntry::new_portal(BlockPos(Vector3::new(100, 64, 200)));
        assert_eq!(entry.x, 100);
        assert_eq!(entry.y, 64);
        assert_eq!(entry.z, 200);
        assert_eq!(entry.poi_type, POI_TYPE_NETHER_PORTAL);
    }

    #[test]
    fn poi_region() {
        let mut region = PoiRegion::new();
        region.add(PoiEntry::new_portal(BlockPos(Vector3::new(100, 64, 200))));
        region.add(PoiEntry::new_portal(BlockPos(Vector3::new(101, 64, 200))));

        assert_eq!(region.get_all().len(), 2);
        assert!(region.is_dirty());

        region.remove(&BlockPos(Vector3::new(100, 64, 200)));
        assert_eq!(region.get_all().len(), 1);
    }

    #[test]
    fn poi_find_closest_matching() {
        let mut storage = PoiStorage::new(std::env::temp_dir().join("pumpkin_poi_closest_test"));

        storage.add_portal(BlockPos(Vector3::new(100, 64, 100)));
        storage.add_portal(BlockPos(Vector3::new(120, 64, 100)));
        storage.add(BlockPos(Vector3::new(101, 64, 100)), "minecraft:home");

        let center = BlockPos(Vector3::new(105, 64, 100));
        let (pos, poi_type) = storage
            .find_closest_matching(center, 256, |t| t == POI_TYPE_NETHER_PORTAL)
            .unwrap();
        assert_eq!(pos, BlockPos(Vector3::new(100, 64, 100)));
        assert_eq!(poi_type, POI_TYPE_NETHER_PORTAL);

        // The overall closest one ignores the type filter mismatch above.
        let (pos, poi_type) = storage
            .find_closest_matching(center, 256, |_| true)
            .unwrap();
        assert_eq!(pos, BlockPos(Vector3::new(101, 64, 100)));
        assert_eq!(poi_type, "minecraft:home");

        assert!(
            storage
                .find_closest_matching(center, 256, |t| t == "minecraft:lodestone")
                .is_none()
        );
        // Out of horizontal range.
        assert!(
            storage
                .find_closest_matching(BlockPos(Vector3::new(1000, 64, 100)), 16, |_| true)
                .is_none()
        );
    }

    #[test]
    fn poi_storage_mca() {
        let dir = std::env::temp_dir().join("pumpkin_poi_mca_test");
        let _ = std::fs::remove_dir_all(&dir);

        let mut storage = PoiStorage::new(dir.join("poi"));

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
        let mut storage2 = PoiStorage::new(dir.join("poi"));
        let results2 = storage2.get_in_square(
            BlockPos(Vector3::new(105, 64, 100)),
            16,
            Some(POI_TYPE_NETHER_PORTAL),
        );
        assert_eq!(results2.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn poi_ticket_round_trip() {
        let dir = std::env::temp_dir().join("pumpkin_poi_ticket_test");
        let _ = std::fs::remove_dir_all(&dir);
        let mut storage = PoiStorage::new(dir);

        let meeting = BlockPos(Vector3::new(0, 64, 0));
        let home = BlockPos(Vector3::new(5, 64, 0));
        assert!(storage.add_poi(meeting, PoiType::Meeting));
        assert!(storage.add_poi(home, PoiType::Home));

        let meeting_records = storage.get_records_in_range(
            &|poi_type| poi_type == PoiType::Meeting,
            meeting,
            1,
            Occupancy::Any,
        );
        assert_eq!(meeting_records.len(), 1);
        assert_eq!(
            meeting_records[0].free_tickets,
            PoiType::Meeting.max_tickets()
        );

        let center = BlockPos(Vector3::new(0, 64, 0));
        assert_eq!(
            storage.take(
                &|poi_type| poi_type == PoiType::Home,
                &|_, _| true,
                center,
                16
            ),
            Some(home)
        );
        assert_eq!(
            storage.take(
                &|poi_type| poi_type == PoiType::Home,
                &|_, _| true,
                center,
                16
            ),
            None
        );
        assert!(storage.release(&home));
        assert!(!storage.release(&home));
        assert!(storage.acquire_ticket(&home));
        assert!(!storage.acquire_ticket(&home));
    }

    #[test]
    fn poi_occupancy_filters() {
        let dir = std::env::temp_dir().join("pumpkin_poi_occupancy_test");
        let _ = std::fs::remove_dir_all(&dir);
        let mut storage = PoiStorage::new(dir);

        let meeting = BlockPos(Vector3::new(0, 64, 0));
        let home = BlockPos(Vector3::new(10, 64, 0));
        assert!(storage.add_poi(meeting, PoiType::Meeting));
        assert!(storage.add_poi(home, PoiType::Home));
        assert_eq!(
            storage.take(&|poi_type| poi_type == PoiType::Home, &|_, _| true, home, 1),
            Some(home)
        );

        let has_space = storage.get_records_in_range(&|_| true, meeting, 32, Occupancy::HasSpace);
        assert_eq!(has_space.len(), 1);
        assert_eq!(has_space[0].pos, meeting);

        let occupied = storage.get_records_in_range(&|_| true, meeting, 32, Occupancy::IsOccupied);
        assert_eq!(occupied.len(), 1);
        assert_eq!(occupied[0].pos, home);

        let any = storage.get_records_in_range(&|_| true, meeting, 32, Occupancy::Any);
        assert_eq!(any.len(), 2);
    }

    #[test]
    fn poi_sections_to_village() {
        let dir = std::env::temp_dir().join("pumpkin_poi_village_test");
        let _ = std::fs::remove_dir_all(&dir);
        let mut storage = PoiStorage::new(dir);

        let home = BlockPos(Vector3::new(0, 64, 0));
        assert!(storage.add_poi(home, PoiType::Home));
        let origin = SectionPos::of(&home);
        assert_eq!(
            storage.sections_to_village(origin),
            PoiStorage::MAX_VILLAGE_DISTANCE + 1
        );

        assert_eq!(
            storage.take(&|poi_type| poi_type == PoiType::Home, &|_, _| true, home, 1),
            Some(home)
        );
        assert_eq!(storage.sections_to_village(origin), 0);
        assert_eq!(
            storage.sections_to_village(SectionPos::new(origin.x + 3, origin.y, origin.z)),
            3
        );
        assert_eq!(
            storage.sections_to_village(SectionPos::new(origin.x, origin.y, origin.z + 6)),
            6
        );
        assert_eq!(
            storage.sections_to_village(SectionPos::new(origin.x, origin.y + 7, origin.z)),
            PoiStorage::MAX_VILLAGE_DISTANCE + 1
        );
    }

    #[test]
    fn poi_refresh_section() {
        let dir = std::env::temp_dir().join("pumpkin_poi_refresh_test");
        let _ = std::fs::remove_dir_all(&dir);
        let mut storage = PoiStorage::new(dir);

        let keep = BlockPos(Vector3::new(1, 64, 1));
        let replace = BlockPos(Vector3::new(2, 64, 2));
        let stale = BlockPos(Vector3::new(3, 64, 3));
        let unknown = BlockPos(Vector3::new(4, 64, 4));
        let new = BlockPos(Vector3::new(5, 64, 5));

        assert!(storage.add_poi(keep, PoiType::Home));
        assert!(storage.add_poi(replace, PoiType::Home));
        assert!(storage.add_poi(stale, PoiType::Home));
        storage.add(unknown, "minecraft:unknown_poi");
        assert_eq!(
            storage.take(&|poi_type| poi_type == PoiType::Home, &|_, _| true, keep, 0),
            Some(keep)
        );

        let section = SectionPos::of(&keep);
        storage.refresh_section(
            section,
            &[
                (keep, PoiType::Home),
                (replace, PoiType::Meeting),
                (new, PoiType::Home),
            ],
        );

        assert_eq!(storage.get_type(&keep), Some(PoiType::Home));
        assert!(!storage.acquire_ticket(&keep));
        assert_eq!(storage.get_type(&replace), Some(PoiType::Meeting));
        let replaced = storage.get_records_in_range(
            &|poi_type| poi_type == PoiType::Meeting,
            replace,
            0,
            Occupancy::Any,
        );
        assert_eq!(replaced.len(), 1);
        assert_eq!(replaced[0].free_tickets, PoiType::Meeting.max_tickets());
        assert_eq!(storage.get_type(&stale), None);
        assert_eq!(
            storage.get_in_square(unknown, 0, Some("minecraft:unknown_poi")),
            vec![unknown]
        );
        assert_eq!(storage.get_type(&new), Some(PoiType::Home));
        assert!(storage.acquire_ticket(&new));
        assert_eq!(storage.total_poi_count(), 4);
    }

    #[test]
    fn poi_save_and_reload_typed() {
        let dir = std::env::temp_dir().join("pumpkin_poi_typed_reload_test");
        let _ = std::fs::remove_dir_all(&dir);

        let home = BlockPos(Vector3::new(0, 64, 0));
        let meeting = BlockPos(Vector3::new(20, 64, 0));
        let mut storage = PoiStorage::new(dir.join("poi"));
        assert!(storage.add_poi(home, PoiType::Home));
        assert!(storage.add_poi(meeting, PoiType::Meeting));
        assert_eq!(
            storage.take(&|poi_type| poi_type == PoiType::Home, &|_, _| true, home, 1),
            Some(home)
        );
        storage.save_all().unwrap();

        let mut reloaded = PoiStorage::new(dir.join("poi"));
        assert_eq!(reloaded.get_type(&home), Some(PoiType::Home));
        let home_records = reloaded.get_records_in_range(
            &|poi_type| poi_type == PoiType::Home,
            home,
            8,
            Occupancy::IsOccupied,
        );
        assert_eq!(home_records.len(), 1);
        assert_eq!(home_records[0].free_tickets, 0);
        let meeting_records = reloaded.get_records_in_range(
            &|poi_type| poi_type == PoiType::Meeting,
            home,
            32,
            Occupancy::Any,
        );
        assert_eq!(meeting_records.len(), 1);
        assert_eq!(
            meeting_records[0].free_tickets,
            PoiType::Meeting.max_tickets()
        );
        assert_eq!(
            reloaded.find_closest_with_type(
                &|poi_type| poi_type == PoiType::Meeting,
                home,
                32,
                Occupancy::Any
            ),
            Some((PoiType::Meeting, meeting))
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
