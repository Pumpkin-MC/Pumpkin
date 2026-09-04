//! TEMPORARY -- development scaffolding for the light engine work, not meant to ship.
//!
//! Sky and block light parity against a real vanilla save.
//!
//! Reads vanilla chunks, relights them with Pumpkin's worldgen light pass and compares the
//! result.
//!
//! `PUMPKIN_VANILLA_SAVE="/path/to/world" cargo test -p pumpkin-world --test lighting_vanilla_parity`
//!
//! Skips when the variable is unset, so a checkout without a save still passes.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::panic
)]

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use flate2::read::{GzDecoder, ZlibDecoder};
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::BlockStateId;
use pumpkin_data::dimension::Dimension;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::ProtoChunk;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::chunk_system::generation_cache::Cache;
use pumpkin_world::chunk_system::{Chunk, StagedChunkEnum};
use pumpkin_world::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
use pumpkin_world::lighting::LightEngine;

const SAVE_ENV: &str = "PUMPKIN_VANILLA_SAVE";
/// Raise for a wider sweep; relighting one chunk rebuilds a whole `ProtoChunk`, so the
/// default keeps the test at a few seconds.
const CENTERS_ENV: &str = "PUMPKIN_VANILLA_CHUNKS";
const MAX_CENTERS: usize = 8;
const MAX_REPORTED: usize = 12;

/// The light pass reaches one chunk out, so a path that enters the rim from further away is
/// not reproduced. In practice this is a handful of cells at the bottom of deep water where
/// two chunks meet, each one level too dark. Budget one per this many compared values; a
/// wrong propagation rule is orders of magnitude denser than that (the sky decay bug this
/// test was written for was ~320 per 100 000).
const VALUES_PER_ALLOWED_SEAM: u64 = 500_000;

const SECTION_VOLUME: usize = 16 * 16 * 16;
const NIBBLE_BYTES: usize = SECTION_VOLUME / 2;

// -------------------------------------------------------------------------------------------
// Region reading
// -------------------------------------------------------------------------------------------

/// Uncompressed chunk NBT keyed by chunk position, for one region file.
fn read_region(path: &Path) -> Vec<(Vector2<i32>, Vec<u8>)> {
    let Some((region_x, region_z)) = region_coords(path) else {
        return Vec::new();
    };
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    if bytes.len() < 8192 {
        return Vec::new();
    }

    let mut out = Vec::new();
    for index in 0..1024usize {
        let entry = &bytes[index * 4..index * 4 + 4];
        let sector = u32::from_be_bytes([0, entry[0], entry[1], entry[2]]) as usize;
        if sector == 0 || entry[3] == 0 {
            continue;
        }

        let start = sector * 4096;
        if start + 5 > bytes.len() {
            continue;
        }
        let length = u32::from_be_bytes(bytes[start..start + 4].try_into().unwrap()) as usize;
        if length == 0 || start + 4 + length > bytes.len() {
            continue;
        }
        let payload = &bytes[start + 5..start + 4 + length];

        let Some(nbt) = decompress(bytes[start + 4], payload) else {
            continue;
        };
        let pos = Vector2::new(
            region_x * 32 + (index % 32) as i32,
            region_z * 32 + (index / 32) as i32,
        );
        out.push((pos, nbt));
    }
    out
}

fn region_coords(path: &Path) -> Option<(i32, i32)> {
    let name = path.file_name()?.to_str()?;
    let mut parts = name.split('.');
    if parts.next()? != "r" {
        return None;
    }
    let x = parts.next()?.parse().ok()?;
    let z = parts.next()?.parse().ok()?;
    Some((x, z))
}

fn decompress(scheme: u8, payload: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(payload.len() * 4);
    match scheme {
        1 => GzDecoder::new(payload).read_to_end(&mut out).ok()?,
        2 => ZlibDecoder::new(payload).read_to_end(&mut out).ok()?,
        3 => {
            out.extend_from_slice(payload);
            payload.len()
        }
        _ => return None,
    };
    Some(out)
}

/// Same named/unnamed detection as [`ChunkData::internal_from_bytes`].
fn parse_nbt(bytes: &[u8]) -> Option<NbtCompound> {
    let is_named = bytes.len() >= 3 && bytes[0] == 0x0a && bytes[1] == 0x00 && bytes[2] == 0x00;
    let mut cursor = std::io::Cursor::new(bytes);
    let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(&mut cursor);
    let nbt = if is_named {
        pumpkin_nbt::Nbt::read(&mut reader)
    } else {
        pumpkin_nbt::Nbt::read_unnamed(&mut reader)
    }
    .ok()?;
    Some(nbt.root_tag)
}

