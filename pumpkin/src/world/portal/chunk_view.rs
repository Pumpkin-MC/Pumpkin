//! Chunk snapshot cache for portal searches.
//!
//! Vanilla reads the destination dimension through `ServerLevel.getBlockState`,
//! which pulls the chunk in on demand (`PortalForcer.java:62-76`). Pumpkin's
//! sync `World::get_block*` helpers instead return air for any chunk that is not
//! already resident, so a portal search running against a dimension the player
//! has not loaded yet sees nothing but air. That is what made every traversal
//! fall through to "build a new portal".
//!
//! `ChunkView` restores vanilla's semantics: it fetches each chunk once, keeps
//! the `Arc<ChunkData>` snapshot, and answers subsequent block queries from
//! memory. A portal search touches only a handful of chunks, so caching them
//! avoids re-fetching the same chunk for every block in a column.

use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use pumpkin_world::{chunk::ChunkHeightmapType, level::SyncChunk};
use rustc_hash::FxHashMap;

use crate::world::World;

/// Caches chunk snapshots for the duration of one portal search.
pub struct ChunkView<'a> {
    world: &'a World,
    chunks: FxHashMap<Vector2<i32>, SyncChunk>,
}

impl<'a> ChunkView<'a> {
    #[must_use]
    pub fn new(world: &'a World) -> Self {
        Self {
            world,
            chunks: FxHashMap::default(),
        }
    }

    /// Fetches a chunk snapshot, loading or generating it if needed.
    ///
    /// No lock is held across the await: the chunk is fetched first, then
    /// inserted into the local map.
    async fn chunk(&mut self, chunk_pos: Vector2<i32>) -> SyncChunk {
        if let Some(chunk) = self.chunks.get(&chunk_pos) {
            return chunk.clone();
        }
        let chunk = self
            .world
            .level
            .get_or_fetch_chunk(chunk_pos, Clone::clone)
            .await;
        self.chunks.insert(chunk_pos, chunk.clone());
        chunk
    }

    /// Vanilla `LevelReader.getBlockState`, honouring the dimension height limit.
    pub async fn state_id(&mut self, pos: &BlockPos) -> BlockStateId {
        if !self.world.is_in_build_limit(*pos) {
            return Block::AIR.default_state.id;
        }
        let (chunk_pos, relative) = pos.chunk_and_chunk_relative_position();
        let chunk = self.chunk(chunk_pos).await;
        chunk
            .section
            .get_block_absolute_y(relative.x as usize, relative.y, relative.z as usize)
            .unwrap_or(Block::AIR.default_state.id)
    }

    /// Synchronous read that only consults already-cached chunks.
    ///
    /// Returns `None` when the containing chunk has not been fetched yet, which
    /// lets callers drive vanilla's synchronous predicate-based helpers (such as
    /// the largest-rectangle scan) after prefetching the region they touch.
    #[must_use]
    pub fn cached_state_id(&self, pos: &BlockPos) -> Option<BlockStateId> {
        if !self.world.is_in_build_limit(*pos) {
            return None;
        }
        let (chunk_pos, relative) = pos.chunk_and_chunk_relative_position();
        self.chunks.get(&chunk_pos).and_then(|chunk| {
            chunk
                .section
                .get_block_absolute_y(relative.x as usize, relative.y, relative.z as usize)
        })
    }

    pub async fn block(&mut self, pos: &BlockPos) -> &'static Block {
        let id = self.state_id(pos).await;
        Block::from_state_id(id)
    }

    pub async fn state(&mut self, pos: &BlockPos) -> &'static BlockState {
        BlockState::from_id(self.state_id(pos).await)
    }

    /// Vanilla `LevelReader.getHeight(Heightmap.Types, x, z)`
    /// (`PortalForcer.java:62`).
    pub async fn heightmap_height(&mut self, heightmap: ChunkHeightmapType, x: i32, z: i32) -> i32 {
        let chunk = self.chunk(Vector2::new(x >> 4, z >> 4)).await;
        let min_y = self.world.min_y;
        // std Mutex: the guard is dropped inside this expression, so it is never
        // held across an await point.
        chunk
            .heightmap
            .lock()
            .map_or(min_y, |map| map.get(heightmap, x, z, min_y))
    }
}
