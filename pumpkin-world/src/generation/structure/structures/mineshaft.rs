//! Mineshaft structure generator (normal + mesa variants).
//!
//! Port of vanilla `MineshaftPieces` / `MineshaftStructure`.
//! Procedural piece-based structure: a room at Y=50 branches into corridors,
//! crossings and stairs. Normal uses oak, mesa uses dark oak.

use std::sync::Arc;

use pumpkin_data::{
    Block, BlockState,
    block_properties::blocks_movement,
    tag::{Taggable, WorldgenBiome},
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::{
    BlockDirection, HeightMap,
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

const MAX_DEPTH: u32 = 8;
const BOUND: i32 = 80;
const MAGIC_START_Y: i32 = 50;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MineshaftType {
    Normal,
    Mesa,
}

impl MineshaftType {
    const fn planks(self) -> &'static BlockState {
        match self {
            Self::Normal => Block::OAK_PLANKS.default_state,
            Self::Mesa => Block::DARK_OAK_PLANKS.default_state,
        }
    }
    const fn log(self) -> &'static BlockState {
        match self {
            Self::Normal => Block::OAK_LOG.default_state,
            Self::Mesa => Block::DARK_OAK_LOG.default_state,
        }
    }
    const fn fence(self) -> &'static BlockState {
        match self {
            Self::Normal => Block::OAK_FENCE.default_state,
            Self::Mesa => Block::DARK_OAK_FENCE.default_state,
        }
    }
}

#[derive(Clone, Copy)]
struct Attachment {
    x: i32,
    y: i32,
    z: i32,
    facing: BlockDirection,
    depth: u32,
}

pub struct MineshaftGenerator {
    pub mineshaft_type: MineshaftType,
}

impl StructureGenerator for MineshaftGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        // Vanilla consumes a double here (leftover from the old probability gate).
        let _ = context.random.next_f64();

        let west = context.chunk_x * 16 + 2;
        let north = context.chunk_z * 16 + 2;
        let mut collector = StructurePiecesCollector::default();

        // Create the starting room (vanilla: fixed Y=50, random width/depth/height).
        let room_box = BlockBox::new(
            west,
            MAGIC_START_Y,
            north,
            west + 7 + context.random.next_bounded_i32(6),
            54 + context.random.next_bounded_i32(6),
            north + 7 + context.random.next_bounded_i32(6),
        );
        let start_min_x = room_box.min.x;
        let start_min_z = room_box.min.z;

        let mut room = MineshaftRoomPiece {
            piece: StructurePiece::new(StructurePieceType::MineshaftRoom, room_box, 0),
            mineshaft_type: self.mineshaft_type,
            child_entrance_boxes: Vec::new(),
        };

        // The room must participate in intersection checks while its children are
        // assembled. Replace the provisional copy afterwards with its entrances.
        collector.add_piece(Box::new(room.clone()));
        room.add_children(
            start_min_x,
            start_min_z,
            &mut context.random,
            &mut collector,
        );
        collector.pieces[0] = Box::new(room);

        if collector.pieces.is_empty() {
            return None;
        }

        let vertical_offset = match self.mineshaft_type {
            MineshaftType::Normal => {
                collector.shift_into(context.sea_level, context.min_y, &mut context.random, 10)
            }
            MineshaftType::Mesa => {
                let bounds = collector.get_bounding_box();
                let center_x = block_box_center(bounds.min.x, bounds.max.x);
                let center_z = block_box_center(bounds.min.z, bounds.max.z);
                let surface_y = context
                    .height_sampler
                    .as_deref_mut()
                    .map_or(context.sea_level, |sampler| {
                        sampler.estimate_height(center_x, center_z)
                    });
                let target_y = if surface_y <= context.sea_level {
                    context.sea_level
                } else {
                    context
                        .random
                        .next_inbetween_i32(context.sea_level, surface_y)
                };
                let offset = target_y - block_box_center(bounds.min.y, bounds.max.y);
                collector.shift(offset);
                offset
            }
        };

        Some(StructurePosition {
            start_pos: BlockPos::new(
                context.chunk_x * 16 + 8,
                MAGIC_START_Y + vertical_offset,
                context.chunk_z * 16,
            ),
            collector: Arc::new(collector.into()),
        })
    }
}

fn generate_and_add_piece(
    attachment: Attachment,
    start_min_x: i32,
    start_min_z: i32,
    mineshaft_type: MineshaftType,
    random: &mut RandomGenerator,
    collector: &mut StructurePiecesCollector,
) -> Option<BlockBox> {
    if attachment.depth > MAX_DEPTH
        || (attachment.x - start_min_x).abs() > BOUND
        || (attachment.z - start_min_z).abs() > BOUND
    {
        return None;
    }

    let roll = random.next_bounded_i32(100);
    if roll >= 80 {
        let piece = MineshaftCrossingPiece::create(&attachment, mineshaft_type, random, collector)?;
        let bounding_box = piece.piece.bounding_box;
        collector.add_piece(Box::new(piece.clone()));
        piece.add_children(start_min_x, start_min_z, random, collector);
        Some(bounding_box)
    } else if roll >= 70 {
        let piece = MineshaftStairsPiece::create(&attachment, mineshaft_type, collector)?;
        let bounding_box = piece.piece.bounding_box;
        collector.add_piece(Box::new(piece.clone()));
        piece.add_children(start_min_x, start_min_z, random, collector);
        Some(bounding_box)
    } else {
        let piece = MineshaftCorridorPiece::create(&attachment, mineshaft_type, random, collector)?;
        let bounding_box = piece.piece.bounding_box;
        collector.add_piece(Box::new(piece.clone()));
        piece.add_children(start_min_x, start_min_z, random, collector);
        Some(bounding_box)
    }
}

// ===========================================================================
// Room
// ===========================================================================

#[derive(Clone)]
struct MineshaftRoomPiece {
    piece: StructurePiece,
    mineshaft_type: MineshaftType,
    child_entrance_boxes: Vec<BlockBox>,
}

