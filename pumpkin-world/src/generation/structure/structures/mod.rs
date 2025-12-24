use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_util::math::{block_box::BlockBox, position::BlockPos, vector3::Vector3};

use crate::{
    ProtoChunk,
    generation::{height_limit::HeightLimitView, structure::StructureType},
};

pub mod buried_treasure;
pub mod nether_fortress;
pub mod swamp_hut;

#[expect(clippy::too_many_arguments)]
pub fn fill_with_outline(
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
    outline: &BlockState,
    inside: &BlockState,
    chunk: &mut ProtoChunk,
) {
    let chunk_x = chunk.x;
    let chunk_z = chunk.z;

    let chunk_min_x = chunk_x * 16;
    let chunk_max_x = chunk_min_x + 15;
    let chunk_min_z = chunk_z * 16;
    let chunk_max_z = chunk_min_z + 15;

    let start_x = min_x.max(chunk_min_x);
    let end_x = max_x.min(chunk_max_x);
    let start_z = min_z.max(chunk_min_z);
    let end_z = max_z.min(chunk_max_z);

    if start_x > end_x || start_z > end_z {
        return;
    }

    for y in min_y..=max_y {
        for x in start_x..=end_x {
            for z in start_z..=end_z {
                let is_border = x == min_x
                    || x == max_x
                    || y == min_y
                    || y == max_y
                    || z == min_z
                    || z == max_z;

                let block = if is_border { outline } else { inside };
                chunk.set_block_state(&Vector3::new(x, y, z), block);
            }
        }
    }
}

/// Fills a solid cuboid.
#[expect(clippy::too_many_arguments)]
pub fn fill(
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
    state: &BlockState,
    chunk: &mut ProtoChunk,
) {
    // Basic bounds check to avoid iterating 1000s of blocks outside the chunk
    let chunk_x = chunk.x;
    let chunk_z = chunk.z;
    let chunk_min_x = chunk_x * 16;
    let chunk_max_x = chunk_min_x + 15;
    let chunk_min_z = chunk_z * 16;
    let chunk_max_z = chunk_min_z + 15;

    let start_x = min_x.max(chunk_min_x);
    let end_x = max_x.min(chunk_max_x);
    let start_z = min_z.max(chunk_min_z);
    let end_z = max_z.min(chunk_max_z);

    if start_x > end_x || start_z > end_z {
        return;
    }

    for y in min_y..=max_y {
        for x in start_x..=end_x {
            for z in start_z..=end_z {
                chunk.set_block_state(&Vector3::new(x, y, z), state);
            }
        }
    }
}

/// Fills downwards from a point until the bottom of the world.
pub fn fill_downwards(x: i32, y: i32, z: i32, state: &BlockState, chunk: &mut ProtoChunk) {
    let start_y = y;
    let end_y = chunk.bottom_y() as i32; // e.g. -64

    // FIX: Use .rev() because ranges like 60..-64 are empty.
    // Also, usually you want to stop if you hit a solid block (optional, depends on need).
    for current_y in (end_y..=start_y).rev() {
        chunk.set_block_state(&Vector3::new(x, current_y, z), state);
    }
}

/// Represents a single component of a structure (e.g., a room, a bridge).
pub trait StructurePiece: Send + Sync {
    /// The bounding box of this specific piece.
    fn bounding_box(&self) -> &BlockBox;

    /// Places the blocks for this piece into the chunk.
    fn place(&self, chunk: &mut ProtoChunk, seed: i64);
}

/// Holds all the pieces that make up a generated structure instance.
#[derive(Default)]
pub struct StructurePiecesCollector {
    pub pieces: Vec<Box<dyn StructurePiece>>,
}

impl StructurePiecesCollector {
    pub fn new() -> Self {
        Self { pieces: Vec::new() }
    }

    pub fn add_piece(&mut self, piece: Box<dyn StructurePiece>) {
        self.pieces.push(piece);
    }

    /// Iterates over all pieces and generates them if they intersect the current chunk.
    pub fn generate_in_chunk(&self, chunk: &mut ProtoChunk, seed: i64) {
        let chunk_x = chunk.x;
        let chunk_z = chunk.z;
        let chunk_box = BlockBox::new(
            chunk_x * 16,
            chunk.bottom_y() as i32,
            chunk_z * 16,
            (chunk_x * 16) + 15,
            chunk.top_y() as i32,
            (chunk_z * 16) + 15,
        );

        for piece in &self.pieces {
            if piece.bounding_box().intersects(&chunk_box) {
                piece.place(chunk, seed);
            }
        }
    }
}

#[derive(Clone)]
pub struct StructurePosition {
    pub start_pos: BlockPos,
    pub collector: Arc<StructurePiecesCollector>,
}

pub trait StructureGenerator {
    fn try_generate(&self, seed: i64, chunk_x: i32, chunk_z: i32) -> Option<StructurePosition>;
}

#[derive(Clone)]
pub enum StructureInstance {
    /// This chunk is the "owner" of the structure.
    Start(StructurePosition, StructureType),
    /// This chunk just contains a piece of a structure starting elsewhere.
    /// Stores the BlockPos of the 'Start' so you can look it up.
    Reference(BlockPos),
}