// -------------------------------------------------------------------------------------------
// What vanilla stored
// -------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum LightKind {
    Sky,
    Block,
}

impl LightKind {
    const fn tag(self) -> &'static str {
        match self {
            Self::Sky => "SkyLight",
            Self::Block => "BlockLight",
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Sky => "sky",
            Self::Block => "block",
        }
    }
}

/// Vanilla omits a section's nibble array when it is uniform, and the array alone does not
/// say whether that means 0 or 15. Those sections carry `None` and are excluded from the
/// comparison instead of guessed at.
struct VanillaLight {
    /// Indexed by section, `min_y_section` first.
    sky: Vec<Option<Box<[u8]>>>,
    block: Vec<Option<Box<[u8]>>>,
    min_y_section: i32,
    lit: bool,
    full: bool,
}

impl VanillaLight {
    fn read(nbt: &NbtCompound, section_count: usize, min_y_section: i32) -> Self {
        let mut sky = vec![None; section_count];
        let mut block = vec![None; section_count];

        if let Some(sections) = nbt.get_list("sections") {
            for section in sections {
                let pumpkin_nbt::tag::NbtTag::Compound(section) = section else {
                    continue;
                };
                let y = i32::from(section.get_byte("Y").unwrap_or(0));
                let Ok(index) = usize::try_from(y - min_y_section) else {
                    continue;
                };
                if index >= section_count {
                    continue;
                }
                sky[index] = nibbles(section, LightKind::Sky);
                block[index] = nibbles(section, LightKind::Block);
            }
        }

        Self {
            sky,
            block,
            min_y_section,
            lit: nbt.get_byte("isLightOn").unwrap_or(0) != 0,
            full: nbt
                .get_string("Status")
                .is_some_and(|s| s == "minecraft:full"),
        }
    }

    const fn layer(&self, kind: LightKind) -> &Vec<Option<Box<[u8]>>> {
        match kind {
            LightKind::Sky => &self.sky,
            LightKind::Block => &self.block,
        }
    }
}

fn nibbles(section: &NbtCompound, kind: LightKind) -> Option<Box<[u8]>> {
    let array = section.get(kind.tag())?.extract_byte_array()?;
    if array.len() != NIBBLE_BYTES {
        return None;
    }
    Some(array.iter().map(|&b| b as u8).collect())
}

#[inline]
const fn nibble_at(data: &[u8], x: usize, y: usize, z: usize) -> u8 {
    let index = y * 256 + z * 16 + x;
    (data[index >> 1] >> (4 * (index & 1))) & 0x0F
}

// -------------------------------------------------------------------------------------------
// Relighting
// -------------------------------------------------------------------------------------------

fn wipe(light: &mut [LightContainer]) {
    for container in light {
        *container = LightContainer::new_empty(0);
    }
}

/// What the eight chunks around the center are while the pass runs.
///
/// Correctness is judged against `Loaded`: their stored light is what lets the rim seed a cell
/// lit from deep inside a neighbour. Cost is reported for `Proto`, which is what the generation
/// pipeline actually hands the pass -- proto chunks answer the per-section skips from a mask
/// they maintain, loaded ones from their block palette, and neither has been lit yet.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Neighbours {
    Loaded,
    Proto,
}

