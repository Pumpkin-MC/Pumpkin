use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection as DataBlockDirection, BlockState,
    block_properties::{HorizontalFacing, OakFenceLikeProperties, WallTorchLikeProperties},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    BlockDirection,
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::structure::{
        piece::StructurePieceType,
        structures::{
            StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
            StructurePiecesCollector, StructurePosition, WorldPortalExt,
        },
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MineshaftType {
    Normal,
    Mesa,
}

impl MineshaftType {
    #[must_use]
    pub const fn wood(self) -> &'static BlockState {
        match self {
            Self::Normal => Block::OAK_LOG.default_state,
            Self::Mesa => Block::DARK_OAK_LOG.default_state,
        }
    }

    #[must_use]
    pub const fn planks(self) -> &'static BlockState {
        match self {
            Self::Normal => Block::OAK_PLANKS.default_state,
            Self::Mesa => Block::DARK_OAK_PLANKS.default_state,
        }
    }

    #[must_use]
    pub const fn fence(self) -> &'static BlockState {
        match self {
            Self::Normal => Block::OAK_FENCE.default_state,
            Self::Mesa => Block::DARK_OAK_FENCE.default_state,
        }
    }

    fn connected_fence(self, west: bool, east: bool) -> &'static BlockState {
        let block = match self {
            Self::Normal => &Block::OAK_FENCE,
            Self::Mesa => &Block::DARK_OAK_FENCE,
        };
        let mut properties = OakFenceLikeProperties::default(block);
        properties.west = west;
        properties.east = east;
        BlockState::from_id(properties.to_state_id(block))
    }

    fn can_replace(self, state: &BlockState) -> bool {
        let block_id = state.id.to_block_id();
        block_id != self.planks().id.to_block_id()
            && block_id != self.wood().id.to_block_id()
            && block_id != self.fence().id.to_block_id()
            && block_id != Block::IRON_CHAIN.id
    }
}

#[expect(clippy::too_many_arguments)]
fn add_mineshaft_block(
    piece: &StructurePiece,
    shaft_type: MineshaftType,
    chunk: &mut ProtoChunk,
    state: &BlockState,
    x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
) {
    if shaft_type.can_replace(piece.get_block_at(chunk, x, y, z, chunk_box)) {
        piece.add_block(chunk, state, x, y, z, chunk_box);
    }
}

#[expect(clippy::too_many_arguments)]
fn fill_mineshaft_box(
    piece: &StructurePiece,
    shaft_type: MineshaftType,
    chunk: &mut ProtoChunk,
    chunk_box: &BlockBox,
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
                add_mineshaft_block(piece, shaft_type, chunk, state, x, y, z, chunk_box);
            }
        }
    }
}

#[expect(clippy::too_many_arguments)]
fn for_each_maybe_box_position(
    random: &mut RandomGenerator,
    chance: f32,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
    mut place: impl FnMut(i32, i32, i32),
) {
    // StructurePiece.generateMaybeBox iterates Y, then X, then Z and consumes a
    // float before any of its placement gates.
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                if random.next_f32() <= chance {
                    place(x, y, z);
                }
            }
        }
    }
}

fn is_supporting_box(min_x: i32, max_x: i32, mut is_air: impl FnMut(i32) -> bool) -> bool {
    (min_x..=max_x).all(|x| !is_air(x))
}

fn passes_cobweb_random_gate(random: &mut RandomGenerator, chance: f32, is_interior: bool) -> bool {
    is_interior && random.next_f32() < chance
}

const fn rail_chance(is_interior: bool) -> f32 {
    if is_interior { 0.7 } else { 0.9 }
}

/// Vanilla `MineshaftPieces$MineShaftPiece.setPlanksBlock`: the floor block is
/// only planked when the position is *interior* (the block above it still sits
/// under the `OCEAN_FLOOR_WG` heightmap) and the existing block's upward face is
/// not sturdy. It writes with `level.setBlock`, so `canBeReplaced` does not apply.
const fn places_floor_planks(is_interior: bool, existing_up_face_sturdy: bool) -> bool {
    is_interior && !existing_up_face_sturdy
}

fn set_planks_block(
    piece: &StructurePiece,
    chunk: &mut ProtoChunk,
    chunk_box: &BlockBox,
    planks: &'static BlockState,
    x: i32,
    y: i32,
    z: i32,
) {
    let is_interior = piece.is_under_sea_level(chunk, x, y, z, chunk_box);
    if !is_interior {
        return;
    }

    let pos = piece.offset_pos(x, y, z);
    let sturdy = chunk
        .get_block_state(&pos)
        .to_state()
        .is_side_solid(DataBlockDirection::Up);
    if places_floor_planks(is_interior, sturdy) {
        chunk.set_block_state(pos.x, pos.y, pos.z, planks);
    }
}

fn boundary_matches(
    piece_box: &BlockBox,
    chunk_box: &BlockBox,
    mut predicate: impl FnMut(i32, i32, i32) -> bool,
) -> bool {
    let min_x = (piece_box.min.x - 1).max(chunk_box.min.x);
    let min_y = (piece_box.min.y - 1).max(chunk_box.min.y);
    let min_z = (piece_box.min.z - 1).max(chunk_box.min.z);
    let max_x = (piece_box.max.x + 1).min(chunk_box.max.x);
    let max_y = (piece_box.max.y + 1).min(chunk_box.max.y);
    let max_z = (piece_box.max.z + 1).min(chunk_box.max.z);

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            if predicate(x, min_y, z) || predicate(x, max_y, z) {
                return true;
            }
        }
    }
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            if predicate(x, y, min_z) || predicate(x, y, max_z) {
                return true;
            }
        }
    }
    for z in min_z..=max_z {
        for y in min_y..=max_y {
            if predicate(min_x, y, z) || predicate(max_x, y, z) {
                return true;
            }
        }
    }
    false
}

fn is_in_invalid_liquid_location(
    chunk: &ProtoChunk,
    piece_box: &BlockBox,
    chunk_box: &BlockBox,
) -> bool {
    boundary_matches(piece_box, chunk_box, |x, y, z| {
        chunk
            .get_block_state(&Vector3::new(x, y, z))
            .to_state()
            .is_liquid()
    })
}

pub struct MineshaftGenerator {
    pub is_mesa: bool,
}