impl MineshaftRoomPiece {
    fn add_children(
        &mut self,
        start_min_x: i32,
        start_min_z: i32,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        let room = self.piece.bounding_box;
        let x_span = room.max.x - room.min.x + 1;
        let z_span = room.max.z - room.min.z + 1;
        let height_space = (room.max.y - room.min.y + 1 - 4).max(1);

        for direction in [BlockDirection::North, BlockDirection::South] {
            let mut offset = 0;
            while offset < x_span {
                offset += random.next_bounded_i32(x_span);
                if offset + 3 > x_span {
                    break;
                }
                let z = if direction == BlockDirection::North {
                    room.min.z - 1
                } else {
                    room.max.z + 1
                };
                if let Some(child) = generate_and_add_piece(
                    Attachment {
                        x: room.min.x + offset,
                        y: room.min.y + random.next_bounded_i32(height_space) + 1,
                        z,
                        facing: direction,
                        depth: self.piece.chain_length,
                    },
                    start_min_x,
                    start_min_z,
                    self.mineshaft_type,
                    random,
                    collector,
                ) {
                    let (min_z, max_z) = if direction == BlockDirection::North {
                        (room.min.z, room.min.z + 1)
                    } else {
                        (room.max.z - 1, room.max.z)
                    };
                    self.child_entrance_boxes.push(BlockBox::new(
                        child.min.x,
                        child.min.y,
                        min_z,
                        child.max.x,
                        child.max.y,
                        max_z,
                    ));
                }
                offset += 4;
            }
        }

        for direction in [BlockDirection::West, BlockDirection::East] {
            let mut offset = 0;
            while offset < z_span {
                offset += random.next_bounded_i32(z_span);
                if offset + 3 > z_span {
                    break;
                }
                let x = if direction == BlockDirection::West {
                    room.min.x - 1
                } else {
                    room.max.x + 1
                };
                if let Some(child) = generate_and_add_piece(
                    Attachment {
                        x,
                        y: room.min.y + random.next_bounded_i32(height_space) + 1,
                        z: room.min.z + offset,
                        facing: direction,
                        depth: self.piece.chain_length,
                    },
                    start_min_x,
                    start_min_z,
                    self.mineshaft_type,
                    random,
                    collector,
                ) {
                    let (min_x, max_x) = if direction == BlockDirection::West {
                        (room.min.x, room.min.x + 1)
                    } else {
                        (room.max.x - 1, room.max.x)
                    };
                    self.child_entrance_boxes.push(BlockBox::new(
                        min_x,
                        child.min.y,
                        child.min.z,
                        max_x,
                        child.max.y,
                        child.max.z,
                    ));
                }
                offset += 4;
            }
        }
    }
}

impl StructurePieceBase for MineshaftRoomPiece {
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
        for entrance in &mut self.child_entrance_boxes {
            entrance.move_pos(x, y, z);
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
        if is_in_invalid_location(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }
        let bb = self.piece.bounding_box;
        let cave_air = Block::CAVE_AIR.default_state;
        fill_world_box(
            chunk,
            chunk_box,
            &BlockBox::new(
                bb.min.x,
                bb.min.y + 1,
                bb.min.z,
                bb.max.x,
                (bb.min.y + 3).min(bb.max.y),
                bb.max.z,
            ),
            cave_air,
            self.mineshaft_type,
        );
        for entrance in &self.child_entrance_boxes {
            fill_world_box(
                chunk,
                chunk_box,
                &BlockBox::new(
                    entrance.min.x,
                    entrance.max.y - 2,
                    entrance.min.z,
                    entrance.max.x,
                    entrance.max.y,
                    entrance.max.z,
                ),
                cave_air,
                self.mineshaft_type,
            );
        }
        fill_upper_half_sphere(
            chunk,
            chunk_box,
            &BlockBox::new(
                bb.min.x,
                bb.min.y + 4,
                bb.min.z,
                bb.max.x,
                bb.max.y,
                bb.max.z,
            ),
            cave_air,
            self.mineshaft_type,
        );
    }
}

// ===========================================================================
// Corridor
// ===========================================================================

#[derive(Clone)]
struct MineshaftCorridorPiece {
    piece: StructurePiece,
    mineshaft_type: MineshaftType,
    num_sections: i32,
    has_rails: bool,
    spider_corridor: bool,
    has_placed_spider: bool,
}

impl MineshaftCorridorPiece {
    fn create(
        att: &Attachment,
        mineshaft_type: MineshaftType,
        random: &mut RandomGenerator,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let mut corridor_length = random.next_bounded_i32(3) + 2;
        while corridor_length > 0 {
            let block_length = corridor_length * 5;
            let bbox = BlockBox::rotated(
                att.x,
                att.y,
                att.z,
                0,
                0,
                0,
                3,
                3,
                block_length,
                &att.facing,
            );
            if collector.get_intersecting(&bbox).is_none() {
                let mut piece =
                    StructurePiece::new(StructurePieceType::MineshaftCorridor, bbox, att.depth + 1);
                piece.set_facing(Some(att.facing));
                let has_rails = random.next_bounded_i32(3) == 0;
                let spider_corridor = !has_rails && random.next_bounded_i32(23) == 0;
                return Some(Self {
                    piece,
                    mineshaft_type,
                    num_sections: corridor_length,
                    has_rails,
                    spider_corridor,
                    has_placed_spider: false,
                });
            }
            corridor_length -= 1;
        }
        None
    }
}

