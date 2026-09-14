use std::sync::Arc;

use pumpkin_data::{
    Block, BlockState,
    block_properties::{HorizontalFacing, WallTorchLikeProperties},
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

        let mut collector = StructurePiecesCollector::default();
        let start_piece_box = start_room.piece.bounding_box;

        let mut children = Vec::new();
        start_room.add_children_to_list(
            &start_piece_box,
            shaft_type,
            &collector,
            &mut context.random,
            &mut children,
        );

        collector.add_piece(Box::new(start_room));

        while !children.is_empty() {
            let next_piece = children.remove(0);
            next_piece.build_children(
                &start_piece_box,
                shaft_type,
                &collector,
                &mut context.random,
                &mut children,
            );
            collector.add_piece(next_piece.into_piece_base());
        }

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
        collector: &StructurePiecesCollector,
        random: &mut RandomGenerator,
        children: &mut Vec<Self>,
    ) {
        match self {
            Self::Corridor(c) => {
                c.add_children_to_list(start_piece_box, shaft_type, collector, random, children);
            }
            Self::Crossing(cr) => {
                cr.add_children_to_list(start_piece_box, shaft_type, collector, random, children);
            }
            Self::Stairs(s) => {
                s.add_children_to_list(start_piece_box, shaft_type, collector, random, children);
            }
        }
    }

    fn into_piece_base(self) -> Box<dyn StructurePieceBase> {
        match self {
            Self::Corridor(c) => Box::new(c),
            Self::Crossing(cr) => Box::new(cr),
            Self::Stairs(s) => Box::new(s),
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

#[expect(clippy::too_many_arguments)]
fn generate_and_add_piece(
    start_piece_box: &BlockBox,
    collector: &StructurePiecesCollector,
    random: &mut RandomGenerator,
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: BlockDirection,
    depth: u32,
    shaft_type: MineshaftType,
    children: &mut Vec<GeneratedPiece>,
) {
    if depth <= 8
        && (foot_x - start_piece_box.min.x).abs() <= 80
        && (foot_z - start_piece_box.min.z).abs() <= 80
        && let Some(new_piece) = create_random_shaft_piece(
            collector,
            random,
            foot_x,
            foot_y,
            foot_z,
            direction,
            depth + 1,
            shaft_type,
        )
    {
        children.push(new_piece);
    }
}

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
        collector: &StructurePiecesCollector,
        random: &mut RandomGenerator,
        children: &mut Vec<GeneratedPiece>,
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

            let prev_len = children.len();
            generate_and_add_piece(
                start_piece_box,
                collector,
                random,
                child_x,
                child_y,
                child_z,
                BlockDirection::North,
                depth,
                shaft_type,
                children,
            );
            if children.len() > prev_len {
                let bb = match &children[prev_len] {
                    GeneratedPiece::Corridor(c) => c.piece.bounding_box,
                    GeneratedPiece::Crossing(c) => c.piece.bounding_box,
                    GeneratedPiece::Stairs(s) => s.piece.bounding_box,
                };
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

            let prev_len = children.len();
            generate_and_add_piece(
                start_piece_box,
                collector,
                random,
                child_x,
                child_y,
                child_z,
                BlockDirection::South,
                depth,
                shaft_type,
                children,
            );
            if children.len() > prev_len {
                let bb = match &children[prev_len] {
                    GeneratedPiece::Corridor(c) => c.piece.bounding_box,
                    GeneratedPiece::Crossing(c) => c.piece.bounding_box,
                    GeneratedPiece::Stairs(s) => s.piece.bounding_box,
                };
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

            let prev_len = children.len();
            generate_and_add_piece(
                start_piece_box,
                collector,
                random,
                child_x,
                child_y,
                child_z,
                BlockDirection::West,
                depth,
                shaft_type,
                children,
            );
            if children.len() > prev_len {
                let bb = match &children[prev_len] {
                    GeneratedPiece::Corridor(c) => c.piece.bounding_box,
                    GeneratedPiece::Crossing(c) => c.piece.bounding_box,
                    GeneratedPiece::Stairs(s) => s.piece.bounding_box,
                };
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

            let prev_len = children.len();
            generate_and_add_piece(
                start_piece_box,
                collector,
                random,
                child_x,
                child_y,
                child_z,
                BlockDirection::East,
                depth,
                shaft_type,
                children,
            );
            if children.len() > prev_len {
                let bb = match &children[prev_len] {
                    GeneratedPiece::Corridor(c) => c.piece.bounding_box,
                    GeneratedPiece::Crossing(c) => c.piece.bounding_box,
                    GeneratedPiece::Stairs(s) => s.piece.bounding_box,
                };
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
        collector: &StructurePiecesCollector,
        random: &mut RandomGenerator,
        children: &mut Vec<GeneratedPiece>,
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
                        children,
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
                        children,
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
                        children,
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
                        children,
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
                        children,
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
                        children,
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
                        children,
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
                        children,
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
                        children,
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
                        children,
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
                        children,
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
                        children,
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
                            children,
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
                            children,
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
                            children,
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
                            children,
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
        let planks = self.shaft_type.planks();
        let fence = self.shaft_type.fence();

        self.piece
            .fill(chunk, chunk_box, x0, y0, z, x0, y1 - 1, z, fence);
        self.piece
            .fill(chunk, chunk_box, x1, y0, z, x1, y1 - 1, z, fence);

        if random.next_bounded_i32(4) == 0 {
            self.piece
                .fill(chunk, chunk_box, x0, y1, z, x0, y1, z, planks);
            self.piece
                .fill(chunk, chunk_box, x1, y1, z, x1, y1, z, planks);
        } else {
            self.piece
                .fill(chunk, chunk_box, x0, y1, z, x1, y1, z, planks);

            let mut props_s = WallTorchLikeProperties::default(&Block::WALL_TORCH);
            props_s.facing = HorizontalFacing::South;
            let torch_s = BlockState::from_id(props_s.to_state_id(&Block::WALL_TORCH));

            let mut props_n = WallTorchLikeProperties::default(&Block::WALL_TORCH);
            props_n.facing = HorizontalFacing::North;
            let torch_n = BlockState::from_id(props_n.to_state_id(&Block::WALL_TORCH));

            if random.next_f32() < 0.05 {
                self.piece
                    .add_block(chunk, torch_s, x0 + 1, y1, z - 1, chunk_box);
            }
            if random.next_f32() < 0.05 {
                self.piece
                    .add_block(chunk, torch_n, x0 + 1, y1, z + 1, chunk_box);
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
                    for py in (below_y + 1)..=world_y {
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
                    for py in (world_y + 2)..=above_y {
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
        let air = Block::CAVE_AIR.default_state;
        let length = self.num_sections * 5 - 1;
        let planks = self.shaft_type.planks();

        self.piece
            .fill(chunk, chunk_box, 0, 0, 0, 2, 1, length, air);

        for z in 0..=length {
            for x in 0..=2 {
                if random.next_f32() < 0.8 {
                    self.piece.add_block(chunk, air, x, 2, z, chunk_box);
                }
                if self.spider_corridor && random.next_f32() < 0.6 {
                    self.piece
                        .add_block(chunk, Block::COBWEB.default_state, x, 0, z, chunk_box);
                }
            }
        }

        for section in 0..self.num_sections {
            let z = 2 + section * 5;
            self.place_support(chunk, chunk_box, 0, 0, z, 2, 2, random);

            for (cx, cy, cz, prob) in [
                (0, 2, z - 1, 0.1f32),
                (2, 2, z - 1, 0.1),
                (0, 2, z + 1, 0.1),
                (2, 2, z + 1, 0.1),
                (0, 2, z - 2, 0.05),
                (2, 2, z - 2, 0.05),
                (0, 2, z + 2, 0.05),
                (2, 2, z + 2, 0.05),
            ] {
                if random.next_f32() < prob {
                    self.piece
                        .add_block(chunk, Block::COBWEB.default_state, cx, cy, cz, chunk_box);
                }
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
                let world_pos = self.piece.offset_pos(x, -1, z);
                if chunk_box.contains_pos(&world_pos) {
                    let below = chunk.get_block_state(&world_pos);
                    if below.to_state().is_air() {
                        chunk.set_block_state(world_pos.x, world_pos.y, world_pos.z, planks);
                    }
                }
            }
        }

        self.fill_pillar_down_or_chain_up(chunk, 0, -1, 2, chunk_box);
        self.fill_pillar_down_or_chain_up(chunk, 2, -1, 2, chunk_box);
        if self.num_sections > 1 {
            let last_support = length - 2;
            self.fill_pillar_down_or_chain_up(chunk, 0, -1, last_support, chunk_box);
            self.fill_pillar_down_or_chain_up(chunk, 2, -1, last_support, chunk_box);
        }

        if self.has_rails {
            let rail = Block::RAIL.default_state;
            for z in 0..=length {
                let floor_pos = self.piece.offset_pos(1, -1, z);
                if chunk_box.contains_pos(&floor_pos) {
                    let floor_state = chunk.get_block_state(&floor_pos);
                    if !floor_state.to_state().is_air() && random.next_f32() < 0.7 {
                        self.piece.add_block(chunk, rail, 1, 0, z, chunk_box);
                    }
                }
            }
        }
    }
}

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
        collector: &StructurePiecesCollector,
        random: &mut RandomGenerator,
        children: &mut Vec<GeneratedPiece>,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
                if chunk_box.contains(x, floor_y, z) {
                    let state = chunk.get_block_state(&Vector3::new(x, floor_y, z));
                    if state.to_state().is_air() {
                        chunk.set_block_state(x, floor_y, z, planks);
                    }
                }
            }
        }
    }
}

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
        collector: &StructurePiecesCollector,
        random: &mut RandomGenerator,
        children: &mut Vec<GeneratedPiece>,
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
                    children,
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
                    children,
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
                    children,
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
                    children,
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