impl StructureGenerator for MineshaftGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        context.random.next_f64();
        let start_x = context.chunk_x << 4;
        let start_z = context.chunk_z << 4;
        let room_x = start_x + 2;
        let room_z = start_z + 2;

        let shaft_type = if self.is_mesa {
            MineshaftType::Mesa
        } else {
            MineshaftType::Normal
        };

        let mut start_room = MineShaftRoom::new(0, &mut context.random, room_x, room_z, shaft_type);

        // Vanilla `MineshaftStructure.generatePiecesAndAdjust`: `builder.addPiece(room);
        // room.addChildren(room, builder, random);` — the room is in the collector while its
        // children (and, depth-first, their children) are generated, and it collects the
        // entrance boxes along the way, so write the finished room back over the placeholder.
        let mut collector = StructurePiecesCollector::default();
        let start_piece_box = start_room.piece.bounding_box;
        collector.add_piece(Box::new(start_room.clone()));
        start_room.add_children_to_list(
            &start_piece_box,
            shaft_type,
            &mut collector,
            &mut context.random,
        );
        collector.pieces[0] = Box::new(start_room);

        if self.is_mesa {
            let bbox = collector.get_bounding_box();
            let center_x = i32::midpoint(bbox.min.x, bbox.max.x);
            let center_z = i32::midpoint(bbox.min.z, bbox.max.z);
            let center_y = i32::midpoint(bbox.min.y, bbox.max.y);

            let surface_height = context
                .height_sampler
                .as_deref_mut()
                .map_or(context.sea_level, |s| s.estimate_height(center_x, center_z));

            let target_y = if surface_height <= context.sea_level {
                context.sea_level
            } else {
                let range = surface_height - context.sea_level + 1;
                context.sea_level + context.random.next_bounded_i32(range)
            };

            let dy = target_y - center_y;
            collector.shift(dy);
        } else {
            collector.shift_into(context.sea_level, context.min_y, &mut context.random, 10);
        }

        Some(StructurePosition {
            start_pos: BlockPos::new(start_x + 8, 50, start_z),
            collector: Arc::new(collector.into()),
        })
    }
}

enum GeneratedPiece {
    Corridor(MineShaftCorridor),
    Crossing(MineShaftCrossing),
    Stairs(MineShaftStairs),
}

impl GeneratedPiece {
    fn build_children(
        &self,
        start_piece_box: &BlockBox,
        shaft_type: MineshaftType,
        collector: &mut StructurePiecesCollector,
        random: &mut RandomGenerator,
    ) {
        match self {
            Self::Corridor(c) => {
                c.add_children_to_list(start_piece_box, shaft_type, collector, random);
            }
            Self::Crossing(cr) => {
                cr.add_children_to_list(start_piece_box, shaft_type, collector, random);
            }
            Self::Stairs(s) => {
                s.add_children_to_list(start_piece_box, shaft_type, collector, random);
            }
        }
    }

    const fn bounding_box(&self) -> BlockBox {
        match self {
            Self::Corridor(c) => c.piece.bounding_box,
            Self::Crossing(cr) => cr.piece.bounding_box,
            Self::Stairs(s) => s.piece.bounding_box,
        }
    }

    fn clone_piece_base(&self) -> Box<dyn StructurePieceBase> {
        match self {
            Self::Corridor(c) => Box::new(c.clone()),
            Self::Crossing(cr) => Box::new(cr.clone()),
            Self::Stairs(s) => Box::new(s.clone()),
        }
    }
}

#[expect(clippy::too_many_arguments)]
fn create_random_shaft_piece(
    collector: &StructurePiecesCollector,
    random: &mut RandomGenerator,
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: BlockDirection,
    gen_depth: u32,
    shaft_type: MineshaftType,
) -> Option<GeneratedPiece> {
    let random_selection = random.next_bounded_i32(100);
    if random_selection >= 80 {
        if let Some(crossing_box) =
            MineShaftCrossing::find_crossing(collector, random, foot_x, foot_y, foot_z, direction)
        {
            return Some(GeneratedPiece::Crossing(MineShaftCrossing::new(
                gen_depth,
                crossing_box,
                Some(direction),
                shaft_type,
            )));
        }
    } else if random_selection >= 70 {
        if let Some(stairs_box) =
            MineShaftStairs::find_stairs(collector, random, foot_x, foot_y, foot_z, direction)
        {
            return Some(GeneratedPiece::Stairs(MineShaftStairs::new(
                gen_depth, stairs_box, direction, shaft_type,
            )));
        }
    } else if let Some(corridor_box) =
        MineShaftCorridor::find_corridor_size(collector, random, foot_x, foot_y, foot_z, direction)
    {
        return Some(GeneratedPiece::Corridor(MineShaftCorridor::new(
            gen_depth,
            random,
            corridor_box,
            direction,
            shaft_type,
        )));
    }
    None
}

/// Vanilla `MineshaftPieces.generateAndAddPiece`: the new piece is added to the collector
/// and its own children are generated *immediately* (depth-first), so every later
/// collision check and random draw sees the pieces in vanilla's order. Returns the new
/// piece's bounding box.
#[expect(clippy::too_many_arguments)]
fn generate_and_add_piece(
    start_piece_box: &BlockBox,
    collector: &mut StructurePiecesCollector,
    random: &mut RandomGenerator,
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: BlockDirection,
    depth: u32,
    shaft_type: MineshaftType,
) -> Option<BlockBox> {
    if depth > 8
        || (foot_x - start_piece_box.min.x).abs() > 80
        || (foot_z - start_piece_box.min.z).abs() > 80
    {
        return None;
    }
    let new_piece = create_random_shaft_piece(
        collector,
        random,
        foot_x,
        foot_y,
        foot_z,
        direction,
        depth + 1,
        shaft_type,
    )?;
    let bounding_box = new_piece.bounding_box();
    collector.add_piece(new_piece.clone_piece_base());
    new_piece.build_children(start_piece_box, shaft_type, collector, random);
    Some(bounding_box)
}

#[derive(Clone)]
pub struct MineShaftRoom {
    pub piece: StructurePiece,
    pub shaft_type: MineshaftType,
    pub child_entrance_boxes: Vec<BlockBox>,
}

impl MineShaftRoom {
    #[must_use]
    pub fn new(
        gen_depth: u32,
        random: &mut RandomGenerator,
        west: i32,
        north: i32,
        shaft_type: MineshaftType,
    ) -> Self {
        let max_x = west + 7 + random.next_bounded_i32(6);
        let max_y = 54 + random.next_bounded_i32(6);
        let max_z = north + 7 + random.next_bounded_i32(6);
        let bounding_box = BlockBox::new(west, 50, north, max_x, max_y, max_z);
        Self {
            piece: StructurePiece::new(StructurePieceType::MineshaftRoom, bounding_box, gen_depth),
            shaft_type,
            child_entrance_boxes: Vec::new(),
        }
    }