/// Rebuilds `center` as a `ProtoChunk` from its stored blocks and runs the worldgen light
/// pass over it with its eight neighbours in the requested shape.
fn relight(
    chunks: &HashMap<Vector2<i32>, Arc<ChunkData>>,
    center: Vector2<i32>,
    neighbours: Neighbours,
) -> (ProtoChunk, std::time::Duration) {
    // Only the blocks are taken from the chunk, so the generator behind the proto chunk is
    // never asked for terrain.
    let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
        Seed(0),
        Dimension::OVERWORLD,
    )));

    let mut cache = Cache::new(center.x - 1, center.y - 1, 3);
    for dx in 0..3 {
        for dz in 0..3 {
            let pos = Vector2::new(cache.x + dx, cache.z + dz);
            let chunk = &chunks[&pos];
            cache
                .chunks
                .push(if pos == center || neighbours == Neighbours::Proto {
                    let mut proto = ProtoChunk::from_chunk_data(chunk, &world_gen);
                    // A `full` chunk is past the lighting stage and would be skipped.
                    proto.stage = StagedChunkEnum::Features;
                    wipe(&mut proto.light.sky_light);
                    wipe(&mut proto.light.block_light);
                    Chunk::Proto(Box::new(proto))
                } else {
                    Chunk::Level(chunk.clone())
                });
        }
    }

    let started = std::time::Instant::now();
    LightEngine::new().initialize_light(&mut cache, &LightingEngineConfig::Default);
    let elapsed = started.elapsed();

    let Chunk::Proto(proto) = cache.chunks.swap_remove(4) else {
        panic!("the center was pushed as a proto chunk")
    };
    (*proto, elapsed)
}

// -------------------------------------------------------------------------------------------
// Comparison
// -------------------------------------------------------------------------------------------

/// Divergence tally for one light kind. `too_dark` and `too_bright` separate a missing
/// source from an over-propagated one; `edge` separates the outermost columns, which depend
/// on how far the pass reaches into the neighbours.
#[derive(Default)]
struct Tally {
    interior: u64,
    edge: u64,
    too_dark: u64,
    too_bright: u64,
    samples: Vec<String>,
}

impl Tally {
    fn record(&mut self, edge: bool, want: u8, got: u8, detail: impl FnOnce() -> String) {
        if edge {
            self.edge += 1;
        } else {
            self.interior += 1;
        }
        if got < want {
            self.too_dark += 1;
        } else {
            self.too_bright += 1;
        }
        if self.samples.len() < MAX_REPORTED {
            self.samples.push(detail());
        }
    }

    const fn total(&self) -> u64 {
        self.interior + self.edge
    }
}

#[derive(Default)]
struct Report {
    compared: u64,
    skipped_sections: u64,
    sky: Tally,
    block: Tally,
}

impl Report {
    const fn tally(&mut self, kind: LightKind) -> &mut Tally {
        match kind {
            LightKind::Sky => &mut self.sky,
            LightKind::Block => &mut self.block,
        }
    }
}

