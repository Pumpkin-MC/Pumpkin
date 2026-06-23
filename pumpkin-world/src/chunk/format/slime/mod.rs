//! SlimeWorld (`.slime`) world format support — AdvancedSlimePaper v13.
//!
//! SlimeWorld packs an entire (typically small) world into a single
//! zstd-compressed file that is loaded wholesale into memory. This module
//! provides a faithful parser for that container ([`read_slime_world`]); the
//! decoded [`SlimeWorld`] is later mapped onto Pumpkin's native chunk
//! representation.
//!
//! The on-disk layout mirrors AdvancedSlimePaper `dev/26.2`
//! (`SlimeSerializer` / `v13SlimeWorldDeSerializer`).

mod convert;
mod io;
mod model;
mod reader;

pub(crate) use convert::{chunk_to_chunk_data, chunk_to_entity_data};
pub(crate) use io::{SlimeChunkIo, SlimeEntityIo};
pub use model::{SlimeChunk, SlimeSection, SlimeWorld};
pub use reader::read_slime_world;

/// First two bytes of every SlimeWorld file (`SlimeFormat.SLIME_HEADER`).
pub const SLIME_MAGIC: [u8; 2] = [0xB1, 0x0B];
/// The only SlimeWorld format version supported (AdvancedSlimePaper `dev/26.2`).
pub const SLIME_VERSION: u8 = 13;

/// Length in bytes of a section light nibble array (16×16×16 / 2).
const LIGHT_ARRAY_SIZE: usize = 16 * 16 * 16 / 2;

// `additionalWorldData` flag bits (`v13AdditionalWorldData` ordinals).
const FLAG_POI: u8 = 1;
const FLAG_BLOCK_TICKS: u8 = 1 << 1;
const FLAG_FLUID_TICKS: u8 = 1 << 2;
const KNOWN_FLAGS: u8 = FLAG_POI | FLAG_BLOCK_TICKS | FLAG_FLUID_TICKS;

/// Errors that can occur while reading a `.slime` file.
#[derive(Debug, thiserror::Error)]
pub enum SlimeError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a slime file (invalid magic)")]
    BadMagic,
    #[error("unsupported slime format version {0} (only version 13 is supported)")]
    UnsupportedVersion(u8),
    #[error("unexpected end of slime data")]
    Truncated,
    #[error("corrupt slime data: {0}")]
    Corrupt(&'static str),
    #[error("zstd decompression of slime data failed")]
    Decompress,
}
