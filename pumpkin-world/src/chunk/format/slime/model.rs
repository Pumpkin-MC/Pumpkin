//! In-memory representation of a parsed `.slime` world (AdvancedSlimePaper v13).
//!
//! The reader ([`super::read_slime_world`]) fills this model with the raw,
//! still-NBT-encoded payloads exactly as they appear on disk. Interpreting the
//! NBT (palette conversion, block entities, entities, …) is left to the
//! converter layer, which keeps the binary container parsing isolated and easy
//! to test.

use bytes::Bytes;

/// A whole `.slime` world held in memory.
///
/// SlimeWorld stores an entire (small) world in a single file; this is the
/// decoded form of that file before it is mapped onto Pumpkin's chunk
/// representation.
#[derive(Debug)]
pub struct SlimeWorld {
    /// Vanilla `DataVersion` the world was saved with.
    pub world_version: i32,
    /// `additionalWorldData` bitmask (POI / block ticks / fluid ticks present).
    pub flags: u8,
    /// All stored chunks, in file order.
    pub chunks: Vec<SlimeChunk>,
    /// Decompressed world "extra" compound (named NBT, includes `properties`);
    /// empty when absent.
    pub extra: Bytes,
}

/// A single chunk within a [`SlimeWorld`].
///
/// Every NBT payload is kept as its raw on-disk bytes (a named root compound,
/// or empty when the segment length was zero).
#[derive(Debug)]
pub struct SlimeChunk {
    /// Chunk X coordinate.
    pub x: i32,
    /// Chunk Z coordinate.
    pub z: i32,
    /// Sections ordered from the world bottom upwards; the absolute section Y is
    /// implicit and must be derived from the target dimension's minimum section.
    pub sections: Vec<SlimeSection>,
    /// `Heightmaps` compound.
    pub height_maps: Bytes,
    /// Compound `{ tileEntities: [ … ] }` (block entities).
    pub block_entities: Bytes,
    /// Compound `{ entities: [ … ] }`.
    pub entities: Bytes,
    /// Compound `{ block_ticks: [ … ] }`, only when the world saves block ticks.
    pub block_ticks: Bytes,
    /// Compound `{ fluid_ticks: [ … ] }`, only when the world saves fluid ticks.
    pub fluid_ticks: Bytes,
    /// POI compound, only when the world saves POI data; not consumed yet.
    pub poi: Bytes,
}

/// A 16×16×16 chunk section within a [`SlimeChunk`].
#[derive(Debug)]
pub struct SlimeSection {
    /// Block light nibbles (2048 bytes) if present.
    pub block_light: Option<Bytes>,
    /// Sky light nibbles (2048 bytes) if present.
    pub sky_light: Option<Bytes>,
    /// Vanilla paletted `block_states` compound (`{ palette, data }`); empty when absent.
    pub block_states: Bytes,
    /// Vanilla paletted `biomes` compound; empty when absent.
    pub biomes: Bytes,
}