impl StructurePieceBase for MineshaftCorridorPiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }
    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }
    #[expect(clippy::too_many_lines)]
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        if is_in_invalid_location(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }
        let planks = self.mineshaft_type.planks();
        let fence = self.mineshaft_type.fence();
        let air = Block::CAVE_AIR.default_state;
        let cobweb = Block::COBWEB.default_state;
        let length = self.num_sections * 5 - 1;

        // Clear the tunnel and irregular ceiling.
        for z in 0..=length {
            for x in 0..=2 {
                add_mineshaft_block(
                    chunk,
                    &self.piece,
                    self.mineshaft_type,
                    air,
                    x,
                    0,
                    z,
                    chunk_box,
                );
                add_mineshaft_block(
                    chunk,
                    &self.piece,
                    self.mineshaft_type,
                    air,
                    x,
                    1,
                    z,
                    chunk_box,
                );
            }
        }
        for x in 0..=2 {
            for z in 0..=length {
                if random.next_f32() <= 0.8 {
                    add_mineshaft_block(
                        chunk,
                        &self.piece,
                        self.mineshaft_type,
                        air,
                        x,
                        2,
                        z,
                        chunk_box,
                    );
                }
            }
        }

        if self.spider_corridor {
            for y in 0..=1 {
                for x in 0..=2 {
                    for z in 0..=length {
                        if random.next_f32() <= 0.6
                            && is_interior(chunk, &self.piece, x, y, z, chunk_box)
                        {
                            add_mineshaft_block(
                                chunk,
                                &self.piece,
                                self.mineshaft_type,
                                cobweb,
                                x,
                                y,
                                z,
                                chunk_box,
                            );
                        }
                    }
                }
            }
        }

        let left_fence_properties =
            Block::from_state_id(fence.id).from_properties(&[("west", "true")]);
        let left_fence =
            BlockState::from_id(left_fence_properties.to_state_id(Block::from_state_id(fence.id)));
        let right_fence_properties =
            Block::from_state_id(fence.id).from_properties(&[("east", "true")]);
        let right_fence =
            BlockState::from_id(right_fence_properties.to_state_id(Block::from_state_id(fence.id)));

        for section in 0..self.num_sections {
            let z = 2 + section * 5;
            if is_supporting_box(chunk, &self.piece, 0, 2, 2, z, chunk_box) {
                for y in 0..=1 {
                    place_mineshaft_block(
                        chunk,
                        &self.piece,
                        self.mineshaft_type,
                        block_registry,
                        left_fence,
                        0,
                        y,
                        z,
                        chunk_box,
                    );
                    place_mineshaft_block(
                        chunk,
                        &self.piece,
                        self.mineshaft_type,
                        block_registry,
                        right_fence,
                        2,
                        y,
                        z,
                        chunk_box,
                    );
                }
                if random.next_bounded_i32(4) == 0 {
                    add_mineshaft_block(
                        chunk,
                        &self.piece,
                        self.mineshaft_type,
                        planks,
                        0,
                        2,
                        z,
                        chunk_box,
                    );
                    add_mineshaft_block(
                        chunk,
                        &self.piece,
                        self.mineshaft_type,
                        planks,
                        2,
                        2,
                        z,
                        chunk_box,
                    );
                } else {
                    for x in 0..=2 {
                        add_mineshaft_block(
                            chunk,
                            &self.piece,
                            self.mineshaft_type,
                            planks,
                            x,
                            2,
                            z,
                            chunk_box,
                        );
                    }
                    maybe_place_wall_torch(
                        chunk,
                        &self.piece,
                        block_registry,
                        random,
                        1,
                        2,
                        z - 1,
                        "south",
                        chunk_box,
                        self.mineshaft_type,
                    );
                    maybe_place_wall_torch(
                        chunk,
                        &self.piece,
                        block_registry,
                        random,
                        1,
                        2,
                        z + 1,
                        "north",
                        chunk_box,
                        self.mineshaft_type,
                    );
                }
            }

            for &(chance, dz) in &[(0.1, -1), (0.1, 1), (0.05, -2), (0.05, 2)] {
                for x in [0, 2] {
                    maybe_place_cobweb(
                        chunk,
                        &self.piece,
                        self.mineshaft_type,
                        random,
                        chance,
                        x,
                        2,
                        z + dz,
                        chunk_box,
                    );
                }
            }
            // Chest minecarts on a randomly oriented rail.
            if random.next_bounded_i32(100) == 0 {
                place_loot_minecart(chunk, &self.piece, 2, 0, z - 1, chunk_box, random);
            }
            if random.next_bounded_i32(100) == 0 {
                place_loot_minecart(chunk, &self.piece, 0, 0, z + 1, chunk_box, random);
            }
            // Cave-spider spawner.
            if self.spider_corridor && !self.has_placed_spider {
                let spawner_z = z - 1 + random.next_bounded_i32(3);
                if (0..=length).contains(&spawner_z)
                    && is_interior(chunk, &self.piece, 1, 0, spawner_z, chunk_box)
                {
                    place_spawner(
                        chunk,
                        &self.piece,
                        1,
                        0,
                        spawner_z,
                        chunk_box,
                        "minecraft:cave_spider",
                    );
                    self.has_placed_spider = true;
                }
            }
        }

        for z in 0..=length {
            for x in 0..=2 {
                set_planks_block(chunk, &self.piece, planks, x, -1, z, chunk_box);
            }
        }

        let log_state = self.mineshaft_type.log();
        place_double_lower_or_upper_support(
            chunk,
            &self.piece,
            self.mineshaft_type,
            log_state,
            -1,
            2,
            chunk_box,
        );
        if self.num_sections > 1 {
            place_double_lower_or_upper_support(
                chunk,
                &self.piece,
                self.mineshaft_type,
                log_state,
                -1,
                length - 2,
                chunk_box,
            );
        }

        if self.has_rails {
            for z in 0..=length {
                let below = self.piece.offset_pos(1, -1, z);
                let below_state = chunk.get_block_state(&below).to_state();
                if chunk_box.contains_pos(&below)
                    && !below_state.is_air()
                    && below_state.is_solid_render()
                    && random.next_f32()
                        < if is_interior(chunk, &self.piece, 1, 0, z, chunk_box) {
                            0.7
                        } else {
                            0.9
                        }
                {
                    place_mineshaft_block(
                        chunk,
                        &self.piece,
                        self.mineshaft_type,
                        block_registry,
                        Block::RAIL.default_state,
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

impl MineshaftCorridorPiece {
    #[expect(clippy::too_many_lines)]
    fn add_children(
        &self,
        start_min_x: i32,
        start_min_z: i32,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        let bb = self.piece.bounding_box;
        let selection = random.next_bounded_i32(4);
        let (x, z, facing) = match self.piece.facing {
            Some(BlockDirection::North) => {
                if selection <= 1 {
                    (bb.min.x, bb.min.z - 1, BlockDirection::North)
                } else if selection == 2 {
                    (bb.min.x - 1, bb.min.z, BlockDirection::West)
                } else {
                    (bb.max.x + 1, bb.min.z, BlockDirection::East)
                }
            }
            Some(BlockDirection::South) => {
                if selection <= 1 {
                    (bb.min.x, bb.max.z + 1, BlockDirection::South)
                } else if selection == 2 {
                    (bb.min.x - 1, bb.max.z - 3, BlockDirection::West)
                } else {
                    (bb.max.x + 1, bb.max.z - 3, BlockDirection::East)
                }
            }
            Some(BlockDirection::West) => {
                if selection <= 1 {
                    (bb.min.x - 1, bb.min.z, BlockDirection::West)
                } else if selection == 2 {
                    (bb.min.x, bb.min.z - 1, BlockDirection::North)
                } else {
                    (bb.min.x, bb.max.z + 1, BlockDirection::South)
                }
            }
            _ => {
                if selection <= 1 {
                    (bb.max.x + 1, bb.min.z, BlockDirection::East)
                } else if selection == 2 {
                    (bb.max.x - 3, bb.min.z - 1, BlockDirection::North)
                } else {
                    (bb.max.x - 3, bb.max.z + 1, BlockDirection::South)
                }
            }
        };
        let _ = generate_and_add_piece(
            Attachment {
                x,
                y: bb.min.y - 1 + random.next_bounded_i32(3),
                z,
                facing,
                depth: self.piece.chain_length,
            },
            start_min_x,
            start_min_z,
            self.mineshaft_type,
            random,
            collector,
        );

        // Side branches deliberately skip another generation depth in vanilla.
        if self.piece.chain_length < MAX_DEPTH {
            let is_ns = matches!(
                self.piece.facing,
                Some(BlockDirection::North | BlockDirection::South)
            );
            if is_ns {
                let mut z = bb.min.z + 3;
                while z + 3 <= bb.max.z {
                    let sel = random.next_bounded_i32(5);
                    if sel == 0 {
                        let _ = generate_and_add_piece(
                            Attachment {
                                x: bb.min.x - 1,
                                y: bb.min.y,
                                z,
                                facing: BlockDirection::West,
                                depth: self.piece.chain_length + 1,
                            },
                            start_min_x,
                            start_min_z,
                            self.mineshaft_type,
                            random,
                            collector,
                        );
                    } else if sel == 1 {
                        let _ = generate_and_add_piece(
                            Attachment {
                                x: bb.max.x + 1,
                                y: bb.min.y,
                                z,
                                facing: BlockDirection::East,
                                depth: self.piece.chain_length + 1,
                            },
                            start_min_x,
                            start_min_z,
                            self.mineshaft_type,
                            random,
                            collector,
                        );
                    }
                    z += 5;
                }
            } else {
                let mut x = bb.min.x + 3;
                while x + 3 <= bb.max.x {
                    let sel = random.next_bounded_i32(5);
                    if sel == 0 {
                        let _ = generate_and_add_piece(
                            Attachment {
                                x,
                                y: bb.min.y,
                                z: bb.min.z - 1,
                                facing: BlockDirection::North,
                                depth: self.piece.chain_length + 1,
                            },
                            start_min_x,
                            start_min_z,
                            self.mineshaft_type,
                            random,
                            collector,
                        );
                    } else if sel == 1 {
                        let _ = generate_and_add_piece(
                            Attachment {
                                x,
                                y: bb.min.y,
                                z: bb.max.z + 1,
                                facing: BlockDirection::South,
                                depth: self.piece.chain_length + 1,
                            },
                            start_min_x,
                            start_min_z,
                            self.mineshaft_type,
                            random,
                            collector,
                        );
                    }
                    x += 5;
                }
            }
        }
    }
}

// ===========================================================================
// Crossing
// ===========================================================================

#[derive(Clone)]
struct MineshaftCrossingPiece {
    piece: StructurePiece,
    mineshaft_type: MineshaftType,
    is_two_floored: bool,
}

impl MineshaftCrossingPiece {
    fn create(
        att: &Attachment,
        mineshaft_type: MineshaftType,
        random: &mut RandomGenerator,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let is_two_floored = random.next_bounded_i32(4) == 0;
        let height = if is_two_floored { 7 } else { 3 };
        let bbox = BlockBox::rotated(att.x, att.y, att.z, -1, 0, 0, 5, height, 5, &att.facing);
        if collector.get_intersecting(&bbox).is_some() {
            return None;
        }
        let mut piece =
            StructurePiece::new(StructurePieceType::MineshaftCrossing, bbox, att.depth + 1);
        piece.set_facing(Some(att.facing));
        Some(Self {
            piece,
            mineshaft_type,
            is_two_floored,
        })
    }
}

impl StructurePieceBase for MineshaftCrossingPiece {
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
        _br: &dyn WorldPortalExt,
        _r: &mut RandomGenerator,
        _s: i64,
        chunk_box: &BlockBox,
    ) {
        if is_in_invalid_location(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }
        let planks = self.mineshaft_type.planks();
        let air = Block::CAVE_AIR.default_state;
        fill_mineshaft(
            chunk,
            &self.piece,
            self.mineshaft_type,
            chunk_box,
            1,
            0,
            0,
            3,
            2,
            4,
            air,
        );
        fill_mineshaft(
            chunk,
            &self.piece,
            self.mineshaft_type,
            chunk_box,
            0,
            0,
            1,
            4,
            2,
            3,
            air,
        );
        if self.is_two_floored {
            fill_mineshaft(
                chunk,
                &self.piece,
                self.mineshaft_type,
                chunk_box,
                1,
                4,
                0,
                3,
                6,
                4,
                air,
            );
            fill_mineshaft(
                chunk,
                &self.piece,
                self.mineshaft_type,
                chunk_box,
                0,
                4,
                1,
                4,
                6,
                3,
                air,
            );
            fill_mineshaft(
                chunk,
                &self.piece,
                self.mineshaft_type,
                chunk_box,
                1,
                3,
                1,
                3,
                3,
                3,
                air,
            );
        }

        let top = if self.is_two_floored { 6 } else { 2 };
        for &(x, z) in &[(1, 1), (1, 3), (3, 1), (3, 3)] {
            let above = self.piece.offset_pos(x, top + 1, z);
            if chunk_box.contains_pos(&above) && !chunk.get_block_state(&above).to_state().is_air()
            {
                fill_mineshaft(
                    chunk,
                    &self.piece,
                    self.mineshaft_type,
                    chunk_box,
                    x,
                    0,
                    z,
                    x,
                    top,
                    z,
                    planks,
                );
            }
        }

        for x in 0..=4 {
            for z in 0..=4 {
                set_planks_block(chunk, &self.piece, planks, x, -1, z, chunk_box);
            }
        }
    }
}

impl MineshaftCrossingPiece {
    fn add_children(
        &self,
        start_min_x: i32,
        start_min_z: i32,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        let bb = self.piece.bounding_box;
        let depth = self.piece.chain_length;
        let direction = self.piece.facing.unwrap_or(BlockDirection::North);
        let exits: &[(BlockDirection, i32, i32)] = match direction {
            BlockDirection::North => &[
                (BlockDirection::North, bb.min.x + 1, bb.min.z - 1),
                (BlockDirection::West, bb.min.x - 1, bb.min.z + 1),
                (BlockDirection::East, bb.max.x + 1, bb.min.z + 1),
            ],
            BlockDirection::South => &[
                (BlockDirection::South, bb.min.x + 1, bb.max.z + 1),
                (BlockDirection::West, bb.min.x - 1, bb.min.z + 1),
                (BlockDirection::East, bb.max.x + 1, bb.min.z + 1),
            ],
            BlockDirection::West => &[
                (BlockDirection::North, bb.min.x + 1, bb.min.z - 1),
                (BlockDirection::South, bb.min.x + 1, bb.max.z + 1),
                (BlockDirection::West, bb.min.x - 1, bb.min.z + 1),
            ],
            _ => &[
                (BlockDirection::North, bb.min.x + 1, bb.min.z - 1),
                (BlockDirection::South, bb.min.x + 1, bb.max.z + 1),
                (BlockDirection::East, bb.max.x + 1, bb.min.z + 1),
            ],
        };
        for &(facing, x, z) in exits {
            let _ = generate_and_add_piece(
                Attachment {
                    x,
                    y: bb.min.y,
                    z,
                    facing,
                    depth,
                },
                start_min_x,
                start_min_z,
                self.mineshaft_type,
                random,
                collector,
            );
        }

        if self.is_two_floored {
            for (facing, x, z) in [
                (BlockDirection::North, bb.min.x + 1, bb.min.z - 1),
                (BlockDirection::West, bb.min.x - 1, bb.min.z + 1),
                (BlockDirection::East, bb.max.x + 1, bb.min.z + 1),
                (BlockDirection::South, bb.min.x + 1, bb.max.z + 1),
            ] {
                if random.next_bool() {
                    let _ = generate_and_add_piece(
                        Attachment {
                            x,
                            y: bb.min.y + 4,
                            z,
                            facing,
                            depth,
                        },
                        start_min_x,
                        start_min_z,
                        self.mineshaft_type,
                        random,
                        collector,
                    );
                }
            }
        }
    }
}

// ===========================================================================
// Stairs
// ===========================================================================

#[derive(Clone)]
struct MineshaftStairsPiece {
    piece: StructurePiece,
    mineshaft_type: MineshaftType,
}

impl MineshaftStairsPiece {
    fn create(
        att: &Attachment,
        mineshaft_type: MineshaftType,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(att.x, att.y, att.z, 0, -5, 0, 3, 8, 9, &att.facing);
        if collector.get_intersecting(&bbox).is_some() {
            return None;
        }
        let mut piece =
            StructurePiece::new(StructurePieceType::MineshaftStairs, bbox, att.depth + 1);
        piece.set_facing(Some(att.facing));
        Some(Self {
            piece,
            mineshaft_type,
        })
    }
}

impl StructurePieceBase for MineshaftStairsPiece {
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
        _br: &dyn WorldPortalExt,
        _r: &mut RandomGenerator,
        _s: i64,
        chunk_box: &BlockBox,
    ) {
        if is_in_invalid_location(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }
        let air = Block::CAVE_AIR.default_state;
        // Carve upper landing.
        fill_mineshaft(
            chunk,
            &self.piece,
            self.mineshaft_type,
            chunk_box,
            0,
            5,
            0,
            2,
            7,
            1,
            air,
        );
        // Carve lower landing.
        fill_mineshaft(
            chunk,
            &self.piece,
            self.mineshaft_type,
            chunk_box,
            0,
            0,
            7,
            2,
            2,
            8,
            air,
        );
        // Descending steps.
        for i in 0..5 {
            let y = 5 - i - i32::from(i < 4);
            fill_mineshaft(
                chunk,
                &self.piece,
                self.mineshaft_type,
                chunk_box,
                0,
                y,
                2 + i,
                2,
                7 - i,
                2 + i,
                air,
            );
        }
    }
}

impl MineshaftStairsPiece {
    fn add_children(
        &self,
        start_min_x: i32,
        start_min_z: i32,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        let bb = self.piece.bounding_box;
        let facing = self.piece.facing.unwrap_or(BlockDirection::North);
        let (x, z) = match facing {
            BlockDirection::North => (bb.min.x, bb.min.z - 1),
            BlockDirection::South => (bb.min.x, bb.max.z + 1),
            BlockDirection::West => (bb.min.x - 1, bb.min.z),
            _ => (bb.max.x + 1, bb.min.z),
        };
        let _ = generate_and_add_piece(
            Attachment {
                x,
                y: bb.min.y,
                z,
                facing,
                depth: self.piece.chain_length,
            },
            start_min_x,
            start_min_z,
            self.mineshaft_type,
            random,
            collector,
        );
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

const fn block_box_center(min: i32, max: i32) -> i32 {
    min + (max - min + 1) / 2
}

#[expect(
    clippy::manual_midpoint,
    reason = "Java integer division truncates negative sums toward zero"
)]
fn java_average(first: i32, second: i32) -> i32 {
    ((i64::from(first) + i64::from(second)) / 2) as i32
}

fn is_in_invalid_location(chunk: &ProtoChunk, bb: &BlockBox, chunk_box: &BlockBox) -> bool {
    let bounds = BlockBox::new(
        (bb.min.x - 1).max(chunk_box.min.x),
        (bb.min.y - 1).max(chunk_box.min.y),
        (bb.min.z - 1).max(chunk_box.min.z),
        (bb.max.x + 1).min(chunk_box.max.x),
        (bb.max.y + 1).min(chunk_box.max.y),
        (bb.max.z + 1).min(chunk_box.max.z),
    );
    let center = Vector3::new(
        java_average(bounds.min.x, bounds.max.x),
        java_average(bounds.min.y, bounds.max.y),
        java_average(bounds.min.z, bounds.max.z),
    );
    if chunk
        .get_terrain_gen_biome(center.x, center.y, center.z)
        .has_tag(&WorldgenBiome::MINECRAFT_MINESHAFT_BLOCKING)
    {
        return true;
    }

    for x in bounds.min.x..=bounds.max.x {
        for z in bounds.min.z..=bounds.max.z {
            if is_liquid(chunk, x, bounds.min.y, z) || is_liquid(chunk, x, bounds.max.y, z) {
                return true;
            }
        }
    }
    for x in bounds.min.x..=bounds.max.x {
        for y in bounds.min.y..=bounds.max.y {
            if is_liquid(chunk, x, y, bounds.min.z) || is_liquid(chunk, x, y, bounds.max.z) {
                return true;
            }
        }
    }
    for z in bounds.min.z..=bounds.max.z {
        for y in bounds.min.y..=bounds.max.y {
            if is_liquid(chunk, bounds.min.x, y, z) || is_liquid(chunk, bounds.max.x, y, z) {
                return true;
            }
        }
    }
    false
}

fn is_liquid(chunk: &ProtoChunk, x: i32, y: i32, z: i32) -> bool {
    chunk
        .get_block_state(&Vector3::new(x, y, z))
        .to_state()
        .is_liquid()
}

fn fill_world_box(
    chunk: &mut ProtoChunk,
    chunk_box: &BlockBox,
    bounds: &BlockBox,
    state: &BlockState,
    mineshaft_type: MineshaftType,
) {
    for x in bounds.min.x..=bounds.max.x {
        for y in bounds.min.y..=bounds.max.y {
            for z in bounds.min.z..=bounds.max.z {
                let pos = Vector3::new(x, y, z);
                if chunk_box.contains_pos(&pos)
                    && can_replace_mineshaft(chunk, &pos, mineshaft_type)
                {
                    chunk.set_block_state(x, y, z, state);
                }
            }
        }
    }
}

fn fill_upper_half_sphere(
    chunk: &mut ProtoChunk,
    chunk_box: &BlockBox,
    bounds: &BlockBox,
    state: &BlockState,
    mineshaft_type: MineshaftType,
) {
    let x_span = (bounds.max.x - bounds.min.x + 1) as f32;
    let y_span = (bounds.max.y - bounds.min.y + 1) as f32;
    let z_span = (bounds.max.z - bounds.min.z + 1) as f32;
    let center_x = bounds.min.x as f32 + x_span / 2.0;
    let center_z = bounds.min.z as f32 + z_span / 2.0;
    for y in bounds.min.y..=bounds.max.y {
        let dy = (y - bounds.min.y) as f32 / y_span;
        for x in bounds.min.x..=bounds.max.x {
            let dx = (x as f32 - center_x) / (x_span * 0.5);
            for z in bounds.min.z..=bounds.max.z {
                let dz = (z as f32 - center_z) / (z_span * 0.5);
                let pos = Vector3::new(x, y, z);
                if dx * dx + dy * dy + dz * dz <= 1.05
                    && chunk_box.contains_pos(&pos)
                    && can_replace_mineshaft(chunk, &pos, mineshaft_type)
                {
                    chunk.set_block_state(x, y, z, state);
                }
            }
        }
    }
}

fn can_replace_mineshaft(
    chunk: &ProtoChunk,
    pos: &Vector3<i32>,
    mineshaft_type: MineshaftType,
) -> bool {
    let block = chunk.get_block_state(pos).to_block();
    block != Block::from_state_id(mineshaft_type.planks().id)
        && block != Block::from_state_id(mineshaft_type.log().id)
        && block != Block::from_state_id(mineshaft_type.fence().id)
        && block != &Block::IRON_CHAIN
}

#[expect(clippy::too_many_arguments)]
fn add_mineshaft_block(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    mineshaft_type: MineshaftType,
    state: &'static BlockState,
    x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
) {
    let pos = piece.offset_pos(x, y, z);
    if chunk_box.contains_pos(&pos) && can_replace_mineshaft(chunk, &pos, mineshaft_type) {
        piece.add_block(chunk, state, x, y, z, chunk_box);
    }
}

#[expect(clippy::too_many_arguments)]
fn place_mineshaft_block(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    mineshaft_type: MineshaftType,
    block_registry: &dyn WorldPortalExt,
    state: &'static BlockState,
    x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
) {
    let pos = piece.offset_pos(x, y, z);
    if chunk_box.contains_pos(&pos) && can_replace_mineshaft(chunk, &pos, mineshaft_type) {
        piece.place_block(chunk, block_registry, state, x, y, z, chunk_box);
    }
}

#[expect(clippy::too_many_arguments)]
fn fill_mineshaft(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    mineshaft_type: MineshaftType,
    chunk_box: &BlockBox,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
    state: &'static BlockState,
) {
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                add_mineshaft_block(chunk, piece, mineshaft_type, state, x, y, z, chunk_box);
            }
        }
    }
}