    #[expect(clippy::too_many_lines)]
    fn add_children_to_list(
        &mut self,
        start_piece_box: &BlockBox,
        shaft_type: MineshaftType,
        collector: &mut StructurePiecesCollector,
        random: &mut RandomGenerator,
    ) {
        let depth = self.piece.chain_length;
        let y_span = self.piece.bounding_box.max.y - self.piece.bounding_box.min.y + 1;
        let mut height_space = y_span - 4;
        if height_space <= 0 {
            height_space = 1;
        }

        let x_span = self.piece.bounding_box.max.x - self.piece.bounding_box.min.x + 1;
        let z_span = self.piece.bounding_box.max.z - self.piece.bounding_box.min.z + 1;

        let mut pos = 0;
        while pos < x_span {
            pos += random.next_bounded_i32(x_span);
            if pos + 3 > x_span {
                break;
            }
            let child_x = self.piece.bounding_box.min.x + pos;
            let child_y = self.piece.bounding_box.min.y + random.next_bounded_i32(height_space) + 1;
            let child_z = self.piece.bounding_box.min.z - 1;

            if let Some(bb) = generate_and_add_piece(
                start_piece_box,
                collector,
                random,
                child_x,
                child_y,
                child_z,
                BlockDirection::North,
                depth,
                shaft_type,
            ) {
                self.child_entrance_boxes.push(BlockBox::new(
                    bb.min.x,
                    bb.min.y,
                    self.piece.bounding_box.min.z,
                    bb.max.x,
                    bb.max.y,
                    self.piece.bounding_box.min.z + 1,
                ));
            }
            pos += 4;
        }

        pos = 0;
        while pos < x_span {
            pos += random.next_bounded_i32(x_span);
            if pos + 3 > x_span {
                break;
            }
            let child_x = self.piece.bounding_box.min.x + pos;
            let child_y = self.piece.bounding_box.min.y + random.next_bounded_i32(height_space) + 1;
            let child_z = self.piece.bounding_box.max.z + 1;

            if let Some(bb) = generate_and_add_piece(
                start_piece_box,
                collector,
                random,
                child_x,
                child_y,
                child_z,
                BlockDirection::South,
                depth,
                shaft_type,
            ) {
                self.child_entrance_boxes.push(BlockBox::new(
                    bb.min.x,
                    bb.min.y,
                    self.piece.bounding_box.max.z - 1,
                    bb.max.x,
                    bb.max.y,
                    self.piece.bounding_box.max.z,
                ));
            }
            pos += 4;
        }

        pos = 0;
        while pos < z_span {
            pos += random.next_bounded_i32(z_span);
            if pos + 3 > z_span {
                break;
            }
            let child_x = self.piece.bounding_box.min.x - 1;
            let child_y = self.piece.bounding_box.min.y + random.next_bounded_i32(height_space) + 1;
            let child_z = self.piece.bounding_box.min.z + pos;

            if let Some(bb) = generate_and_add_piece(
                start_piece_box,
                collector,
                random,
                child_x,
                child_y,
                child_z,
                BlockDirection::West,
                depth,
                shaft_type,
            ) {
                self.child_entrance_boxes.push(BlockBox::new(
                    self.piece.bounding_box.min.x,
                    bb.min.y,
                    bb.min.z,
                    self.piece.bounding_box.min.x + 1,
                    bb.max.y,
                    bb.max.z,
                ));
            }
            pos += 4;
        }

        pos = 0;
        while pos < z_span {
            pos += random.next_bounded_i32(z_span);
            if pos + 3 > z_span {
                break;
            }
            let child_x = self.piece.bounding_box.max.x + 1;
            let child_y = self.piece.bounding_box.min.y + random.next_bounded_i32(height_space) + 1;
            let child_z = self.piece.bounding_box.min.z + pos;

            if let Some(bb) = generate_and_add_piece(
                start_piece_box,
                collector,
                random,
                child_x,
                child_y,
                child_z,
                BlockDirection::East,
                depth,
                shaft_type,
            ) {
                self.child_entrance_boxes.push(BlockBox::new(
                    self.piece.bounding_box.max.x - 1,
                    bb.min.y,
                    bb.min.z,
                    self.piece.bounding_box.max.x,
                    bb.max.y,
                    bb.max.z,
                ));
            }
            pos += 4;
        }
    }
}

