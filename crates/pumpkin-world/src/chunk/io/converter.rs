use std::path::{Path, PathBuf};

use bytes::Bytes;
use pumpkin_config::chunk::{AnvilChunkConfig, ChunkConfig};
use pumpkin_util::math::vector2::Vector2;
use rayon::prelude::*;
use thiserror::Error;
use tracing::warn;

use crate::chunk::{
    ChunkData, ChunkReadingError, ChunkWritingError,
    format::{
        anvil::{AnvilRegion, CHUNK_COUNT, REGION_SIZE},
        linear::LinearV2File,
        pump::{PUMP_CHUNK_MAGIC, PumpFile},
    },
    io::RegionFile,
};

#[derive(Error, Debug)]
pub enum ChunkConversionError {
    #[error("Failed reading source chunk: {0}")]
    Read(#[from] ChunkReadingError),
    #[error("Failed writing target chunk: {0}")]
    Write(#[from] ChunkWritingError),
    #[error("I/O error during conversion: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionFormat {
    Anvil,
    Linear,
    Pump,
}

impl RegionFormat {
    #[must_use]
    pub const fn from_config(config: &ChunkConfig) -> Self {
        match config {
            ChunkConfig::Anvil(_) => Self::Anvil,
            ChunkConfig::Linear => Self::Linear,
            ChunkConfig::Pump => Self::Pump,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Anvil => "anvil",
            Self::Linear => "linear",
            Self::Pump => "pump",
        }
    }

    #[must_use]
    pub fn file_name(self, region: Vector2<i32>) -> String {
        match self {
            Self::Anvil => AnvilRegion::file_name(region),
            Self::Linear => LinearV2File::file_name(region),
            Self::Pump => PumpFile::file_name(region),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ConversionStats {
    pub regions_converted: usize,
    pub chunks_converted: usize,
}

impl ConversionStats {
    const fn add(&mut self, other: Self) {
        self.regions_converted += other.regions_converted;
        self.chunks_converted += other.chunks_converted;
    }
}

/// Parses a region filename (`r.<x>.<z>.<ext>`) into its region coordinates and format.
#[must_use]
pub fn parse_region_file_name(file_name: &str) -> Option<(Vector2<i32>, RegionFormat)> {
    let rest = file_name.strip_prefix("r.")?;
    let (x_str, rest) = rest.split_once('.')?;
    let (z_str, ext) = rest.split_once('.')?;
    let x = x_str.parse::<i32>().ok()?;
    let z = z_str.parse::<i32>().ok()?;
    let format = match ext {
        "mca" => RegionFormat::Anvil,
        "linear" => RegionFormat::Linear,
        "pump" => RegionFormat::Pump,
        _ => return None,
    };
    Some((Vector2::new(x, z), format))
}

fn transcode_payload(
    payload: Bytes,
    chunk_pos: Vector2<i32>,
    dst_format: RegionFormat,
) -> Result<Bytes, ChunkReadingError> {
    match dst_format {
        RegionFormat::Pump => {
            if !payload.starts_with(&PUMP_CHUNK_MAGIC)
                && let Ok(chunk) = ChunkData::internal_from_bytes(&payload, chunk_pos)
            {
                return Ok(chunk.internal_to_pump_bytes());
            }
        }
        RegionFormat::Anvil | RegionFormat::Linear => {
            if payload.starts_with(&PUMP_CHUNK_MAGIC) {
                let chunk = ChunkData::internal_from_pump_bytes(&payload, chunk_pos)
                    .map_err(ChunkReadingError::ParsingError)?;
                return Ok(chunk.internal_to_bytes());
            }
        }
    }
    Ok(payload)
}

fn transcode_chunks<Src: RegionFile, Dst: RegionFile>(
    src: &Src,
    dst: &mut Dst,
    region: Vector2<i32>,
    dst_format: RegionFormat,
    dst_config: &Dst::Config,
    src_overwrites_dst: bool,
) -> Result<usize, ChunkConversionError> {
    let mut raw_chunks = Vec::new();
    for index in 0..CHUNK_COUNT {
        if !src_overwrites_dst && dst.read_chunk(index)?.is_some() {
            continue;
        }
        if let Some(raw) = src.read_chunk(index)? {
            raw_chunks.push((index, raw));
        }
    }

    let encoded: Result<Vec<(usize, Dst::Encoded)>, ChunkConversionError> = raw_chunks
        .into_par_iter()
        .map(|(index, raw)| {
            let chunk_pos = Vector2::new(
                region.x * REGION_SIZE as i32 + (index % REGION_SIZE) as i32,
                region.y * REGION_SIZE as i32 + (index / REGION_SIZE) as i32,
            );
            let decoded = Src::decode_chunk(raw)?;
            let payload = transcode_payload(decoded, chunk_pos, dst_format)?;
            let encoded = Dst::encode_chunk(payload, dst_config)?;
            Ok((index, encoded))
        })
        .collect();
    let encoded = encoded?;
    let count = encoded.len();
    for (index, chunk) in encoded {
        dst.stage_chunk(index, chunk);
    }
    if dst.has_staged() {
        dst.flush(true).map_err(ChunkWritingError::IoError)?;
    }
    Ok(count)
}

fn convert_into_dst<Dst: RegionFile>(
    src_path: &Path,
    src_format: RegionFormat,
    dst_path: &Path,
    dst_format: RegionFormat,
    region: Vector2<i32>,
    dst_config: &Dst::Config,
    remove_source: bool,
) -> Result<usize, ChunkConversionError> {
    if src_path == dst_path {
        return Ok(0);
    }

    let src_mtime = std::fs::metadata(src_path).and_then(|m| m.modified()).ok();
    let dst_mtime = std::fs::metadata(dst_path).and_then(|m| m.modified()).ok();
    let src_overwrites_dst = match (src_mtime, dst_mtime) {
        (Some(s), Some(d)) => s >= d,
        _ => true,
    };

    let mut dst = Dst::open(dst_path, region)?;
    let mut external_files = Vec::new();
    let count = match src_format {
        RegionFormat::Anvil => {
            let src = AnvilRegion::open(src_path, region)?;
            if remove_source {
                external_files = src.external_chunk_paths();
            }
            transcode_chunks(
                &src,
                &mut dst,
                region,
                dst_format,
                dst_config,
                src_overwrites_dst,
            )?
        }
        RegionFormat::Linear => {
            let src = LinearV2File::open(src_path, region)?;
            transcode_chunks(
                &src,
                &mut dst,
                region,
                dst_format,
                dst_config,
                src_overwrites_dst,
            )?
        }
        RegionFormat::Pump => {
            let src = PumpFile::open(src_path, region)?;
            transcode_chunks(
                &src,
                &mut dst,
                region,
                dst_format,
                dst_config,
                src_overwrites_dst,
            )?
        }
    };
    drop(dst);

    if remove_source {
        std::fs::remove_file(src_path)?;
        for external in external_files {
            if let Err(err) = std::fs::remove_file(&external)
                && err.kind() != std::io::ErrorKind::NotFound
            {
                warn!(
                    "Failed to remove external chunk {}: {err}",
                    external.display()
                );
            }
        }
    }

    Ok(count)
}

/// Converts a single region file at `src_path` into the format configured by `target`.
pub fn convert_region(
    src_path: &Path,
    src_format: RegionFormat,
    region: Vector2<i32>,
    target: &ChunkConfig,
    remove_source: bool,
) -> Result<usize, ChunkConversionError> {
    let dst_format = RegionFormat::from_config(target);
    if src_format == dst_format {
        return Ok(0);
    }
    let dst_path = src_path.with_file_name(dst_format.file_name(region));
    match target {
        ChunkConfig::Anvil(config) => convert_into_dst::<AnvilRegion>(
            src_path,
            src_format,
            &dst_path,
            dst_format,
            region,
            config,
            remove_source,
        ),
        ChunkConfig::Linear => convert_into_dst::<LinearV2File>(
            src_path,
            src_format,
            &dst_path,
            dst_format,
            region,
            &(),
            remove_source,
        ),
        ChunkConfig::Pump => convert_into_dst::<PumpFile>(
            src_path,
            src_format,
            &dst_path,
            dst_format,
            region,
            &(),
            remove_source,
        ),
    }
}

/// Converts all region files in `folder` (such as a dimension's `region/` or `entities/` folder)
/// that are in another format into `target`.
pub fn convert_folder(
    folder: &Path,
    target: &ChunkConfig,
    remove_source: bool,
) -> Result<ConversionStats, ChunkConversionError> {
    let entries = match std::fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ConversionStats::default());
        }
        Err(err) => return Err(ChunkConversionError::Io(err)),
    };

    let target_format = RegionFormat::from_config(target);
    let mut to_convert: Vec<(PathBuf, Vector2<i32>, RegionFormat)> = Vec::new();
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(name_str) = file_name.to_str() else {
            continue;
        };
        if let Some((region, format)) = parse_region_file_name(name_str)
            && format != target_format
        {
            to_convert.push((entry.path(), region, format));
        }
    }

    if to_convert.is_empty() {
        return Ok(ConversionStats::default());
    }

    let results: Result<Vec<usize>, ChunkConversionError> = to_convert
        .into_par_iter()
        .map(|(path, region, src_format)| {
            convert_region(&path, src_format, region, target, remove_source)
        })
        .collect();
    let counts = results?;

    Ok(ConversionStats {
        regions_converted: counts.len(),
        chunks_converted: counts.into_iter().sum(),
    })
}

/// Discovers all dimension folders under `root_folder` (both 26.2 `dimensions/<ns>/<name>` and
/// legacy root/`DIM-1`/`DIM1`) and converts their `region/` and `entities/` directories to
/// `target`.
pub fn convert_world(
    root_folder: &Path,
    target: &ChunkConfig,
    remove_source: bool,
) -> Result<ConversionStats, ChunkConversionError> {
    let mut dim_folders = vec![
        root_folder.to_path_buf(),
        root_folder.join("DIM-1"),
        root_folder.join("DIM1"),
    ];

    let dimensions_dir = root_folder.join("dimensions");
    if let Ok(namespaces) = std::fs::read_dir(&dimensions_dir) {
        for ns_entry in namespaces.flatten() {
            if let Ok(dims) = std::fs::read_dir(ns_entry.path()) {
                for dim_entry in dims.flatten() {
                    dim_folders.push(dim_entry.path());
                }
            }
        }
    }

    let mut total = ConversionStats::default();
    for dim_folder in dim_folders {
        for subfolder in ["region", "entities"] {
            let folder = dim_folder.join(subfolder);
            if folder.is_dir() {
                total.add(convert_folder(&folder, target, remove_source)?);
            }
        }
    }
    Ok(total)
}

/// Parses a format name (`"pump"`, `"anvil"`, `"mca"`, `"linear"`) into a [`ChunkConfig`].
#[must_use]
pub fn parse_chunk_config(format: &str, anvil_default: AnvilChunkConfig) -> Option<ChunkConfig> {
    match format.trim().to_ascii_lowercase().as_str() {
        "pump" | ".pump" => Some(ChunkConfig::Pump),
        "anvil" | "mca" | ".mca" => Some(ChunkConfig::Anvil(anvil_default)),
        "linear" | ".linear" => Some(ChunkConfig::Linear),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use tempfile::TempDir;

    #[test]
    fn converts_regions_across_anvil_linear_and_pump() {
        let dir = TempDir::new().unwrap();
        let region_dir = dir.path().join("region");
        std::fs::create_dir_all(&region_dir).unwrap();
        let coords = Vector2::new(1, -2);

        let mca_path = region_dir.join(AnvilRegion::file_name(coords));
        let anvil_cfg = AnvilChunkConfig::default();
        let mut anvil = AnvilRegion::open(&mca_path, coords).unwrap();
        let payload_a = Bytes::from(vec![0xAA; 512]);
        let payload_b = Bytes::from(vec![0xBB; 1024]);
        anvil.stage_chunk(
            0,
            AnvilRegion::encode_chunk(payload_a.clone(), &anvil_cfg).unwrap(),
        );
        anvil.stage_chunk(
            511,
            AnvilRegion::encode_chunk(payload_b.clone(), &anvil_cfg).unwrap(),
        );
        anvil.flush(true).unwrap();
        drop(anvil);

        // Anvil -> Pump
        let stats = convert_folder(&region_dir, &ChunkConfig::Pump, true).unwrap();
        assert_eq!(stats.regions_converted, 1);
        assert_eq!(stats.chunks_converted, 2);
        assert!(!mca_path.exists());
        let pump_path = region_dir.join(PumpFile::file_name(coords));
        assert!(pump_path.exists());

        let pump = PumpFile::open(&pump_path, coords).unwrap();
        assert_eq!(
            PumpFile::decode_chunk(pump.read_chunk(0).unwrap().unwrap()).unwrap(),
            payload_a
        );
        assert_eq!(
            PumpFile::decode_chunk(pump.read_chunk(511).unwrap().unwrap()).unwrap(),
            payload_b
        );
        drop(pump);

        // Pump -> Linear
        let stats = convert_folder(&region_dir, &ChunkConfig::Linear, true).unwrap();
        assert_eq!(stats.regions_converted, 1);
        assert_eq!(stats.chunks_converted, 2);
        assert!(!pump_path.exists());
        let linear_path = region_dir.join(LinearV2File::file_name(coords));
        assert!(linear_path.exists());

        // Linear -> Anvil
        let stats = convert_folder(&region_dir, &ChunkConfig::Anvil(anvil_cfg), true).unwrap();
        assert_eq!(stats.regions_converted, 1);
        assert_eq!(stats.chunks_converted, 2);
        assert!(!linear_path.exists());
        assert!(mca_path.exists());

        let reopened = AnvilRegion::open(&mca_path, coords).unwrap();
        assert_eq!(
            AnvilRegion::decode_chunk(reopened.read_chunk(0).unwrap().unwrap()).unwrap(),
            payload_a
        );
        assert_eq!(
            AnvilRegion::decode_chunk(reopened.read_chunk(511).unwrap().unwrap()).unwrap(),
            payload_b
        );
    }

    #[test]
    fn transcodes_chunk_data_between_nbt_and_pump_binary() {
        use crate::{
            chunk::{
                ChunkHeightmaps, ChunkLight, ChunkSections,
                format::anvil::SingleChunkDataSerializer,
            },
            tick::scheduler::ChunkTickScheduler,
        };
        use pumpkin_data::{Block, chunk::ChunkStatus};
        use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
        use rustc_hash::FxHashMap;
        use std::sync::{
            Mutex,
            atomic::{AtomicBool, AtomicU64},
        };

        let dir = TempDir::new().unwrap();
        let region_dir = dir.path().join("region");
        std::fs::create_dir_all(&region_dir).unwrap();
        let region_coords = Vector2::new(0, 0);
        let chunk_pos = Vector2::new(3, 5);
        let chunk_index = 5 * REGION_SIZE + 3;

        let sections = ChunkSections::new(24, -64);
        sections.set_block_absolute_y(2, -60, 4, Block::DIAMOND_ORE.default_state.id);
        sections.set_block_absolute_y(7, 64, 9, Block::OAK_LOG.default_state.id);

        let chunk = ChunkData {
            section: sections,
            heightmap: Mutex::new(ChunkHeightmaps::default()),
            x: chunk_pos.x,
            z: chunk_pos.y,
            block_ticks: ChunkTickScheduler::default(),
            fluid_ticks: ChunkTickScheduler::default(),
            pending_block_entities: Mutex::new(FxHashMap::default()),
            light_engine: Mutex::new(ChunkLight::default()),
            light_populated: AtomicBool::new(true),
            status: ChunkStatus::Full,
            dirty: AtomicBool::new(false),
            blending_data: None,
            inhabited_time: AtomicU64::new(12345),
            custom_data: Mutex::new(NbtCompound::default()),
        };
        chunk.set_custom_data("test_ns", "flag", NbtTag::Int(99));

        let mca_path = region_dir.join(AnvilRegion::file_name(region_coords));
        let anvil_cfg = AnvilChunkConfig::default();
        let mut anvil = AnvilRegion::open(&mca_path, region_coords).unwrap();
        let nbt_bytes = AnvilRegion::serialize_chunk(&chunk).unwrap();
        assert_eq!(nbt_bytes[0], 0x0A);
        anvil.stage_chunk(
            chunk_index,
            AnvilRegion::encode_chunk(nbt_bytes, &anvil_cfg).unwrap(),
        );
        anvil.flush(true).unwrap();
        drop(anvil);

        // Convert Anvil (.mca with Java NBT) -> Pump (.pump with PCHK binary)
        convert_folder(&region_dir, &ChunkConfig::Pump, true).unwrap();
        let pump_path = region_dir.join(PumpFile::file_name(region_coords));
        let pump = PumpFile::open(&pump_path, region_coords).unwrap();
        let pump_decoded =
            PumpFile::decode_chunk(pump.read_chunk(chunk_index).unwrap().unwrap()).unwrap();
        assert!(pump_decoded.starts_with(&PUMP_CHUNK_MAGIC));
        let parsed_pump = ChunkData::from_bytes(&pump_decoded, chunk_pos).unwrap();
        assert_eq!(
            parsed_pump.section.get_block_absolute_y(2, -60, 4),
            Some(Block::DIAMOND_ORE.default_state.id)
        );
        assert_eq!(
            parsed_pump.section.get_block_absolute_y(7, 64, 9),
            Some(Block::OAK_LOG.default_state.id)
        );
        assert_eq!(
            parsed_pump.get_custom_data("test_ns", "flag"),
            Some(NbtTag::Int(99))
        );
        drop(pump);

        // Convert Pump (.pump with PCHK binary) -> Anvil (.mca with Java NBT)
        convert_folder(&region_dir, &ChunkConfig::Anvil(anvil_cfg), true).unwrap();
        let anvil = AnvilRegion::open(&mca_path, region_coords).unwrap();
        let anvil_decoded =
            AnvilRegion::decode_chunk(anvil.read_chunk(chunk_index).unwrap().unwrap()).unwrap();
        assert_eq!(anvil_decoded[0], 0x0A);
        let parsed_anvil = ChunkData::from_bytes(&anvil_decoded, chunk_pos).unwrap();
        assert_eq!(
            parsed_anvil.section.get_block_absolute_y(2, -60, 4),
            Some(Block::DIAMOND_ORE.default_state.id)
        );
        assert_eq!(
            parsed_anvil.section.get_block_absolute_y(7, 64, 9),
            Some(Block::OAK_LOG.default_state.id)
        );
        assert_eq!(
            parsed_anvil.get_custom_data("test_ns", "flag"),
            Some(NbtTag::Int(99))
        );
    }
}
