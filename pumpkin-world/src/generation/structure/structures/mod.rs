use std::sync::{Arc, Mutex};

use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_util::HeightMap;
use pumpkin_util::{
    BlockDirection,
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl, get_carver_seed, xoroshiro128::Xoroshiro},
};

use crate::generation::structure::structures::stronghold::PieceWeight;
use crate::generation::structure::structures::stronghold::StrongholdPieceType;
use crate::{
    ProtoChunk,
    generation::{
        height_limit::HeightLimitView,
        positions::chunk_pos::{start_block_x, start_block_z},
        structure::piece::StructurePieceType,
    },
};

pub mod buried_treasure;
pub mod nether_fortress;
pub mod stronghold;
pub mod swamp_hut;

pub trait BlockRandomizer {
    fn get_block(&self, rng: &mut RandomGenerator, is_border: bool) -> &BlockState;
}

/// Represents a single component of a structure (e.g., a room, a bridge).
pub trait StructurePieceBase: Send + Sync {
    fn get_structure_piece(&self) -> &StructurePiece;

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece;

    fn bounding_box(&self) -> BlockBox {
        self.get_structure_piece().bounding_box
    }

    fn translate(&mut self, x: i32, y: i32, z: i32) {
        self.get_structure_piece_mut().translate(x, y, z);
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase>;

    /// Places the blocks for this piece into the chunk.
    fn place(&mut self, chunk: &mut ProtoChunk, random: &mut RandomGenerator, seed: i64);

    #[expect(clippy::too_many_arguments)]
    fn fill_openings(
        &self,
        _start: &StructurePiece,
        _random: &mut RandomGenerator,
        // TODO: this is only for Stronghold and should not be here
        _weights: &mut Vec<PieceWeight>,
        _last_piece_type: &mut Option<StrongholdPieceType>,
        _has_portal_room: &mut bool,

        _collector: &mut StructurePiecesCollector,
        _pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
    ) {
    }
}

#[derive(Clone)]
pub struct StructurePiece {
    pub r#type: StructurePieceType,
    pub bounding_box: BlockBox,
    pub facing: Option<BlockDirection>,
    pub chain_length: u32,
}

impl StructurePiece {
    #[must_use]
    pub const fn new(
        r#type: StructurePieceType,
        bounding_box: BlockBox,
        chain_length: u32,
    ) -> Self {
        Self {
            r#type,
            bounding_box,
            facing: None,
            chain_length,
        }
    }

    pub const fn set_facing(&mut self, facing: Option<BlockDirection>) {
        self.facing = facing;
    }

    const fn offset_pos(&self, x: i32, y: i32, z: i32) -> Vector3<i32> {
        Vector3::new(
            self.apply_x_transform(x, z),
            self.apply_y_transform(y),
            self.apply_z_transform(x, z),
        )
    }

    const fn apply_x_transform(&self, x: i32, z: i32) -> i32 {
        match self.facing {
            None => x,
            Some(BlockDirection::North | BlockDirection::South) => self.bounding_box.min.x + x,
            Some(BlockDirection::West) => self.bounding_box.max.x - z,
            Some(BlockDirection::East) => self.bounding_box.min.x + z,
            _ => x,
        }
    }

    const fn apply_y_transform(&self, y: i32) -> i32 {
        match self.facing {
            None => y,
            Some(_) => y + self.bounding_box.min.y,
        }
    }