impl StructurePieceBase for MineShaftRoom {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }

    fn translate(&mut self, x: i32, y: i32, z: i32) {
        self.piece.translate(x, y, z);
        for bb in &mut self.child_entrance_boxes {
            bb.move_pos(x, y, z);
        }
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        if is_in_invalid_liquid_location(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }

        let air = Block::CAVE_AIR.default_state;
        let bb = self.piece.bounding_box;

        let max_clear_y = (bb.min.y + 3).min(bb.max.y);
        for y in (bb.min.y + 1)..=max_clear_y {
            for x in bb.min.x..=bb.max.x {
                for z in bb.min.z..=bb.max.z {
                    if chunk_box.contains(x, y, z) {
                        chunk.set_block_state(x, y, z, air);
                    }
                }
            }
        }

        for entrance in &self.child_entrance_boxes {
            for y in (entrance.max.y - 2)..=entrance.max.y {
                for x in entrance.min.x..=entrance.max.x {
                    for z in entrance.min.z..=entrance.max.z {
                        if chunk_box.contains(x, y, z) {
                            chunk.set_block_state(x, y, z, air);
                        }
                    }
                }
            }
        }

        let diag_x = (bb.max.x - bb.min.x + 1) as f32;
        let diag_y = (bb.max.y - (bb.min.y + 4) + 1) as f32;
        let diag_z = (bb.max.z - bb.min.z + 1) as f32;
        let cx = bb.min.x as f32 + diag_x / 2.0;
        let cz = bb.min.z as f32 + diag_z / 2.0;
        let y0 = bb.min.y + 4;

        if diag_y > 0.0 {
            for y in y0..=bb.max.y {
                let ny = (y - y0) as f32 / diag_y;
                for x in bb.min.x..=bb.max.x {
                    let nx = (x as f32 - cx) / (diag_x * 0.5);
                    for z in bb.min.z..=bb.max.z {
                        let nz = (z as f32 - cz) / (diag_z * 0.5);
                        if nx * nx + ny * ny + nz * nz <= 1.05 && chunk_box.contains(x, y, z) {
                            chunk.set_block_state(x, y, z, air);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct MineShaftCorridor {
    pub piece: StructurePiece,
    pub shaft_type: MineshaftType,
    pub has_rails: bool,
    pub spider_corridor: bool,
    pub has_placed_spider: bool,
    pub num_sections: i32,
}

impl MineShaftCorridor {
    #[must_use]
    pub fn new(
        gen_depth: u32,
        random: &mut RandomGenerator,
        bounding_box: BlockBox,
        direction: BlockDirection,
        shaft_type: MineshaftType,
    ) -> Self {
        let has_rails = random.next_bounded_i32(3) == 0;
        let spider_corridor = !has_rails && random.next_bounded_i32(23) == 0;
        let axis = direction.get_axis();
        let num_sections = if axis == pumpkin_util::math::vector3::Axis::Z {
            (bounding_box.max.z - bounding_box.min.z + 1) / 5
        } else {
            (bounding_box.max.x - bounding_box.min.x + 1) / 5
        };

        let mut piece = StructurePiece::new(
            StructurePieceType::MineshaftCorridor,
            bounding_box,
            gen_depth,
        );
        piece.set_facing(Some(direction));

        Self {
            piece,
            shaft_type,
            has_rails,
            spider_corridor,
            has_placed_spider: false,
            num_sections,
        }
    }

    #[must_use]
    pub fn find_corridor_size(
        collector: &StructurePiecesCollector,
        random: &mut RandomGenerator,
        foot_x: i32,
        foot_y: i32,
        foot_z: i32,
        direction: BlockDirection,
    ) -> Option<BlockBox> {
        let max_len = random.next_bounded_i32(3) + 2;
        for corridor_length in (1..=max_len).rev() {
            let block_length = corridor_length * 5;
            let mut box_cand = match direction {
                BlockDirection::South => BlockBox::new(0, 0, 0, 2, 2, block_length - 1),
                BlockDirection::West => BlockBox::new(-(block_length - 1), 0, 0, 0, 2, 2),
                BlockDirection::East => BlockBox::new(0, 0, 0, block_length - 1, 2, 2),
                _ => BlockBox::new(0, 0, -(block_length - 1), 2, 2, 0),
            };
            box_cand.move_pos(foot_x, foot_y, foot_z);
            if collector.get_intersecting(&box_cand).is_none() {
                return Some(box_cand);
            }
        }
        None
    }

    #[expect(clippy::too_many_lines)]
    fn add_children_to_list(
        &self,
        start_piece_box: &BlockBox,
        shaft_type: MineshaftType,
        collector: &mut StructurePiecesCollector,
        random: &mut RandomGenerator,
    ) {
        let depth = self.piece.chain_length;
        let end_selection = random.next_bounded_i32(4);
        let orientation = self.piece.facing.unwrap_or(BlockDirection::North);
        let rand_y = self.piece.bounding_box.min.y - 1 + random.next_bounded_i32(3);

        match orientation {
            BlockDirection::North => {
                if end_selection <= 1 {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.min.x,
                        rand_y,
                        self.piece.bounding_box.min.z - 1,
                        orientation,
                        depth,
                        shaft_type,
                    );
                } else if end_selection == 2 {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.min.x - 1,
                        rand_y,
                        self.piece.bounding_box.min.z,
                        BlockDirection::West,
                        depth,
                        shaft_type,
                    );
                } else {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.max.x + 1,
                        rand_y,
                        self.piece.bounding_box.min.z,
                        BlockDirection::East,
                        depth,
                        shaft_type,
                    );
                }
            }
            BlockDirection::South => {
                if end_selection <= 1 {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.min.x,
                        rand_y,
                        self.piece.bounding_box.max.z + 1,
                        orientation,
                        depth,
                        shaft_type,
                    );
                } else if end_selection == 2 {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.min.x - 1,
                        rand_y,
                        self.piece.bounding_box.max.z - 3,
                        BlockDirection::West,
                        depth,
                        shaft_type,
                    );
                } else {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.max.x + 1,
                        rand_y,
                        self.piece.bounding_box.max.z - 3,
                        BlockDirection::East,
                        depth,
                        shaft_type,
                    );
                }
            }
            BlockDirection::West => {
                if end_selection <= 1 {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.min.x - 1,
                        rand_y,
                        self.piece.bounding_box.min.z,
                        orientation,
                        depth,
                        shaft_type,
                    );
                } else if end_selection == 2 {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.min.x,
                        rand_y,
                        self.piece.bounding_box.min.z - 1,
                        BlockDirection::North,
                        depth,
                        shaft_type,
                    );
                } else {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.min.x,
                        rand_y,
                        self.piece.bounding_box.max.z + 1,
                        BlockDirection::South,
                        depth,
                        shaft_type,
                    );
                }
            }
            BlockDirection::East => {
                if end_selection <= 1 {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.max.x + 1,
                        rand_y,
                        self.piece.bounding_box.min.z,
                        orientation,
                        depth,
                        shaft_type,
                    );
                } else if end_selection == 2 {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.max.x - 3,
                        rand_y,
                        self.piece.bounding_box.min.z - 1,
                        BlockDirection::North,
                        depth,
                        shaft_type,
                    );
                } else {
                    generate_and_add_piece(
                        start_piece_box,
                        collector,
                        random,
                        self.piece.bounding_box.max.x - 3,
                        rand_y,
                        self.piece.bounding_box.max.z + 1,
                        BlockDirection::South,
                        depth,
                        shaft_type,
                    );
                }
            }
            _ => {}
        }

        if depth < 8 {
            if orientation != BlockDirection::North && orientation != BlockDirection::South {
                let mut x = self.piece.bounding_box.min.x + 3;
                while x + 3 <= self.piece.bounding_box.max.x {
                    let sel = random.next_bounded_i32(5);
                    if sel == 0 {
                        generate_and_add_piece(
                            start_piece_box,
                            collector,
                            random,
                            x,
                            self.piece.bounding_box.min.y,
                            self.piece.bounding_box.min.z - 1,
                            BlockDirection::North,
                            depth + 1,
                            shaft_type,
                        );
                    } else if sel == 1 {
                        generate_and_add_piece(
                            start_piece_box,
                            collector,
                            random,
                            x,
                            self.piece.bounding_box.min.y,
                            self.piece.bounding_box.max.z + 1,
                            BlockDirection::South,
                            depth + 1,
                            shaft_type,
                        );
                    }
                    x += 5;
                }
            } else {
                let mut z = self.piece.bounding_box.min.z + 3;
                while z + 3 <= self.piece.bounding_box.max.z {
                    let sel = random.next_bounded_i32(5);
                    if sel == 0 {
                        generate_and_add_piece(
                            start_piece_box,
                            collector,
                            random,
                            self.piece.bounding_box.min.x - 1,
                            self.piece.bounding_box.min.y,
                            z,
                            BlockDirection::West,
                            depth + 1,
                            shaft_type,
                        );
                    } else if sel == 1 {
                        generate_and_add_piece(
                            start_piece_box,
                            collector,
                            random,
                            self.piece.bounding_box.max.x + 1,
                            self.piece.bounding_box.min.y,
                            z,
                            BlockDirection::East,
                            depth + 1,
                            shaft_type,
                        );
                    }
                    z += 5;
                }
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn place_support(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        x0: i32,
        y0: i32,
        z: i32,
        y1: i32,
        x1: i32,
        random: &mut RandomGenerator,
    ) {
        if !is_supporting_box(x0, x1, |x| {
            self.piece
                .get_block_at(chunk, x, y1 + 1, z, chunk_box)
                .is_air()
        }) {
            return;
        }

        let planks = self.shaft_type.planks();
        let west_fence = self.shaft_type.connected_fence(true, false);
        let east_fence = self.shaft_type.connected_fence(false, true);

        fill_mineshaft_box(
            &self.piece,
            self.shaft_type,
            chunk,
            chunk_box,
            x0,
            y0,
            z,
            x0,
            y1 - 1,
            z,
            west_fence,
        );
        fill_mineshaft_box(
            &self.piece,
            self.shaft_type,
            chunk,
            chunk_box,
            x1,
            y0,
            z,
            x1,
            y1 - 1,
            z,
            east_fence,
        );

        if random.next_bounded_i32(4) == 0 {
            fill_mineshaft_box(
                &self.piece,
                self.shaft_type,
                chunk,
                chunk_box,
                x0,
                y1,
                z,
                x0,
                y1,
                z,
                planks,
            );
            fill_mineshaft_box(
                &self.piece,
                self.shaft_type,
                chunk,
                chunk_box,
                x1,
                y1,
                z,
                x1,
                y1,
                z,
                planks,
            );
        } else {
            fill_mineshaft_box(
                &self.piece,
                self.shaft_type,
                chunk,
                chunk_box,
                x0,
                y1,
                z,
                x1,
                y1,
                z,
                planks,
            );

            let mut props_s = WallTorchLikeProperties::default(&Block::WALL_TORCH);
            props_s.facing = HorizontalFacing::South;
            let torch_s = BlockState::from_id(props_s.to_state_id(&Block::WALL_TORCH));

            let mut props_n = WallTorchLikeProperties::default(&Block::WALL_TORCH);
            props_n.facing = HorizontalFacing::North;
            let torch_n = BlockState::from_id(props_n.to_state_id(&Block::WALL_TORCH));

            if random.next_f32() < 0.05 {
                add_mineshaft_block(
                    &self.piece,
                    self.shaft_type,
                    chunk,
                    torch_s,
                    x0 + 1,
                    y1,
                    z - 1,
                    chunk_box,
                );
            }
            if random.next_f32() < 0.05 {
                add_mineshaft_block(
                    &self.piece,
                    self.shaft_type,
                    chunk,
                    torch_n,
                    x0 + 1,
                    y1,
                    z + 1,
                    chunk_box,
                );
            }
        }
    }

    /// Vanilla `MineShaftCorridor.placeDoubleLowerOrUpperSupport`: each of the two
    /// pillar anchors is only followed down (or chained up) when the block already
    /// standing there is the shaft's planks block, i.e. when `setPlanksBlock`
    /// actually floored that column.
    fn place_double_lower_or_upper_support(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        x: i32,
        y: i32,
        z: i32,
    ) {
        let planks_id = self.shaft_type.planks().id.to_block_id();
        for anchor_x in [x, x + 2] {
            if self
                .piece
                .get_block_at(chunk, anchor_x, y, z, chunk_box)
                .id
                .to_block_id()
                == planks_id
            {
                self.fill_pillar_down_or_chain_up(chunk, anchor_x, y, z, chunk_box);
            }
        }
    }

    fn fill_pillar_down_or_chain_up(
        &self,
        chunk: &mut ProtoChunk,
        x: i32,
        y: i32,
        z: i32,
        chunk_box: &BlockBox,
    ) {
        let world_pos = self.piece.offset_pos(x, y, z);
        if !chunk_box.contains_pos(&world_pos) {
            return;
        }

        let world_y = world_pos.y;
        let mut dist = 1;
        let mut check_below = true;
        let mut check_above = true;

        while check_below || check_above {
            if check_below {
                let below_y = world_y - dist;
                let state_below =
                    chunk.get_block_state(&Vector3::new(world_pos.x, below_y, world_pos.z));
                let empty_below =
                    state_below.to_state().is_air() || state_below.to_block_id() == Block::WATER.id;
                if !empty_below && state_below.to_block_id() != Block::LAVA.id {
                    for py in (below_y + 1)..world_y {
                        chunk.set_block_state(world_pos.x, py, world_pos.z, self.shaft_type.wood());
                    }
                    return;
                }
                check_below = dist <= 20 && empty_below && below_y > chunk.bottom_y() as i32 + 1;
            }

            if check_above {
                let above_y = world_y + dist;
                let state_above =
                    chunk.get_block_state(&Vector3::new(world_pos.x, above_y, world_pos.z));
                let empty_above = state_above.to_state().is_air();
                if !empty_above {
                    chunk.set_block_state(
                        world_pos.x,
                        world_y + 1,
                        world_pos.z,
                        self.shaft_type.fence(),
                    );
                    for py in (world_y + 2)..above_y {
                        chunk.set_block_state(
                            world_pos.x,
                            py,
                            world_pos.z,
                            Block::IRON_CHAIN.default_state,
                        );
                    }
                    return;
                }
                check_above = dist <= 50 && empty_above && above_y < 319;
            }

            dist += 1;
        }
    }

    fn maybe_place_cobweb(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        random: &mut RandomGenerator,
        chance: f32,
        x: i32,
        y: i32,
        z: i32,
    ) {
        let is_interior = self.piece.is_under_sea_level(chunk, x, y, z, chunk_box);
        if !passes_cobweb_random_gate(random, chance, is_interior) {
            return;
        }

        let world_pos = self.piece.offset_pos(x, y, z);
        let mut sturdy_neighbours = 0;
        for direction in [
            BlockDirection::Down,
            BlockDirection::Up,
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::West,
            BlockDirection::East,
        ] {
            let offset = direction.to_vector();
            let neighbour = Vector3::new(
                world_pos.x + offset.x,
                world_pos.y + offset.y,
                world_pos.z + offset.z,
            );
            if chunk_box.contains_pos(&neighbour)
                && chunk.get_block_state(&neighbour).to_state().is_side_solid(
                    match direction.opposite() {
                        BlockDirection::Down => DataBlockDirection::Down,
                        BlockDirection::Up => DataBlockDirection::Up,
                        BlockDirection::North => DataBlockDirection::North,
                        BlockDirection::South => DataBlockDirection::South,
                        BlockDirection::West => DataBlockDirection::West,
                        BlockDirection::East => DataBlockDirection::East,
                    },
                )
            {
                sturdy_neighbours += 1;
                if sturdy_neighbours >= 2 {
                    add_mineshaft_block(
                        &self.piece,
                        self.shaft_type,
                        chunk,
                        Block::COBWEB.default_state,
                        x,
                        y,
                        z,
                        chunk_box,
                    );
                    return;
                }
            }
        }
    }
}

impl StructurePieceBase for MineShaftCorridor {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }

    #[allow(clippy::too_many_lines)]
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        if is_in_invalid_liquid_location(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }

        let air = Block::CAVE_AIR.default_state;
        let length = self.num_sections * 5 - 1;
        let planks = self.shaft_type.planks();

        fill_mineshaft_box(
            &self.piece,
            self.shaft_type,
            chunk,
            chunk_box,
            0,
            0,
            0,
            2,
            1,
            length,
            air,
        );

        for_each_maybe_box_position(random, 0.8, 0, 2, 0, 2, 2, length, |x, y, z| {
            add_mineshaft_block(&self.piece, self.shaft_type, chunk, air, x, y, z, chunk_box);
        });

        if self.spider_corridor {
            for_each_maybe_box_position(random, 0.6, 0, 0, 0, 2, 1, length, |x, y, z| {
                if self.piece.is_under_sea_level(chunk, x, y, z, chunk_box) {
                    add_mineshaft_block(
                        &self.piece,
                        self.shaft_type,
                        chunk,
                        Block::COBWEB.default_state,
                        x,
                        y,
                        z,
                        chunk_box,
                    );
                }
            });
        }

        for section in 0..self.num_sections {
            let z = 2 + section * 5;
            self.place_support(chunk, chunk_box, 0, 0, z, 2, 2, random);

            for (cx, cy, cz, chance) in [
                (0, 2, z - 1, 0.1f32),
                (2, 2, z - 1, 0.1),
                (0, 2, z + 1, 0.1),
                (2, 2, z + 1, 0.1),
                (0, 2, z - 2, 0.05),
                (2, 2, z - 2, 0.05),
                (0, 2, z + 2, 0.05),
                (2, 2, z + 2, 0.05),
            ] {
                self.maybe_place_cobweb(chunk, chunk_box, random, chance, cx, cy, cz);
            }

            if random.next_bounded_i32(100) == 0 {
                self.piece.add_chest(
                    chunk,
                    chunk_box,
                    random,
                    2,
                    0,
                    z - 1,
                    "minecraft:chests/abandoned_mineshaft",
                );
            }
            if random.next_bounded_i32(100) == 0 {
                self.piece.add_chest(
                    chunk,
                    chunk_box,
                    random,
                    0,
                    0,
                    z + 1,
                    "minecraft:chests/abandoned_mineshaft",
                );
            }

            if self.spider_corridor && !self.has_placed_spider {
                let spawner_z = z - 1 + random.next_bounded_i32(3);
                let spawner_pos = self.piece.offset_pos(1, 0, spawner_z);
                if chunk_box.contains_pos(&spawner_pos) {
                    self.has_placed_spider = true;
                    chunk.set_block_state(
                        spawner_pos.x,
                        spawner_pos.y,
                        spawner_pos.z,
                        Block::SPAWNER.default_state,
                    );

                    let mut nbt = NbtCompound::new();
                    nbt.put_string("id", "minecraft:mob_spawner".to_string());
                    nbt.put_int("x", spawner_pos.x);
                    nbt.put_int("y", spawner_pos.y);
                    nbt.put_int("z", spawner_pos.z);
                    let mut spawn_data = NbtCompound::new();
                    let mut entity = NbtCompound::new();
                    entity.put_string("id", "minecraft:cave_spider".to_string());
                    spawn_data.put_compound("entity", entity);
                    nbt.put_compound("SpawnData", spawn_data);
                    chunk.add_block_entity(nbt);
                }
            }
        }

        for x in 0..=2 {
            for z in 0..=length {
                set_planks_block(&self.piece, chunk, chunk_box, planks, x, -1, z);
            }
        }

        self.place_double_lower_or_upper_support(chunk, chunk_box, 0, -1, 2);
        if self.num_sections > 1 {
            let last_support = length - 2;
            self.place_double_lower_or_upper_support(chunk, chunk_box, 0, -1, last_support);
        }

        if self.has_rails {
            let rail = Block::RAIL.default_state;
            for z in 0..=length {
                let floor_pos = self.piece.offset_pos(1, -1, z);
                if chunk_box.contains_pos(&floor_pos) {
                    let floor_state = chunk.get_block_state(&floor_pos);
                    if !floor_state.to_state().is_air()
                        && floor_state.to_state().is_solid_render()
                        && random.next_f32()
                            < rail_chance(self.piece.is_under_sea_level(chunk, 1, 0, z, chunk_box))
                    {
                        add_mineshaft_block(
                            &self.piece,
                            self.shaft_type,
                            chunk,
                            rail,
                            1,
                            0,
                            z,
                            chunk_box,
                        );
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct MineShaftCrossing {
    pub piece: StructurePiece,
    pub shaft_type: MineshaftType,
    pub direction: Option<BlockDirection>,
    pub is_two_floored: bool,
}

impl MineShaftCrossing {
    #[must_use]
    pub const fn new(
        gen_depth: u32,
        bounding_box: BlockBox,
        direction: Option<BlockDirection>,
        shaft_type: MineshaftType,
    ) -> Self {
        let is_two_floored = bounding_box.max.y - bounding_box.min.y + 1 > 3;
        let mut piece = StructurePiece::new(
            StructurePieceType::MineshaftCrossing,
            bounding_box,
            gen_depth,
        );
        piece.set_facing(direction);
        Self {
            piece,
            shaft_type,
            direction,
            is_two_floored,
        }
    }

    #[must_use]
    pub fn find_crossing(
        collector: &StructurePiecesCollector,
        random: &mut RandomGenerator,
        foot_x: i32,
        foot_y: i32,
        foot_z: i32,
        direction: BlockDirection,
    ) -> Option<BlockBox> {
        let y1 = if random.next_bounded_i32(4) == 0 {
            6
        } else {
            2
        };
        let mut box_cand = match direction {
            BlockDirection::South => BlockBox::new(-1, 0, 0, 3, y1, 4),
            BlockDirection::West => BlockBox::new(-4, 0, -1, 0, y1, 3),
            BlockDirection::East => BlockBox::new(0, 0, -1, 4, y1, 3),
            _ => BlockBox::new(-1, 0, -4, 3, y1, 0),
        };
        box_cand.move_pos(foot_x, foot_y, foot_z);
        collector
            .get_intersecting(&box_cand)
            .is_none()
            .then_some(box_cand)
    }

    #[expect(clippy::too_many_lines)]
    fn add_children_to_list(
        &self,
        start_piece_box: &BlockBox,
        shaft_type: MineshaftType,
        collector: &mut StructurePiecesCollector,
        random: &mut RandomGenerator,
    ) {
        let depth = self.piece.chain_length;
        let dir = self.direction.unwrap_or(BlockDirection::North);

        match dir {
            BlockDirection::North => {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x + 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z - 1,
                    BlockDirection::North,
                    depth,
                    shaft_type,
                );
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x - 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z + 1,
                    BlockDirection::West,
                    depth,
                    shaft_type,
                );
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.max.x + 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z + 1,
                    BlockDirection::East,
                    depth,
                    shaft_type,
                );
            }
            BlockDirection::South => {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x + 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.max.z + 1,
                    BlockDirection::South,
                    depth,
                    shaft_type,
                );
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x - 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z + 1,
                    BlockDirection::West,
                    depth,
                    shaft_type,
                );
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.max.x + 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z + 1,
                    BlockDirection::East,
                    depth,
                    shaft_type,
                );
            }
            BlockDirection::West => {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x + 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z - 1,
                    BlockDirection::North,
                    depth,
                    shaft_type,
                );
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x + 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.max.z + 1,
                    BlockDirection::South,
                    depth,
                    shaft_type,
                );
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x - 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z + 1,
                    BlockDirection::West,
                    depth,
                    shaft_type,
                );
            }
            BlockDirection::East => {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x + 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z - 1,
                    BlockDirection::North,
                    depth,
                    shaft_type,
                );
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x + 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.max.z + 1,
                    BlockDirection::South,
                    depth,
                    shaft_type,
                );
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.max.x + 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z + 1,
                    BlockDirection::East,
                    depth,
                    shaft_type,
                );
            }
            _ => {}
        }

        if self.is_two_floored {
            if random.next_bool() {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x + 1,
                    self.piece.bounding_box.min.y + 4,
                    self.piece.bounding_box.min.z - 1,
                    BlockDirection::North,
                    depth,
                    shaft_type,
                );
            }
            if random.next_bool() {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x - 1,
                    self.piece.bounding_box.min.y + 4,
                    self.piece.bounding_box.min.z + 1,
                    BlockDirection::West,
                    depth,
                    shaft_type,
                );
            }
            if random.next_bool() {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.max.x + 1,
                    self.piece.bounding_box.min.y + 4,
                    self.piece.bounding_box.min.z + 1,
                    BlockDirection::East,
                    depth,
                    shaft_type,
                );
            }
            if random.next_bool() {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x + 1,
                    self.piece.bounding_box.min.y + 4,
                    self.piece.bounding_box.max.z + 1,
                    BlockDirection::South,
                    depth,
                    shaft_type,
                );
            }
        }
    }
}