fn compare(
    kind: LightKind,
    pos: Vector2<i32>,
    proto: &ProtoChunk,
    vanilla: &VanillaLight,
    chunk: &ChunkData,
    report: &mut Report,
) {
    let ours = match kind {
        LightKind::Sky => &proto.light.sky_light,
        LightKind::Block => &proto.light.block_light,
    };

    for (index, expected) in vanilla.layer(kind).iter().enumerate() {
        let (Some(expected), Some(ours)) = (expected.as_deref(), ours.get(index)) else {
            report.skipped_sections += 1;
            continue;
        };
        let section_base_y = (vanilla.min_y_section + index as i32) * 16;

        for y in 0..16usize {
            for z in 0..16usize {
                for x in 0..16usize {
                    let want = nibble_at(expected, x, y, z);
                    let got = ours.get(x, y, z);
                    report.compared += 1;
                    if want == got {
                        continue;
                    }
                    let world_y = section_base_y + y as i32;
                    let edge = x == 0 || x == 15 || z == 0 || z == 15;
                    report.tally(kind).record(edge, want, got, || {
                        let state = chunk
                            .section
                            .get_block_absolute_y(x, world_y, z)
                            .unwrap_or(BlockStateId::AIR);
                        format!(
                            "{} at ({}, {world_y}, {}) in chunk {},{}: vanilla {want}, pumpkin \
                             {got} (block {}, opacity {})",
                            kind.name(),
                            pos.x * 16 + x as i32,
                            pos.y * 16 + z as i32,
                            pos.x,
                            pos.y,
                            state.to_block_id().to_block().name,
                            state.to_state().opacity,
                        )
                    });
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------

/// Whether a chunk's stored sky light contradicts vanilla's own rule, and so cannot be a
/// reference.
///
/// Vanilla holds 15 only in a source column: `ChunkSkyLightSources.findLowestSourceY` ends the
/// column at the first block with `getLightDampening() != 0`, and a horizontal step caps at 14.
/// A stored 15 at or under such a block is light vanilla wrote before the blocks above it were
/// placed -- a tree grown over an already lit column -- which no correct relight reproduces.
///
/// Only `dampening != 0` is tested, not the shape occlusion `isEdgeOccluded` also applies, so
/// the check errs towards keeping a chunk.
fn sky_contradicts_vanillas_own_rule(chunk: &ChunkData, vanilla: &VanillaLight) -> bool {
    let min_y = chunk.section.min_y;
    let top_y = min_y + (chunk.section.count * 16) as i32 - 1;

    for local_x in 0..16 {
        for local_z in 0..16 {
            let mut occluded = false;
            for y in (min_y..=top_y).rev() {
                let state = chunk
                    .section
                    .get_block_absolute_y(local_x, y, local_z)
                    .unwrap_or(BlockStateId::AIR);
                // The occluder's own cell is already out of the source column.
                occluded |= state.to_state().opacity != 0;
                if !occluded {
                    continue;
                }
                let section = ((y - min_y) >> 4) as usize;
                if let Some(data) = vanilla.sky[section].as_ref()
                    && nibble_at(data, local_x, (y & 15) as usize, local_z) == 15
                {
                    return true;
                }
            }
        }
    }
    false
}

fn load_chunks(region_dir: &Path) -> HashMap<Vector2<i32>, (Arc<ChunkData>, VanillaLight)> {
    let mut out = HashMap::new();
    let Ok(entries) = std::fs::read_dir(region_dir) else {
        return out;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "mca") {
            continue;
        }
        for (pos, bytes) in read_region(&path) {
            let (Some(nbt), Ok(chunk)) = (
                parse_nbt(&bytes),
                ChunkData::internal_from_bytes(&bytes, pos),
            ) else {
                continue;
            };
            let light = VanillaLight::read(&nbt, chunk.section.count, chunk.section.min_y / 16);
            if !light.lit || !light.full {
                continue;
            }
            out.insert(pos, (Arc::new(chunk), light));
        }
    }
    out
}

/// The light pass reads a 1 chunk rim, so only centers with all eight neighbours present can be
/// judged. Ordered so a run picks the same chunks every time.
fn eligible_centers(
    loaded: &HashMap<Vector2<i32>, (Arc<ChunkData>, VanillaLight)>,
) -> Vec<Vector2<i32>> {
    let mut centers: Vec<Vector2<i32>> = loaded
        .keys()
        .copied()
        .filter(|pos| {
            (-1..=1).all(|dx| {
                (-1..=1).all(|dz| loaded.contains_key(&Vector2::new(pos.x + dx, pos.y + dz)))
            })
        })
        .collect();
    centers.sort_by_key(|pos| (pos.x, pos.y));
    centers.truncate(
        std::env::var(CENTERS_ENV)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(MAX_CENTERS),
    );
    centers
}

fn save_dir() -> Option<PathBuf> {
    let root = PathBuf::from(std::env::var_os(SAVE_ENV)?);
    let region = root.join("dimensions/minecraft/overworld/region");
    if region.is_dir() {
        return Some(region);
    }
    // Pre-1.21 layout, and the shape a server writes.
    let region = root.join("region");
    region.is_dir().then_some(region)
}

#[test]
fn pumpkin_reproduces_the_light_vanilla_stored() {
    let Some(region_dir) = save_dir() else {
        eprintln!("{SAVE_ENV} not set or has no overworld region directory, skipping");
        return;
    };

    let loaded = load_chunks(&region_dir);
    assert!(
        !loaded.is_empty(),
        "no lit, fully generated chunks found under {}",
        region_dir.display()
    );

    let centers = eligible_centers(&loaded);
    assert!(
        !centers.is_empty(),
        "no chunk in {} has all eight neighbours saved",
        region_dir.display()
    );

    let blocks: HashMap<_, _> = loaded
        .iter()
        .map(|(pos, (chunk, _))| (*pos, chunk.clone()))
        .collect();

    let mut report = Report::default();
    let mut relight_time = std::time::Duration::ZERO;
    let mut worldgen_time = std::time::Duration::ZERO;
    let mut compared_chunks = 0u32;
    let mut stale_chunks = 0usize;
    for center in &centers {
        let (chunk, vanilla) = &loaded[center];
        if sky_contradicts_vanillas_own_rule(chunk, vanilla) {
            stale_chunks += 1;
            continue;
        }

        let (proto, elapsed) = relight(&blocks, *center, Neighbours::Loaded);
        relight_time += elapsed;
        // Timed separately, because judging the result needs the neighbours' stored light.
        worldgen_time += relight(&blocks, *center, Neighbours::Proto).1;
        compared_chunks += 1;
        compare(LightKind::Sky, *center, &proto, vanilla, chunk, &mut report);
        compare(
            LightKind::Block,
            *center,
            &proto,
            vanilla,
            chunk,
            &mut report,
        );
    }
    assert!(
        compared_chunks > 0,
        "every candidate chunk stores sky light vanilla's own rule cannot produce"
    );

    println!(
        "compared {} light values across {compared_chunks} chunks ({} uniform sections vanilla did not store, {stale_chunks} of {} chunks skipped as stale)",
        report.compared,
        report.skipped_sections,
        centers.len(),
    );
    let chunks_timed = compared_chunks;
    println!(
        "light pass, proto neighbours (what worldgen runs): {:?} per chunk",
        worldgen_time / chunks_timed,
    );
    println!(
        "light pass, loaded neighbours (what this comparison runs): {:?} per chunk",
        relight_time / chunks_timed,
    );

    let diverged = report.sky.total() + report.block.total();
    let budget = report.compared / VALUES_PER_ALLOWED_SEAM;
    if diverged > 0 {
        use std::fmt::Write;

        let mut message = String::new();
        let _ = writeln!(
            message,
            "{diverged} of {} compared values diverge (budget {budget})",
            report.compared
        );
        for (kind, tally) in [
            (LightKind::Sky, &report.sky),
            (LightKind::Block, &report.block),
        ] {
            let _ = writeln!(
                message,
                "  {}: {} total ({} interior, {} edge; {} too dark, {} too bright)",
                kind.name(),
                tally.total(),
                tally.interior,
                tally.edge,
                tally.too_dark,
                tally.too_bright,
            );
            for line in &tally.samples {
                message.push_str("    ");
                message.push_str(line);
                message.push('\n');
            }
        }
        assert!(diverged <= budget, "{message}");
        println!("{message}");
    }
}

/// TEMPORARY -- census: how many light sections carry a full nibble array that holds
/// only one repeated value.
#[test]
fn uniform_sections_do_not_hold_a_nibble_array() {
    let Some(region_dir) = save_dir() else {
        eprintln!("{SAVE_ENV} not set or has no overworld region directory, skipping");
        return;
    };

    let loaded = load_chunks(&region_dir);
    let centers = eligible_centers(&loaded);
    let chunks: HashMap<_, _> = loaded.iter().map(|(p, (c, _))| (*p, c.clone())).collect();

    let mut counts = [[0u64; 3]; 2];
    for center in &centers {
        let (proto, _) = relight(&chunks, *center, Neighbours::Proto);
        for (layer, containers) in [proto.light.sky_light, proto.light.block_light]
            .into_iter()
            .enumerate()
        {
            for container in containers {
                let slot = match &container {
                    LightContainer::Empty(_) => 0,
                    LightContainer::Full(data) => {
                        if data.iter().all(|&b| b == data[0] && b >> 4 == b & 0x0F) {
                            1
                        } else {
                            2
                        }
                    }
                };
                counts[layer][slot] += 1;
            }
        }
    }

    let n = centers.len() as f64;
    for (layer, name) in [(0, "sky"), (1, "block")] {
        println!(
            "{name}: {:.1} uniform-empty, {:.1} uniform-Full ({:.1} KiB wasted), {:.1} varied  per chunk",
            counts[layer][0] as f64 / n,
            counts[layer][1] as f64 / n,
            counts[layer][1] as f64 / n * 2.0,
            counts[layer][2] as f64 / n,
        );
    }
}

/// TEMPORARY -- dumps one column so an underwater divergence can be read off directly:
/// blocks, opacity, what vanilla stored and what the pass produced.
///
/// `PUMPKIN_DUMP_COLUMN="-261,16"` picks the column; the chunk containing it must be saved.
#[test]
fn dump_a_column() {
    let (Some(region_dir), Some(spec)) = (save_dir(), std::env::var_os("PUMPKIN_DUMP_COLUMN"))
    else {
        eprintln!("PUMPKIN_DUMP_COLUMN not set, skipping");
        return;
    };
    let spec = spec.to_string_lossy().to_string();
    let (bx, bz) = spec.split_once(',').expect("expected \"x,z\"");
    let (bx, bz): (i32, i32) = (bx.trim().parse().unwrap(), bz.trim().parse().unwrap());
    let center = Vector2::new(bx >> 4, bz >> 4);

    let loaded = load_chunks(&region_dir);
    let chunks: HashMap<_, _> = loaded.iter().map(|(p, (c, _))| (*p, c.clone())).collect();
    assert!(
        chunks.contains_key(&center),
        "chunk {center:?} is not saved"
    );

    let (proto, _) = relight(&chunks, center, Neighbours::Loaded);
    let (chunk, vanilla) = &loaded[&center];
    let min_y_section = chunk.section.min_y / 16;
    let (lx, lz) = ((bx & 15) as usize, (bz & 15) as usize);

    println!("column x={bx} z={bz} in chunk {center:?}  (local {lx},{lz})");
    println!("   y | block                         op | vanilla | pumpkin");
    for y in (30..=100).rev() {
        let section = (y >> 4) - min_y_section;
        let Ok(section) = usize::try_from(section) else {
            continue;
        };
        let state = chunk
            .section
            .get_block_absolute_y(lx, y, lz)
            .unwrap_or(BlockStateId::AIR);
        let block = pumpkin_data::Block::from_state_id(state);
        let opacity = pumpkin_data::BlockState::from_id(state).opacity;

        let van = vanilla.layer(LightKind::Sky)[section]
            .as_ref()
            .map(|data| nibble_at(data, lx, (y & 15) as usize, lz));
        let ours = proto.light.sky_light[section].get(lx, (y & 15) as usize, lz);

        let flag = match van {
            Some(v) if v != ours => "  <-- diverges",
            Some(_) => "",
            None => "  (section not stored)",
        };
        println!(
            "{y:>5} | {:<28} {opacity:>2} | {:>7} | {ours:>7}{flag}",
            block.name,
            van.map_or("-".to_string(), |v| v.to_string()),
        );
    }
}

/// TEMPORARY -- a horizontal slice of one chunk at one Y, vanilla against ours, so a
/// horizontal propagation failure can be located.
///
/// `PUMPKIN_DUMP_SLICE="-261,54,16"`
#[test]
fn dump_a_slice() {
    let (Some(region_dir), Some(spec)) = (save_dir(), std::env::var_os("PUMPKIN_DUMP_SLICE"))
    else {
        eprintln!("PUMPKIN_DUMP_SLICE not set, skipping");
        return;
    };
    let spec = spec.to_string_lossy().to_string();
    let parts: Vec<i32> = spec.split(',').map(|p| p.trim().parse().unwrap()).collect();
    let (bx, by, bz) = (parts[0], parts[1], parts[2]);
    let center = Vector2::new(bx >> 4, bz >> 4);

    let loaded = load_chunks(&region_dir);
    let chunks: HashMap<_, _> = loaded.iter().map(|(p, (c, _))| (*p, c.clone())).collect();
    let (proto, _) = relight(&chunks, center, Neighbours::Loaded);
    let (chunk, vanilla) = &loaded[&center];
    let section = ((by >> 4) - chunk.section.min_y / 16) as usize;
    let ly = (by & 15) as usize;

    for (title, pick) in [("vanilla", true), ("pumpkin", false)] {
        println!("\n{title} sky at y={by}, chunk {center:?}   (rows = local z, cols = local x)");
        print!("     ");
        for lx in 0..16 {
            print!("{lx:>3}");
        }
        println!();
        for lz in 0..16usize {
            print!("z{lz:>3} ");
            for lx in 0..16usize {
                let value = if pick {
                    vanilla.layer(LightKind::Sky)[section]
                        .as_ref()
                        .map_or(-1i32, |d| i32::from(nibble_at(d, lx, ly, lz)))
                } else {
                    i32::from(proto.light.sky_light[section].get(lx, ly, lz))
                };
                let solid = pumpkin_data::BlockState::from_id(
                    chunk
                        .section
                        .get_block_absolute_y(lx, by, lz)
                        .unwrap_or(BlockStateId::AIR),
                )
                .opacity
                    >= 15;
                if solid {
                    print!("  #");
                } else {
                    print!("{value:>3}");
                }
            }
            println!();
        }
    }
    println!("\n(# = opaque block, numbers = sky light; target was x={bx} z={bz})");
}
