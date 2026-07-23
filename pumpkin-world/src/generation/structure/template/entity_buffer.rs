//! Cross-pipeline buffer for structure-template entities.
//!
//! Structure placement runs on the block-generation thread; entity chunks are
//! spawned on a separate path. Entities are stored here keyed by chunk so either
//! path can pick them up without racing.

use dashmap::DashMap;
use pumpkin_nbt::compound::NbtCompound;
use std::sync::LazyLock;

static STRUCTURE_ENTITY_BUFFER: LazyLock<DashMap<(i32, i32), Vec<NbtCompound>>> =
    LazyLock::new(DashMap::new);

/// Queue a structure-template entity for later spawn in chunk (`chunk_x`, `chunk_z`).
pub fn push(chunk_x: i32, chunk_z: i32, nbt: NbtCompound) {
    STRUCTURE_ENTITY_BUFFER
        .entry((chunk_x, chunk_z))
        .or_default()
        .push(nbt);
}

/// Drain all buffered structure entities for a chunk.
#[must_use]
pub fn take(chunk_x: i32, chunk_z: i32) -> Vec<NbtCompound> {
    STRUCTURE_ENTITY_BUFFER
        .remove(&(chunk_x, chunk_z))
        .map(|(_, v)| v)
        .unwrap_or_default()
}