fn is_interior(
    chunk: &ProtoChunk,
    piece: &StructurePiece,
    x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
) -> bool {
    let pos = piece.offset_pos(x, y + 1, z);
    chunk_box.contains_pos(&pos)
        && pos.y < current_ocean_floor_height_exclusive(chunk, pos.x, pos.z)
}

fn current_ocean_floor_height_exclusive(chunk: &ProtoChunk, x: i32, z: i32) -> i32 {
    let bottom = i32::from(chunk.bottom_y());
    let stored_top = chunk
        .get_top_y(&HeightMap::OceanFloorWg, x, z)
        .min(bottom + i32::from(chunk.height()));

    // ProtoChunk's compact generation heightmap is monotonic. Structures can
    // carve its previous top block, so validate downward from that cached bound
    // to retain Vanilla's live OCEAN_FLOOR_WG semantics.
    for y in (bottom..stored_top).rev() {
        let state = chunk.get_block_state(&Vector3::new(x, y, z)).to_state();
        if blocks_movement(state, state.id.to_block_id()) {
            return y + 1;
        }
    }
    bottom
}

fn set_planks_block(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    planks: &BlockState,
    x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
) {
    if !is_interior(chunk, piece, x, y, z, chunk_box) {
        return;
    }
    let pos = piece.offset_pos(x, y, z);
    if !chunk
        .get_block_state(&pos)
        .to_state()
        .is_side_solid(pumpkin_data::BlockDirection::Up)
    {
        chunk.set_block_state(pos.x, pos.y, pos.z, planks);
    }
}