    const fn apply_z_transform(&self, x: i32, z: i32) -> i32 {
        match self.facing {
            None => z,
            Some(BlockDirection::North) => self.bounding_box.max.z - z,
            Some(BlockDirection::South) => self.bounding_box.min.z + z,
            Some(BlockDirection::West | BlockDirection::East) => self.bounding_box.min.z + x,
            _ => z,
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn fill_outline_random(
        &self,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        randomizer: &impl BlockRandomizer,
        chunk: &mut ProtoChunk,
        cant_replace_air: bool,
        rng: &mut RandomGenerator,
        box_limit: &BlockBox,
    ) {
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    if cant_replace_air && self.get_block_at(chunk, x, y, z, box_limit).is_air() {
                        continue;
                    }
                    let is_border = x == min_x
                        || x == max_x
                        || y == min_y
                        || y == max_y
                        || z == min_z
                        || z == max_z;
                    let state = randomizer.get_block(rng, is_border);
                    self.add_block(chunk, state, x, y, z, box_limit);
                }
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn fill_with_outline(
        &self,
        chunk: &mut ProtoChunk,
        box_limit: &BlockBox,
        cant_replace_air: bool,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        outline: &BlockState,
        inside: &BlockState,
    ) {
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    if cant_replace_air && self.get_block_at(chunk, x, y, z, box_limit).is_air() {
                        continue;
                    }
                    let is_border = x == min_x
                        || x == max_x
                        || y == min_y
                        || y == max_y
                        || z == min_z
                        || z == max_z;

                    let block = if is_border { outline } else { inside };
                    self.add_block(chunk, block, x, y, z, box_limit);
                }
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn fill_with_outline_under_sea_level(
        &self,
        chunk: &mut ProtoChunk,
        box_limit: &BlockBox,
        rng: &mut RandomGenerator,
        block_chance: f32,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        outline: &BlockState,
        inside: &BlockState,
        cant_replace_air: bool,
        stay_below_sea_level: bool,
    ) {
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    // 1. Random Threshold Check
                    if rng.next_f32() > block_chance {
                        continue;
                    }

                    // 2. Air Replacement Check
                    if cant_replace_air && self.get_block_at(chunk, x, y, z, box_limit).is_air() {
                        continue;
                    }

                    if stay_below_sea_level && !self.is_under_sea_level(chunk, x, y, z, box_limit) {
                        continue;
                    }

                    let is_border = x == min_x
                        || x == max_x
                        || y == min_y
                        || y == max_y
                        || z == min_z
                        || z == max_z;

                    let state = if is_border { outline } else { inside };
                    self.add_block(chunk, state, x, y, z, box_limit);
                }
            }
        }
    }

    /// Fills a solid cuboid.
    #[expect(clippy::too_many_arguments)]
    pub fn fill(
        &self,
        chunk: &mut ProtoChunk,
        box_limit: &BlockBox,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        state: &BlockState,
    ) {
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    self.add_block(chunk, state, x, y, z, box_limit);
                }
            }
        }
    }

    /// Fills downwards from a relative point until the bottom of the world.
    pub fn fill_downwards(
        &self,
        chunk: &mut ProtoChunk,
        state: &BlockState,
        x: i32,
        y: i32,
        z: i32,
        box_limit: &BlockBox,
    ) {
        // We transform the starting point to world space to get the real Y
        let world_pos = self.offset_pos(x, y, z);
        let start_y = world_pos.y;
        let end_y = chunk.bottom_y() as i32;

        for current_y in (end_y..=start_y).rev() {
            // We bypass add_block here because add_block transforms X/Z again.
            // We use the already-transformed world X/Z but iterate Y manually.
            if box_limit.contains(world_pos.x, current_y, world_pos.z) {
                chunk.set_block_state(world_pos.x, current_y, world_pos.z, state);
            }
        }
    }

    pub fn is_under_sea_level(
        &self,
        chunk: &mut ProtoChunk,
        x: i32,
        y: i32,
        z: i32,
        box_limit: &BlockBox,
    ) -> bool {
        let block_pos = self.offset_pos(x, y, z);

        if !box_limit.contains_pos(&block_pos) {
            return false;
        }

        let sea_level_at_pos = chunk.get_top_y(&HeightMap::OceanFloorWg, block_pos.x, block_pos.z);
        block_pos.y < sea_level_at_pos
    }

    #[must_use]
    pub fn get_block_at(
        &self,
        chunk: &ProtoChunk,
        x: i32,
        y: i32,
        z: i32,
        box_limit: &BlockBox,
    ) -> &BlockState {
        let block_pos = self.offset_pos(x, y, z);

        if !box_limit.contains_pos(&block_pos) {
            log::debug!("Structure out of bounds");
            return Block::AIR.default_state;
        }

        chunk.get_block_state(&block_pos).to_state()
    }

    pub fn add_block(
        &self,
        world: &mut ProtoChunk,
        block: &BlockState,
        x: i32,
        y: i32,
        z: i32,
        box_limit: &BlockBox,
    ) {
        let block_pos = self.offset_pos(x, y, z);

        // Bounds and logic checks
        if !box_limit.contains_pos(&block_pos) {
            log::debug!("Structure out of bounds");
            return;
        }

        // // Apply Mirror and Rotation
        // if self.mirror != BlockMirror::None {
        //     block = block.mirror(self.mirror);
        // }
        // if self.rotation != BlockRotation::None {
        //     block = block.rotate(self.rotation);
        // }

        // World interaction
        world.set_block_state(block_pos.x, block_pos.y, block_pos.z, block);

        // let fluid_state = world.get_fluid_state(&block_pos);
        // if !fluid_state.is_empty() {
        //     world.schedule_fluid_tick(&block_pos, fluid_state.fluid(), 0);
        // }

        // if block.needs_post_processing() {
        //     world.mark_block_for_post_processing(&block_pos);
        // }
    }

    pub fn get_random_horizontal_direction(random: &mut impl RandomImpl) -> BlockDirection {
        match random.next_bounded_i32(4) {
            0 => BlockDirection::North,
            1 => BlockDirection::East,
            2 => BlockDirection::South,
            _ => BlockDirection::West,
        }
    }
}