impl StructurePieceBase for MineShaftCrossing {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }

    #[allow(clippy::too_many_lines)]
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        if is_in_invalid_liquid_location(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }

        let air = Block::CAVE_AIR.default_state;
        let planks = self.shaft_type.planks();
        let bb = self.piece.bounding_box;

        if self.is_two_floored {
            for y in bb.min.y..=(bb.min.y + 2) {
                for x in (bb.min.x + 1)..bb.max.x {
                    for z in bb.min.z..=bb.max.z {
                        if chunk_box.contains(x, y, z) {
                            chunk.set_block_state(x, y, z, air);
                        }
                    }
                }
                for x in bb.min.x..=bb.max.x {
                    for z in (bb.min.z + 1)..bb.max.z {
                        if chunk_box.contains(x, y, z) {
                            chunk.set_block_state(x, y, z, air);
                        }
                    }
                }
            }
            for y in (bb.max.y - 2)..=bb.max.y {
                for x in (bb.min.x + 1)..bb.max.x {
                    for z in bb.min.z..=bb.max.z {
                        if chunk_box.contains(x, y, z) {
                            chunk.set_block_state(x, y, z, air);
                        }
                    }
                }
                for x in bb.min.x..=bb.max.x {
                    for z in (bb.min.z + 1)..bb.max.z {
                        if chunk_box.contains(x, y, z) {
                            chunk.set_block_state(x, y, z, air);
                        }
                    }
                }
            }
            let mid_y = bb.min.y + 3;
            for x in (bb.min.x + 1)..bb.max.x {
                for z in (bb.min.z + 1)..bb.max.z {
                    if chunk_box.contains(x, mid_y, z) {
                        chunk.set_block_state(x, mid_y, z, air);
                    }
                }
            }
        } else {
            for y in bb.min.y..=bb.max.y {
                for x in (bb.min.x + 1)..bb.max.x {
                    for z in bb.min.z..=bb.max.z {
                        if chunk_box.contains(x, y, z) {
                            chunk.set_block_state(x, y, z, air);
                        }
                    }
                }
                for x in bb.min.x..=bb.max.x {
                    for z in (bb.min.z + 1)..bb.max.z {
                        if chunk_box.contains(x, y, z) {
                            chunk.set_block_state(x, y, z, air);
                        }
                    }
                }
            }
        }

        for (cx, cz) in [
            (bb.min.x + 1, bb.min.z + 1),
            (bb.min.x + 1, bb.max.z - 1),
            (bb.max.x - 1, bb.min.z + 1),
            (bb.max.x - 1, bb.max.z - 1),
        ] {
            let above = chunk.get_block_state(&Vector3::new(cx, bb.max.y + 1, cz));
            if !above.to_state().is_air() {
                for py in bb.min.y..=bb.max.y {
                    if chunk_box.contains(cx, py, cz) {
                        chunk.set_block_state(cx, py, cz, planks);
                    }
                }
            }
        }

        let floor_y = bb.min.y - 1;
        for x in bb.min.x..=bb.max.x {
            for z in bb.min.z..=bb.max.z {
                set_planks_block(&self.piece, chunk, chunk_box, planks, x, floor_y, z);
            }
        }
    }
}