fn is_supporting_box(
    chunk: &ProtoChunk,
    piece: &StructurePiece,
    min_x: i32,
    max_x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
) -> bool {
    (min_x..=max_x).all(|x| {
        let pos = piece.offset_pos(x, y + 1, z);
        chunk_box.contains_pos(&pos) && !chunk.get_block_state(&pos).to_state().is_air()
    })
}

#[expect(clippy::too_many_arguments)]
fn maybe_place_wall_torch(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    block_registry: &dyn WorldPortalExt,
    random: &mut RandomGenerator,
    x: i32,
    y: i32,
    z: i32,
    facing: &str,
    chunk_box: &BlockBox,
    mineshaft_type: MineshaftType,
) {
    if random.next_f32() < 0.05 {
        let properties = Block::WALL_TORCH.from_properties(&[("facing", facing)]);
        place_mineshaft_block(
            chunk,
            piece,
            mineshaft_type,
            block_registry,
            BlockState::from_id(properties.to_state_id(&Block::WALL_TORCH)),
            x,
            y,
            z,
            chunk_box,
        );
    }
}

#[expect(clippy::too_many_arguments)]
fn maybe_place_cobweb(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    mineshaft_type: MineshaftType,
    random: &mut RandomGenerator,
    chance: f32,
    x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
) {
    if !is_interior(chunk, piece, x, y, z, chunk_box) || random.next_f32() >= chance {
        return;
    }
    let pos = piece.offset_pos(x, y, z);
    let neighbours = [
        (1, 0, 0, pumpkin_data::BlockDirection::West),
        (-1, 0, 0, pumpkin_data::BlockDirection::East),
        (0, 1, 0, pumpkin_data::BlockDirection::Down),
        (0, -1, 0, pumpkin_data::BlockDirection::Up),
        (0, 0, 1, pumpkin_data::BlockDirection::North),
        (0, 0, -1, pumpkin_data::BlockDirection::South),
    ]
    .into_iter()
    .filter(|&(dx, dy, dz, face)| {
        let neighbour = Vector3::new(pos.x + dx, pos.y + dy, pos.z + dz);
        chunk_box.contains_pos(&neighbour)
            && chunk
                .get_block_state(&neighbour)
                .to_state()
                .is_side_solid(face)
    })
    .count();
    if neighbours >= 2 && can_replace_mineshaft(chunk, &pos, mineshaft_type) {
        chunk.set_block_state(pos.x, pos.y, pos.z, Block::COBWEB.default_state);
    }
}