impl StructurePieceBase for StructurePiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, _chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {}

    fn translate(&mut self, x: i32, y: i32, z: i32) {
        self.bounding_box.move_pos(x, y, z);
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        self
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        self
    }
}

/// Holds all the pieces that make up a generated structure instance.
#[derive(Default)]
pub struct StructurePiecesCollector {
    pub pieces: Vec<Box<dyn StructurePieceBase>>,
    cached_box: Option<BlockBox>,
}

impl StructurePiecesCollector {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pieces: Vec::new(),
            cached_box: None,
        }
    }

    pub fn add_piece(&mut self, piece: Box<dyn StructurePieceBase>) {
        self.pieces.push(piece);
        self.cached_box = None;
    }

    #[must_use]
    pub fn get_intersecting(&self, box_to_check: &BlockBox) -> Option<&dyn StructurePieceBase> {
        self.pieces
            .iter()
            .find(|piece| {
                piece
                    .get_structure_piece()
                    .bounding_box
                    .intersects(box_to_check)
            })
            .map(|v| v.as_ref() as &dyn StructurePieceBase)
    }

    /// Iterates over all pieces and generates them if they intersect the current chunk.
    pub fn generate_in_chunk(
        &mut self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        seed: i64,
    ) {
        let chunk_x = start_block_x(chunk.x);
        let chunk_z = start_block_z(chunk.z);
        let chunk_box = BlockBox::new(
            chunk_x,
            chunk.bottom_y() as i32 + 1,
            chunk_z,
            chunk_x + 15,
            chunk.bottom_y() as i32 + chunk.top_y() as i32 - 1,
            chunk_z + 15,
        );

        for piece in &mut self.pieces {
            if piece.bounding_box().intersects(&chunk_box) {
                piece.place(chunk, random, seed);
            }
        }
    }

    pub fn shift(&mut self, y_offset: i32) {
        for piece in &mut self.pieces {
            piece.translate(0, y_offset, 0);
        }
        self.cached_box = None;
    }

    /// Calculates a random vertical position and shifts the structure to fit.
    /// Matches 'shiftInto(int topY, int bottomY, Random random, int topPenalty)'
    pub fn shift_into(
        &mut self,
        top_y: i32,
        bottom_y: i32,
        random: &mut RandomGenerator,
        top_penalty: i32,
    ) -> i32 {
        let i = top_y - top_penalty;
        let bounding_box = self.get_bounding_box();

        let mut j = (bounding_box.max.y - bounding_box.min.y + 1) + bottom_y + 1;

        if j < i {
            j += random.next_bounded_i32(i - j);
        }

        let k = j - bounding_box.max.y;

        self.shift(k);

        k
    }

    pub fn get_bounding_box(&mut self) -> BlockBox {
        if let Some(bbox) = self.cached_box {
            return bbox;
        }

        let bbox = BlockBox::encompass_all(self.pieces.iter().map(|p| p.bounding_box()))
            .expect("Structure must have at least one piece to calculate a bounding box");

        self.cached_box = Some(bbox);
        bbox
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    pub fn clear(&mut self) {
        self.pieces.clear();
    }
}

#[derive(Clone)]
pub struct StructurePosition {
    pub start_pos: BlockPos,
    pub collector: Arc<Mutex<StructurePiecesCollector>>,
}

impl StructurePosition {
    #[must_use]
    pub fn get_bounding_box(&self) -> BlockBox {
        self.collector.lock().unwrap().get_bounding_box()
    }
}

pub trait StructureGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext,
    ) -> Option<StructurePosition>;
}

pub struct StructureGeneratorContext {
    pub seed: i64,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub random: RandomGenerator,
    pub sea_level: i32,
    pub min_y: i32,
}

#[must_use]
pub fn create_chunk_random(seed: i64, chunk_x: i32, chunk_z: i32) -> RandomGenerator {
    let mut random: RandomGenerator = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed as u64));
    let carver_seed = get_carver_seed(&mut random, seed as u64, chunk_x, chunk_z);
    RandomGenerator::Xoroshiro(Xoroshiro::from_seed(carver_seed))
}

#[derive(Clone)]
pub enum StructureInstance {
    /// This chunk is the "owner" of the structure.
    Start(StructurePosition),
    /// This chunk just contains a piece of a structure starting elsewhere.
    /// Stores the `BlockPos` of the 'Start' so you can look it up.
    Reference(BlockPos),
}