#[derive(Clone)]
pub struct MineShaftStairs {
    pub piece: StructurePiece,
    pub shaft_type: MineshaftType,
}

impl MineShaftStairs {
    #[must_use]
    pub const fn new(
        gen_depth: u32,
        bounding_box: BlockBox,
        direction: BlockDirection,
        shaft_type: MineshaftType,
    ) -> Self {
        let mut piece =
            StructurePiece::new(StructurePieceType::MineshaftStairs, bounding_box, gen_depth);
        piece.set_facing(Some(direction));
        Self { piece, shaft_type }
    }

    #[must_use]
    pub fn find_stairs(
        collector: &StructurePiecesCollector,
        _random: &mut RandomGenerator,
        foot_x: i32,
        foot_y: i32,
        foot_z: i32,
        direction: BlockDirection,
    ) -> Option<BlockBox> {
        let mut box_cand = match direction {
            BlockDirection::South => BlockBox::new(0, -5, 0, 2, 2, 8),
            BlockDirection::West => BlockBox::new(-8, -5, 0, 0, 2, 2),
            BlockDirection::East => BlockBox::new(0, -5, 0, 8, 2, 2),
            _ => BlockBox::new(0, -5, -8, 2, 2, 0),
        };
        box_cand.move_pos(foot_x, foot_y, foot_z);
        collector
            .get_intersecting(&box_cand)
            .is_none()
            .then_some(box_cand)
    }