fn place_double_lower_or_upper_support(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    mineshaft_type: MineshaftType,
    log: &BlockState,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
) {
    for x in [0, 2] {
        let pos = piece.offset_pos(x, y, z);
        if chunk_box.contains_pos(&pos)
            && chunk.get_block_state(&pos).to_block()
                == Block::from_state_id(mineshaft_type.planks().id)
        {
            fill_pillar_down_or_chain_up(chunk, piece, mineshaft_type, log, x, y, z, chunk_box);
        }
    }
}

#[expect(clippy::too_many_arguments)]
fn fill_pillar_down_or_chain_up(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    mineshaft_type: MineshaftType,
    log: &BlockState,
    x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
) {
    let base = piece.offset_pos(x, y, z);
    if !chunk_box.contains_pos(&base) {
        return;
    }
    let mut distance = 1;
    let mut search_down = true;
    let mut search_up = true;
    while search_down || search_up {
        if search_down {
            let pos = Vector3::new(base.x, base.y - distance, base.z);
            let state = chunk.get_block_state(&pos).to_state();
            let replaceable =
                is_structure_replaceable(state) && state.id.to_block() != &Block::LAVA;
            if !replaceable && state.is_side_solid(pumpkin_data::BlockDirection::Up) {
                for fill_y in (pos.y + 1)..base.y {
                    chunk.set_block_state(base.x, fill_y, base.z, log);
                }
                return;
            }
            search_down = distance <= 20 && replaceable && pos.y > chunk.bottom_y() as i32 + 1;
        }

        if search_up {
            let pos = Vector3::new(base.x, base.y + distance, base.z);
            let state = chunk.get_block_state(&pos).to_state();
            let replaceable = is_structure_replaceable(state);
            if !replaceable
                && state.is_center_solid(pumpkin_data::BlockDirection::Down)
                && !is_falling_block(state.id.to_block())
            {
                chunk.set_block_state(base.x, base.y + 1, base.z, mineshaft_type.fence());
                for fill_y in (base.y + 2)..pos.y {
                    chunk.set_block_state(base.x, fill_y, base.z, Block::IRON_CHAIN.default_state);
                }
                return;
            }
            search_up = distance <= 50
                && replaceable
                && pos.y < chunk.bottom_y() as i32 + chunk.height() as i32 - 1;
        }
        distance += 1;
    }
}

