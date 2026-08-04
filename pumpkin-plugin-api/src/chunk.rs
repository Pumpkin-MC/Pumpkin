use crate::world;

/// A block entity stored as raw NBT bytes together with its absolute position.
pub struct BlockEntityData {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub nbt_data: Vec<u8>,
}

impl From<world::SerializedBlockEntity> for BlockEntityData {
    fn from(be: world::SerializedBlockEntity) -> Self {
        Self {
            x: be.x,
            y: be.y,
            z: be.z,
            nbt_data: be.nbt_data,
        }
    }
}

impl From<BlockEntityData> for world::SerializedBlockEntity {
    fn from(be: BlockEntityData) -> Self {
        Self {
            x: be.x,
            y: be.y,
            z: be.z,
            nbt_data: be.nbt_data,
        }
    }
}

/// A full chunk's data stored as flat arrays in WASM memory.
///
/// Blocks are indexed as `[x][y][z]` where `x` and `z` are in `0..16` and
/// `y` goes from `0` (absolute `min_y`) to `height - 1` (exclusive).
/// The flat index is `x * (height * 16) + y * 16 + z`.
///
/// Biomes use quarter-scale coordinates: `x` and `z` in `0..4`, `y` in
/// `0..(height/4)`.  The flat index is `x * (biome_height * 4) + y * 4 + z`.
pub struct ChunkData {
    pub x: i32,
    pub z: i32,
    pub min_y: i32,
    pub height: i32,
    blocks: Vec<u16>,
    biomes: Vec<u8>,
    pub block_entities: Vec<BlockEntityData>,
}

impl ChunkData {
    /// Wrap an existing [`world::SerializedChunk`] received from the host.
    pub fn from_serialized(chunk: world::SerializedChunk) -> Self {
        let block_entities = chunk
            .block_entities
            .into_iter()
            .map(BlockEntityData::from)
            .collect();

        Self {
            x: chunk.x,
            z: chunk.z,
            min_y: chunk.min_y,
            height: chunk.height,
            blocks: chunk.blocks,
            biomes: chunk.biomes,
            block_entities,
        }
    }

    /// Convert this chunk back into the WIT type so it can be sent to the host.
    pub fn into_serialized(self) -> world::SerializedChunk {
        world::SerializedChunk {
            x: self.x,
            z: self.z,
            min_y: self.min_y,
            height: self.height,
            blocks: self.blocks,
            biomes: self.biomes,
            block_entities: self
                .block_entities
                .into_iter()
                .map(world::SerializedBlockEntity::from)
                .collect(),
        }
    }

    /// Create an empty chunk at the given coordinates.
    ///
    /// All blocks are initialised to `default_block` (usually `0` = air) and
    /// all biomes to `default_biome`.
    pub fn new_empty(
        x: i32,
        z: i32,
        min_y: i32,
        height: i32,
        default_block: u16,
        default_biome: u8,
    ) -> Self {
        let block_count = (16 * height * 16) as usize;
        let biome_height = (height as usize) / 4;
        let biome_count = 4 * biome_height * 4;

        Self {
            x,
            z,
            min_y,
            height,
            blocks: vec![default_block; block_count],
            biomes: vec![default_biome; biome_count],
            block_entities: Vec::new(),
        }
    }

    // ── block access ──────────────────────────────────────────────

    #[inline]
    fn block_index(&self, x: i32, y: i32, z: i32) -> usize {
        (x as usize) * (self.height as usize * 16) + (y as usize) * 16 + (z as usize)
    }

    /// Get the block state ID at the given local coordinates.
    ///
    /// `x` and `z` must be in `0..16`. `y` is relative to `min_y` (i.e.
    /// `0` is the bottom of the chunk, `height - 1` is the top).
    #[inline]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> u16 {
        self.blocks[self.block_index(x, y, z)]
    }

    /// Set the block state ID at the given local coordinates.
    #[inline]
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, id: u16) {
        let idx = self.block_index(x, y, z);
        self.blocks[idx] = id;
    }

    /// Fill every block in the chunk with the given state ID.
    pub fn fill_blocks(&mut self, id: u16) {
        self.blocks.fill(id);
    }

    // ── biome access ──────────────────────────────────────────────

    #[inline]
    fn biome_index(&self, x: i32, y: i32, z: i32) -> usize {
        let biome_height = (self.height as usize) / 4;
        (x as usize) * (biome_height * 4) + (y as usize) * 4 + (z as usize)
    }

    /// Get the biome ID at the given local biome coordinates.
    ///
    /// `x` and `z` must be in `0..4`. `y` is relative to `min_y >> 2`
    /// (i.e. `0` is the bottom biome section, `height/4 - 1` is the top).
    #[inline]
    pub fn get_biome(&self, x: i32, y: i32, z: i32) -> u8 {
        self.biomes[self.biome_index(x, y, z)]
    }

    /// Set the biome ID at the given local biome coordinates.
    #[inline]
    pub fn set_biome(&mut self, x: i32, y: i32, z: i32, id: u8) {
        let idx = self.biome_index(x, y, z);
        self.biomes[idx] = id;
    }

    /// Fill every biome quad in the chunk with the given biome ID.
    pub fn fill_biomes(&mut self, id: u8) {
        self.biomes.fill(id);
    }

    // ── block entity access ───────────────────────────────────────

    /// Add a block entity to this chunk.
    ///
    /// Does **not** deduplicate by position – if a block entity with the
    /// same coordinates already exists, both will be present.  The host
    /// will use the last one when building the chunk.
    pub fn add_block_entity(&mut self, entity: BlockEntityData) {
        self.block_entities.push(entity);
    }

    /// Remove all block entities at the given position.
    pub fn remove_block_entities(&mut self, x: i32, y: i32, z: i32) {
        self.block_entities
            .retain(|be| be.x != x || be.y != y || be.z != z);
    }

    /// Clear all block entities from this chunk.
    pub fn clear_block_entities(&mut self) {
        self.block_entities.clear();
    }
}

impl From<world::SerializedChunk> for ChunkData {
    fn from(chunk: world::SerializedChunk) -> Self {
        Self::from_serialized(chunk)
    }
}

impl From<ChunkData> for world::SerializedChunk {
    fn from(chunk: ChunkData) -> Self {
        chunk.into_serialized()
    }
}