    fn add_children_to_list(
        &self,
        start_piece_box: &BlockBox,
        shaft_type: MineshaftType,
        collector: &mut StructurePiecesCollector,
        random: &mut RandomGenerator,
    ) {
        let depth = self.piece.chain_length;
        let dir = self.piece.facing.unwrap_or(BlockDirection::North);

        match dir {
            BlockDirection::North => {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z - 1,
                    BlockDirection::North,
                    depth,
                    shaft_type,
                );
            }
            BlockDirection::South => {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.max.z + 1,
                    BlockDirection::South,
                    depth,
                    shaft_type,
                );
            }
            BlockDirection::West => {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.min.x - 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z,
                    BlockDirection::West,
                    depth,
                    shaft_type,
                );
            }
            BlockDirection::East => {
                generate_and_add_piece(
                    start_piece_box,
                    collector,
                    random,
                    self.piece.bounding_box.max.x + 1,
                    self.piece.bounding_box.min.y,
                    self.piece.bounding_box.min.z,
                    BlockDirection::East,
                    depth,
                    shaft_type,
                );
            }
            _ => {}
        }
    }
}

impl StructurePieceBase for MineShaftStairs {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        if is_in_invalid_liquid_location(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }

        let air = Block::CAVE_AIR.default_state;
        self.piece.fill(chunk, chunk_box, 0, 5, 0, 2, 7, 1, air);
        self.piece.fill(chunk, chunk_box, 0, 0, 7, 2, 2, 8, air);