fn is_falling_block(block: &Block) -> bool {
    block == &Block::SAND
        || block == &Block::RED_SAND
        || block == &Block::GRAVEL
        || block == &Block::DRAGON_EGG
        || block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_ANVIL)
        || block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_CONCRETE_POWDERS)
}

fn is_structure_replaceable(state: &BlockState) -> bool {
    let block = state.id.to_block();
    state.is_air()
        || state.is_liquid()
        || block == &Block::GLOW_LICHEN
        || block == &Block::SEAGRASS
        || block == &Block::TALL_SEAGRASS
}

fn place_spawner(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
    entity_id: &str,
) {
    let pos = piece.offset_pos(x, y, z);
    if !chunk_box.contains(pos.x, pos.y, pos.z) {
        return;
    }
    chunk.set_block_state(pos.x, pos.y, pos.z, Block::SPAWNER.default_state);
    let mut nbt = NbtCompound::new();
    nbt.put_string("id", "minecraft:mob_spawner".to_string());
    nbt.put_int("x", pos.x);
    nbt.put_int("y", pos.y);
    nbt.put_int("z", pos.z);
    let mut spawn_data = NbtCompound::new();
    let mut entity = NbtCompound::new();
    entity.put_string("id", entity_id.to_string());
    spawn_data.put_compound("entity", entity);
    nbt.put_compound("SpawnData", spawn_data);
    chunk.add_block_entity(nbt);
}

fn place_loot_minecart(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
    random: &mut RandomGenerator,
) -> bool {
    let pos = piece.offset_pos(x, y, z);
    if !chunk_box.contains(pos.x, pos.y, pos.z)
        || !chunk.get_block_state(&pos).to_state().is_air()
        || chunk
            .get_block_state(&Vector3::new(pos.x, pos.y - 1, pos.z))
            .to_state()
            .is_air()
    {
        return false;
    }

    let local_north_south = random.next_bool();
    let piece_north_south = matches!(
        piece.facing,
        Some(BlockDirection::North | BlockDirection::South)
    );
    let rail = if local_north_south == piece_north_south {
        Block::RAIL.default_state
    } else {
        let properties = Block::RAIL.from_properties(&[("shape", "east_west")]);
        BlockState::from_id(properties.to_state_id(&Block::RAIL))
    };
    chunk.set_block_state(pos.x, pos.y, pos.z, rail);

    let mut nbt = NbtCompound::new();
    nbt.put_string("id", "minecraft:chest_minecart".to_string());
    nbt.put_list(
        "Pos",
        vec![
            (f64::from(pos.x) + 0.5).into(),
            (f64::from(pos.y) + 0.5).into(),
            (f64::from(pos.z) + 0.5).into(),
        ],
    );
    nbt.put(
        "Motion",
        NbtTag::List(vec![0.0f64.into(), 0.0f64.into(), 0.0f64.into()]),
    );
    nbt.put("Rotation", NbtTag::List(vec![0.0f32.into(), 0.0f32.into()]));
    nbt.put_string(
        "LootTable",
        "minecraft:chests/abandoned_mineshaft".to_string(),
    );
    nbt.put_long("LootTableSeed", random.next_i64());
    chunk.add_structure_entity(nbt);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::structure::structures::{
        HeightSampler, StructureGeneratorContext, create_chunk_random,
    };
    use pumpkin_data::{dimension::Dimension, structures::StructureKeys};
    use pumpkin_util::world_seed::Seed;

    struct FixedHeight(i32);

    impl HeightSampler for FixedHeight {
        fn estimate_height(&mut self, _block_x: i32, _block_z: i32) -> i32 {
            self.0
        }
    }

    const fn direction_id(direction: Option<BlockDirection>) -> i32 {
        match direction {
            None => -1,
            Some(BlockDirection::North) => 0,
            Some(BlockDirection::South) => 1,
            Some(BlockDirection::West) => 2,
            Some(BlockDirection::East) => 3,
            Some(BlockDirection::Up | BlockDirection::Down) => unreachable!(),
        }
    }

    fn hash_i32(hash: &mut u64, value: i32) {
        for byte in value.to_le_bytes() {
            *hash ^= u64::from(byte);
            *hash = hash.wrapping_mul(0x100_0000_01b3);
        }
    }

    fn hash_box(hash: &mut u64, bounds: BlockBox) {
        for value in [
            bounds.min.x,
            bounds.min.y,
            bounds.min.z,
            bounds.max.x,
            bounds.max.y,
            bounds.max.z,
        ] {
            hash_i32(hash, value);
        }
    }

    fn generation_signature(
        seed: i64,
        chunk_x: i32,
        chunk_z: i32,
        mineshaft_type: MineshaftType,
        surface_y: i32,
    ) -> u64 {
        let generator = MineshaftGenerator { mineshaft_type };
        let mut height = FixedHeight(surface_y);
        let position = generator
            .get_structure_position(StructureGeneratorContext {
                seed,
                chunk_x,
                chunk_z,
                random: create_chunk_random(seed, chunk_x, chunk_z),
                sea_level: 63,
                min_y: -64,
                height_sampler: Some(&mut height),
                structure_key: Some(match mineshaft_type {
                    MineshaftType::Normal => StructureKeys::Mineshaft,
                    MineshaftType::Mesa => StructureKeys::MineshaftMesa,
                }),
            })
            .expect("mineshaft should generate");
        let collector = position.collector.lock().unwrap();
        let room = collector.pieces[0]
            .as_any()
            .downcast_ref::<MineshaftRoomPiece>()
            .expect("first mineshaft piece should be its room");
        let mut hash = 0xcbf2_9ce4_8422_2325;
        hash_i32(&mut hash, room.piece.bounding_box.min.y - MAGIC_START_Y);
        hash_i32(&mut hash, collector.pieces.len() as i32);

        for piece in &collector.pieces {
            let base = piece.get_structure_piece();
            if let Some(room) = piece.as_any().downcast_ref::<MineshaftRoomPiece>() {
                hash_i32(&mut hash, 0);
                hash_i32(&mut hash, base.chain_length as i32);
                hash_box(&mut hash, base.bounding_box);
                hash_i32(&mut hash, -1);
                hash_i32(&mut hash, room.child_entrance_boxes.len() as i32);
                for entrance in &room.child_entrance_boxes {
                    hash_box(&mut hash, *entrance);
                }
            } else if let Some(corridor) = piece.as_any().downcast_ref::<MineshaftCorridorPiece>() {
                hash_i32(&mut hash, 1);
                hash_i32(&mut hash, base.chain_length as i32);
                hash_box(&mut hash, base.bounding_box);
                hash_i32(&mut hash, direction_id(base.facing));
                hash_i32(&mut hash, i32::from(corridor.has_rails));
                hash_i32(&mut hash, i32::from(corridor.spider_corridor));
                hash_i32(&mut hash, corridor.num_sections);
            } else if let Some(crossing) = piece.as_any().downcast_ref::<MineshaftCrossingPiece>() {
                hash_i32(&mut hash, 2);
                hash_i32(&mut hash, base.chain_length as i32);
                hash_box(&mut hash, base.bounding_box);
                hash_i32(&mut hash, -1);
                hash_i32(&mut hash, i32::from(crossing.is_two_floored));
                hash_i32(&mut hash, direction_id(base.facing));
            } else {
                assert!(piece.as_any().is::<MineshaftStairsPiece>());
                hash_i32(&mut hash, 3);
                hash_i32(&mut hash, base.chain_length as i32);
                hash_box(&mut hash, base.bounding_box);
                hash_i32(&mut hash, direction_id(base.facing));
            }
        }
        hash
    }

    #[test]
    fn vanilla_center_rounding_is_preserved() {
        assert_eq!(block_box_center(0, 5), 3);
        assert_eq!(block_box_center(-6, -1), -3);
        assert_eq!(java_average(-6, -1), -3);
        assert_eq!(java_average(-5, 0), -2);
    }

    #[test]
    fn live_ocean_floor_scan_accounts_for_structure_carving() {
        let generator = crate::generation::get_world_gen(
            Seed(0),
            Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );
        let mut chunk = ProtoChunk::new(0, 0, &generator);
        chunk.set_block_state(0, 60, 0, Block::STONE.default_state);
        chunk.set_block_state(0, 70, 0, Block::STONE.default_state);
        chunk.set_block_state(0, 70, 0, Block::CAVE_AIR.default_state);

        assert_eq!(chunk.get_top_y(&HeightMap::OceanFloorWg, 0, 0), 71);
        assert_eq!(current_ocean_floor_height_exclusive(&chunk, 0, 0), 61);
    }

    #[test]
    fn mineshaft_assembles_multiple_pieces() {
        let generator = MineshaftGenerator {
            mineshaft_type: MineshaftType::Normal,
        };
        let context = StructureGeneratorContext {
            seed: 42,
            chunk_x: 0,
            chunk_z: 0,
            random: create_chunk_random(42, 0, 0),
            sea_level: 63,
            min_y: -64,
            height_sampler: None,
            structure_key: Some(StructureKeys::Mineshaft),
        };
        let position = generator
            .get_structure_position(context)
            .expect("mineshaft should generate");
        let count = position.collector.lock().unwrap().pieces.len();
        assert!(count > 2, "mineshaft generated only {count} pieces");
    }

    #[test]
    fn piece_graphs_match_vanilla_26_2() {
        for (seed, chunk_x, chunk_z, kind, surface_y, expected) in [
            (42, 0, 0, MineshaftType::Normal, 90, 0xb8e0_88f3_ab03_af36),
            (
                123_456_789,
                -37,
                84,
                MineshaftType::Normal,
                90,
                0x74d3_de7e_6584_ba3f,
            ),
            (0, 100, -200, MineshaftType::Mesa, 90, 0xb7c8_124b_8c6f_1fe4),
            (
                -987_654_321,
                -12,
                -34,
                MineshaftType::Mesa,
                120,
                0xfb37_31c2_ce84_f691,
            ),
        ] {
            assert_eq!(
                generation_signature(seed, chunk_x, chunk_z, kind, surface_y),
                expected,
                "piece graph differed for seed {seed} at {chunk_x}, {chunk_z}"
            );
        }
    }
}