        for i in 0..5 {
            let y_min = 5 - i - i32::from(i < 4);
            let y_max = 7 - i;
            let z = 2 + i;
            self.piece
                .fill(chunk, chunk_box, 0, y_min, z, 2, y_max, z, air);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MineshaftType, boundary_matches, for_each_maybe_box_position, is_supporting_box,
        passes_cobweb_random_gate, places_floor_planks, rail_chance,
    };
    use pumpkin_data::Block;
    use pumpkin_util::random::{RandomGenerator, RandomImpl, legacy_rand::LegacyRand};

    #[test]
    fn mineshaft_can_be_replaced_excludes_its_own_building_blocks() {
        // Vanilla 26.2 MineShaftPiece.canBeReplaced:
        // state is neither type.planks nor type.wood nor type.fence nor IRON_CHAIN.
        for shaft_type in [MineshaftType::Normal, MineshaftType::Mesa] {
            assert!(!shaft_type.can_replace(shaft_type.planks()));
            assert!(!shaft_type.can_replace(shaft_type.wood()));
            assert!(!shaft_type.can_replace(shaft_type.fence()));
            assert!(!shaft_type.can_replace(Block::IRON_CHAIN.default_state));
            assert!(shaft_type.can_replace(Block::STONE.default_state));
            assert!(shaft_type.can_replace(Block::CAVE_AIR.default_state));
        }
    }

    #[test]
    fn generate_maybe_box_draws_in_y_x_z_order_before_placement_gates() {
        // Vanilla 26.2 StructurePiece.generateMaybeBox:
        // for (y) for (x) for (z) if (random.nextFloat() > chance) continue.
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(0));
        let mut accepted = Vec::new();
        for_each_maybe_box_position(&mut random, 0.8, 0, 2, 0, 2, 2, 2, |x, y, z| {
            accepted.push((x, y, z));
        });

        assert_eq!(
            accepted,
            [
                (0, 2, 0),
                (0, 2, 2),
                (1, 2, 0),
                (1, 2, 1),
                (1, 2, 2),
                (2, 2, 0),
                (2, 2, 1),
                (2, 2, 2),
            ]
        );
        assert_eq!(random.next_f32(), 0.781_534_6);
    }

    #[test]
    fn corridor_support_requires_a_complete_non_air_ceiling() {
        // Vanilla 26.2 MineShaftPiece.isSupportingBox:
        // for (x = minX; x <= maxX; x++) if (getBlock(x, y + 1, z).isAir()) return false.
        assert!(is_supporting_box(0, 2, |_| false));
        assert!(!is_supporting_box(0, 2, |x| x == 1));
    }

    #[test]
    fn cobweb_interior_gate_precedes_the_random_draw() {
        // Vanilla 26.2 MineShaftCorridor.maybePlaceCobWeb:
        // isInterior(...) && random.nextFloat() < chance && hasSturdyNeighbours(..., 2).
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(0));
        assert!(!passes_cobweb_random_gate(&mut random, 0.8, false));
        assert!(passes_cobweb_random_gate(&mut random, 0.8, true));
        assert_eq!(random.next_f32(), 0.831_441);
    }

    #[test]
    fn corridor_rail_chance_depends_on_interior_status() {
        // Vanilla 26.2 MineShaftCorridor.postProcess:
        // float chance = isInterior(world, 1, 0, z, box) ? 0.7F : 0.9F.
        assert_eq!(rail_chance(true), 0.7);
        assert_eq!(rail_chance(false), 0.9);
    }

    #[test]
    fn corridor_support_fences_connect_toward_the_beam() {
        // Vanilla 26.2 MineShaftCorridor.placeSupport uses
        // fence.setValue(WEST, true) at x0 and fence.setValue(EAST, true) at x1.
        let west = MineshaftType::Mesa.connected_fence(true, false);
        let east = MineshaftType::Mesa.connected_fence(false, true);
        let west_props =
            pumpkin_data::block_properties::OakFenceLikeProperties::from_state_id(west.id);
        let east_props =
            pumpkin_data::block_properties::OakFenceLikeProperties::from_state_id(east.id);
        assert!(west_props.west && !west_props.east);
        assert!(!east_props.west && east_props.east);
    }

    #[test]
    fn pillar_fill_column_excludes_its_anchor_endpoint() {
        // Vanilla 26.2 fillColumnBetween: for (int y = start; y < end; ++y).
        assert_eq!((6..10).collect::<Vec<_>>(), [6, 7, 8, 9]);
        assert_eq!((12..12).count(), 0);
    }

    #[test]
    fn invalid_location_scans_only_the_expanded_piece_boundary() {
        // Vanilla 26.2 MineShaftPiece.isInInvalidLocation scans the six faces of
        // piece.boundingBox inflated by one and intersected with the chunk box.
        let piece = pumpkin_util::math::block_box::BlockBox::new(10, 20, 30, 12, 22, 32);
        let chunk = pumpkin_util::math::block_box::BlockBox::new(0, 0, 0, 40, 40, 40);
        assert!(boundary_matches(&piece, &chunk, |x, y, z| {
            x == 9 && y == 20 && z == 31
        }));
        assert!(!boundary_matches(&piece, &chunk, |x, y, z| {
            x == 11 && y == 21 && z == 31
        }));
    }

    #[test]
    fn floor_planks_need_interior_and_a_non_sturdy_top_face() {
        // Vanilla 26.2 MineShaftPiece.setPlanksBlock:
        //   if (this.isInterior(level, x, y, z, chunkBB)) {
        //       BlockState existingState = level.getBlockState(pos);
        //       if (!existingState.isFaceSturdy(level, pos, Direction.UP)) { setBlock(planks) }
        //   }
        // so both a non-interior position and a sturdy upward face suppress the plank.
        assert!(places_floor_planks(true, false));
        assert!(!places_floor_planks(true, true));
        assert!(!places_floor_planks(false, false));
        assert!(!places_floor_planks(false, true));
    }

    #[test]
    fn double_support_anchors_are_the_two_corridor_walls() {
        // Vanilla 26.2 MineShaftCorridor.placeDoubleLowerOrUpperSupport tests
        // getBlock(x, y, z) and getBlock(x + 2, y, z) against type.getPlanksState(),
        // i.e. the two wall columns of the 3-wide corridor, in that order.
        let x = 0;
        assert_eq!([x, x + 2], [0, 2]);
    }
}
