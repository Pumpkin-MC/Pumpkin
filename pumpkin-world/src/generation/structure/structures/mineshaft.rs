//! Vanilla 26.2 mineshaft generation.
//!
//! Ported from:
//! - `net/minecraft/world/level/levelgen/structure/structures/MineshaftStructure.java`
//! - `net/minecraft/world/level/levelgen/structure/structures/MineshaftPieces.java`
//!
//! Line numbers in comments refer to the decompiled vanilla sources above.
//! Random draws are performed in exactly the vanilla order so that piece
//! layouts are seed-compatible.

use std::sync::Arc;

use pumpkin_data::BlockDirection as BlockFace;
use pumpkin_data::{
    Block, BlockState,
    tag::{RegistryKey, get_tag_ids},
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::{
    BlockDirection, HeightMap,
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::{
        biome_coords,
        positions::chunk_pos::{get_center_x, start_block_x, start_block_z},
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition, WorldPortalExt,
            },
        },
    },
};

// MineshaftPieces.java l.53-56
const MAX_PILLAR_HEIGHT: i32 = 20;
const MAX_CHAIN_HEIGHT: i32 = 50;
const MAX_DEPTH: u32 = 8;
const MAGIC_START_Y: i32 = 50;

// MineshaftPieces.java l.460: BuiltInLootTables.ABANDONED_MINESHAFT
const ABANDONED_MINESHAFT_LOOT: &str = "minecraft:chests/abandoned_mineshaft";

/// `MineshaftStructure.Type` (MineshaftStructure.java l.83-100).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MineshaftType {
    /// NORMAL: oak log / oak planks / oak fence (l.85).
    Normal,
    /// MESA: dark oak log / dark oak planks / dark oak fence (l.86).
    Mesa,
}

impl MineshaftType {
    const fn wood_block(self) -> &'static Block {
        match self {
            Self::Normal => &Block::OAK_LOG,
            Self::Mesa => &Block::DARK_OAK_LOG,
        }
    }

    const fn planks_block(self) -> &'static Block {
        match self {
            Self::Normal => &Block::OAK_PLANKS,
            Self::Mesa => &Block::DARK_OAK_PLANKS,
        }
    }

    const fn fence_block(self) -> &'static Block {
        match self {
            Self::Normal => &Block::OAK_FENCE,
            Self::Mesa => &Block::DARK_OAK_FENCE,
        }
    }
}

pub struct MineshaftGenerator {
    pub is_mesa: bool,
}

impl StructureGenerator for MineshaftGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let mut random = context.random;

        // MineshaftStructure.findGenerationPoint (l.51): vanilla consumes one
        // nextDouble before generating any piece.
        random.next_f64();

        let mineshaft_type = if self.is_mesa {
            MineshaftType::Mesa
        } else {
            MineshaftType::Normal
        };

        // MineshaftStructure.generatePiecesAndAdjust (l.59-76).
        // Room start corner: chunkPos.getBlockX(2) / getBlockZ(2) (l.63).
        let west = start_block_x(context.chunk_x) + 2;
        let north = start_block_z(context.chunk_z) + 2;

        // MineShaftRoom ctor (MineshaftPieces.java l.709-710). Java evaluates the
        // BoundingBox arguments left to right, so the three nextInt(6) draws are
        // max-x, then max-y, then max-z.
        let max_x = west + 7 + random.next_bounded_i32(6);
        let max_y = 54 + random.next_bounded_i32(6);
        let max_z = north + 7 + random.next_bounded_i32(6);
        let room_box = BlockBox::new(west, MAGIC_START_Y, north, max_x, max_y, max_z);

        let mut pieces = vec![MineshaftPiece {
            piece: StructurePiece::new(StructurePieceType::MineshaftRoom, room_box, 0),
            mineshaft_type,
            kind: PieceKind::Room {
                entrances: Vec::new(),
            },
        }];
        let start = StartInfo {
            min_x: room_box.min.x,
            min_z: room_box.min.z,
            mineshaft_type,
        };
        // builder.addPiece(room); room.addChildren(room, builder, random) (l.64-65).
        add_children(&mut pieces, 0, start, &mut random);

        let mut collector = StructurePiecesCollector::default();
        for piece in pieces {
            collector.add_piece(Box::new(piece));
        }

        let y_offset = if self.is_mesa {
            // MESA (MineshaftStructure.java l.67-73): project the structure up
            // towards the terrain surface instead of sinking it below sea level.
            let bounding_box = collector.get_bounding_box();
            // BoundingBox.getCenter(): min + (max - min + 1) / 2 per axis.
            let center_x = bounding_box.min.x + (bounding_box.max.x - bounding_box.min.x + 1) / 2;
            let center_y = bounding_box.min.y + (bounding_box.max.y - bounding_box.min.y + 1) / 2;
            let center_z = bounding_box.min.z + (bounding_box.max.z - bounding_box.min.z + 1) / 2;
            let sea_level = context.sea_level;
            // Vanilla queries WORLD_SURFACE_WG via ChunkGenerator.getBaseHeight
            // (l.69); Pumpkin's noise-based surface estimator is the equivalent.
            // With no sampler available we fall back to sea level.
            let surface_height = context.height_sampler.map_or(sea_level, |sampler| {
                sampler.estimate_height(center_x, center_z)
            });
            // l.70: Mth.randomBetweenInclusive(random, seaLevel, surfaceHeight),
            // drawn only when the surface lies above sea level.
            let target_y = if surface_height <= sea_level {
                sea_level
            } else {
                random.next_inbetween_i32(sea_level, surface_height)
            };
            let offset = target_y - center_y;
            collector.shift(offset);
            offset
        } else {
            // l.75: builder.moveBelowSeaLevel(seaLevel, minY, random, 10).
            collector.shift_into(context.sea_level, context.min_y, &mut random, 10)
        };

        // findGenerationPoint (l.53-56): (middleBlockX, 50, minBlockZ) + yOffset.
        Some(StructurePosition {
            start_pos: BlockPos::new(
                get_center_x(context.chunk_x),
                MAGIC_START_Y + y_offset,
                start_block_z(context.chunk_z),
            ),
            collector: Arc::new(collector.into()),
        })
    }
}

/// Data of the start (room) piece needed while expanding the piece tree
/// (vanilla passes the start `StructurePiece` itself, l.79-93).
#[derive(Clone, Copy)]
struct StartInfo {
    min_x: i32,
    min_z: i32,
    mineshaft_type: MineshaftType,
}

/// Per-variant state of a mineshaft piece.
enum PieceKind {
    /// `MineShaftRoom` (MineshaftPieces.java l.705-780).
    Room { entrances: Vec<BlockBox> },
    /// `MineShaftCorridor` (l.276-615).
    Corridor {
        has_rails: bool,
        spider_corridor: bool,
        has_placed_spider: bool,
        num_sections: i32,
    },
    /// `MineShaftCrossing` (l.95-211). Vanilla never calls setOrientation for
    /// crossings; it keeps a separate `direction` field and works in absolute
    /// coordinates.
    Crossing {
        direction: BlockDirection,
        is_two_floored: bool,
    },
    /// `MineShaftStairs` (l.213-274).
    Stairs,
}

#[derive(Clone, Copy)]
enum PieceTag {
    Room,
    Corridor,
    Crossing,
    Stairs,
}

pub struct MineshaftPiece {
    piece: StructurePiece,
    mineshaft_type: MineshaftType,
    kind: PieceKind,
}

// ---------------------------------------------------------------------------
// Piece-tree expansion (vanilla addChildren / createRandomShaftPiece)
// ---------------------------------------------------------------------------

fn intersects_any(pieces: &[MineshaftPiece], bounding_box: &BlockBox) -> bool {
    // StructurePieceAccessor.findCollisionPiece equivalent.
    pieces
        .iter()
        .any(|piece| piece.piece.bounding_box.intersects(bounding_box))
}

/// `MineshaftPieces.generateAndAddPiece` (l.79-93).
#[allow(clippy::too_many_arguments)]
fn generate_and_add_piece(
    pieces: &mut Vec<MineshaftPiece>,
    start: StartInfo,
    random: &mut RandomGenerator,
    x: i32,
    y: i32,
    z: i32,
    direction: BlockDirection,
    depth: u32,
) -> Option<usize> {
    // l.80: recursion depth limit.
    if depth > MAX_DEPTH {
        return None;
    }
    // l.83: stay within 80 blocks of the start piece corner.
    if (x - start.min_x).abs() > 80 || (z - start.min_z).abs() > 80 {
        return None;
    }
    let index = create_random_shaft_piece(pieces, random, x, y, z, direction, depth + 1, start)?;
    // l.89-90: addPiece happened inside create_random_shaft_piece; now recurse.
    add_children(pieces, index, start, random);
    Some(index)
}

/// `MineshaftPieces.createRandomShaftPiece` (l.58-77). Pushes the created
/// piece (vanilla `addPiece`, l.89) and returns its index.
#[allow(clippy::too_many_arguments)]
fn create_random_shaft_piece(
    pieces: &mut Vec<MineshaftPiece>,
    random: &mut RandomGenerator,
    x: i32,
    y: i32,
    z: i32,
    direction: BlockDirection,
    gen_depth: u32,
    start: StartInfo,
) -> Option<usize> {
    // l.59: weights are 80..99 crossing, 70..79 stairs, 0..69 corridor.
    let selection = random.next_bounded_i32(100);
    if selection >= 80 {
        let bounding_box = find_crossing(pieces, random, x, y, z, direction)?;
        // MineShaftCrossing ctor (l.113-117).
        let is_two_floored = bounding_box.max.y - bounding_box.min.y + 1 > 3;
        pieces.push(MineshaftPiece {
            piece: StructurePiece::new(
                StructurePieceType::MineshaftCrossing,
                bounding_box,
                gen_depth,
            ),
            mineshaft_type: start.mineshaft_type,
            kind: PieceKind::Crossing {
                direction,
                is_two_floored,
            },
        });
    } else if selection >= 70 {
        let bounding_box = find_stairs(pieces, x, y, z, direction)?;
        // MineShaftStairs ctor (l.215-218).
        let mut piece =
            StructurePiece::new(StructurePieceType::MineshaftStairs, bounding_box, gen_depth);
        piece.set_facing(Some(direction));
        pieces.push(MineshaftPiece {
            piece,
            mineshaft_type: start.mineshaft_type,
            kind: PieceKind::Stairs,
        });
    } else {
        let bounding_box = find_corridor_size(pieces, random, x, y, z, direction)?;
        // MineShaftCorridor ctor (l.300-306).
        let mut piece = StructurePiece::new(
            StructurePieceType::MineshaftCorridor,
            bounding_box,
            gen_depth,
        );
        piece.set_facing(Some(direction));
        let has_rails = random.next_bounded_i32(3) == 0; // l.303
        // l.304: nextInt(23) is only drawn when hasRails failed (Java `&&`).
        let spider_corridor = !has_rails && random.next_bounded_i32(23) == 0;
        // l.305: sections along the corridor axis.
        let num_sections = if matches!(direction, BlockDirection::North | BlockDirection::South) {
            (bounding_box.max.z - bounding_box.min.z + 1) / 5
        } else {
            (bounding_box.max.x - bounding_box.min.x + 1) / 5
        };
        pieces.push(MineshaftPiece {
            piece,
            mineshaft_type: start.mineshaft_type,
            kind: PieceKind::Corridor {
                has_rails,
                spider_corridor,
                has_placed_spider: false,
                num_sections,
            },
        });
    }
    Some(pieces.len() - 1)
}

/// `MineShaftCrossing.findCrossing` (l.119-132).
fn find_crossing(
    pieces: &[MineshaftPiece],
    random: &mut RandomGenerator,
    x: i32,
    y: i32,
    z: i32,
    direction: BlockDirection,
) -> Option<BlockBox> {
    // l.120: 1/4 chance for a two-floored (height 6) crossing.
    let max_y = if random.next_bounded_i32(4) == 0 {
        6
    } else {
        2
    };
    // l.121-126.
    let mut bounding_box = match direction {
        BlockDirection::South => BlockBox::new(-1, 0, 0, 3, max_y, 4),
        BlockDirection::West => BlockBox::new(-4, 0, -1, 0, max_y, 3),
        BlockDirection::East => BlockBox::new(0, 0, -1, 4, max_y, 3),
        // NORTH is the `default` arm in vanilla (l.122).
        _ => BlockBox::new(-1, 0, -4, 3, max_y, 0),
    };
    bounding_box.move_pos(x, y, z);
    (!intersects_any(pieces, &bounding_box)).then_some(bounding_box)
}

/// `MineShaftStairs.findStairs` (l.224-236). Draws no random values.
fn find_stairs(
    pieces: &[MineshaftPiece],
    x: i32,
    y: i32,
    z: i32,
    direction: BlockDirection,
) -> Option<BlockBox> {
    // l.225-230.
    let mut bounding_box = match direction {
        BlockDirection::South => BlockBox::new(0, -5, 0, 2, 2, 8),
        BlockDirection::West => BlockBox::new(-8, -5, 0, 0, 2, 2),
        BlockDirection::East => BlockBox::new(0, -5, 0, 8, 2, 2),
        _ => BlockBox::new(0, -5, -8, 2, 2, 0),
    };
    bounding_box.move_pos(x, y, z);
    (!intersects_any(pieces, &bounding_box)).then_some(bounding_box)
}

/// `MineShaftCorridor.findCorridorSize` (l.311-328).
fn find_corridor_size(
    pieces: &[MineshaftPiece],
    random: &mut RandomGenerator,
    x: i32,
    y: i32,
    z: i32,
    direction: BlockDirection,
) -> Option<BlockBox> {
    // l.312: 2-4 sections, shrinking until the corridor fits.
    let mut sections = random.next_bounded_i32(3) + 2;
    while sections > 0 {
        let length = sections * 5;
        // l.315-320.
        let mut bounding_box = match direction {
            BlockDirection::South => BlockBox::new(0, 0, 0, 2, 2, length - 1),
            BlockDirection::West => BlockBox::new(-(length - 1), 0, 0, 0, 2, 2),
            BlockDirection::East => BlockBox::new(0, 0, 0, length - 1, 2, 2),
            _ => BlockBox::new(0, 0, -(length - 1), 2, 2, 0),
        };
        bounding_box.move_pos(x, y, z);
        if !intersects_any(pieces, &bounding_box) {
            return Some(bounding_box);
        }
        sections -= 1;
    }
    None
}

/// Dispatches to the per-variant `addChildren` implementation.
fn add_children(
    pieces: &mut Vec<MineshaftPiece>,
    index: usize,
    start: StartInfo,
    random: &mut RandomGenerator,
) {
    let bounding_box = pieces[index].piece.bounding_box;
    let depth = pieces[index].piece.chain_length;
    let facing = pieces[index].piece.facing;
    let (tag, crossing_data) = match &pieces[index].kind {
        PieceKind::Room { .. } => (PieceTag::Room, None),
        PieceKind::Corridor { .. } => (PieceTag::Corridor, None),
        PieceKind::Crossing {
            direction,
            is_two_floored,
        } => (PieceTag::Crossing, Some((*direction, *is_two_floored))),
        PieceKind::Stairs => (PieceTag::Stairs, None),
    };
    match tag {
        PieceTag::Room => add_room_children(pieces, index, bounding_box, depth, start, random),
        PieceTag::Corridor => {
            add_corridor_children(pieces, bounding_box, facing, depth, start, random);
        }
        PieceTag::Crossing => {
            let (direction, is_two_floored) =
                crossing_data.expect("crossing piece carries crossing data");
            add_crossing_children(
                pieces,
                bounding_box,
                direction,
                is_two_floored,
                depth,
                start,
                random,
            );
        }
        PieceTag::Stairs => add_stairs_children(pieces, bounding_box, facing, depth, start, random),
    }
}

fn push_room_entrance(pieces: &mut [MineshaftPiece], room_index: usize, entrance: BlockBox) {
    if let PieceKind::Room { entrances } = &mut pieces[room_index].kind {
        entrances.push(entrance);
    }
}

/// `MineShaftRoom.addChildren` (l.720-753).
#[allow(clippy::too_many_lines)]
fn add_room_children(
    pieces: &mut Vec<MineshaftPiece>,
    room_index: usize,
    bb: BlockBox,
    depth: u32,
    start: StartInfo,
    random: &mut RandomGenerator,
) {
    // l.725-728.
    let mut height_space = (bb.max.y - bb.min.y + 1) - 3 - 1;
    if height_space <= 0 {
        height_space = 1;
    }
    let x_span = bb.max.x - bb.min.x + 1;
    let z_span = bb.max.z - bb.min.z + 1;

    // North wall exits (l.729-734).
    let mut offset = 0;
    while offset < x_span {
        offset += random.next_bounded_i32(x_span);
        if offset + 3 > x_span {
            break;
        }
        let y = bb.min.y + random.next_bounded_i32(height_space) + 1;
        if let Some(child) = generate_and_add_piece(
            pieces,
            start,
            random,
            bb.min.x + offset,
            y,
            bb.min.z - 1,
            BlockDirection::North,
            depth,
        ) {
            let cb = pieces[child].piece.bounding_box;
            push_room_entrance(
                pieces,
                room_index,
                BlockBox::new(
                    cb.min.x,
                    cb.min.y,
                    bb.min.z,
                    cb.max.x,
                    cb.max.y,
                    bb.min.z + 1,
                ),
            );
        }
        offset += 4;
    }

    // South wall exits (l.735-740).
    let mut offset = 0;
    while offset < x_span {
        offset += random.next_bounded_i32(x_span);
        if offset + 3 > x_span {
            break;
        }
        let y = bb.min.y + random.next_bounded_i32(height_space) + 1;
        if let Some(child) = generate_and_add_piece(
            pieces,
            start,
            random,
            bb.min.x + offset,
            y,
            bb.max.z + 1,
            BlockDirection::South,
            depth,
        ) {
            let cb = pieces[child].piece.bounding_box;
            push_room_entrance(
                pieces,
                room_index,
                BlockBox::new(
                    cb.min.x,
                    cb.min.y,
                    bb.max.z - 1,
                    cb.max.x,
                    cb.max.y,
                    bb.max.z,
                ),
            );
        }
        offset += 4;
    }

    // West wall exits (l.741-746).
    let mut offset = 0;
    while offset < z_span {
        offset += random.next_bounded_i32(z_span);
        if offset + 3 > z_span {
            break;
        }
        let y = bb.min.y + random.next_bounded_i32(height_space) + 1;
        if let Some(child) = generate_and_add_piece(
            pieces,
            start,
            random,
            bb.min.x - 1,
            y,
            bb.min.z + offset,
            BlockDirection::West,
            depth,
        ) {
            let cb = pieces[child].piece.bounding_box;
            push_room_entrance(
                pieces,
                room_index,
                BlockBox::new(
                    bb.min.x,
                    cb.min.y,
                    cb.min.z,
                    bb.min.x + 1,
                    cb.max.y,
                    cb.max.z,
                ),
            );
        }
        offset += 4;
    }

    // East wall exits (l.747-752).
    let mut offset = 0;
    while offset < z_span {
        offset += random.next_bounded_i32(z_span);
        if offset + 3 > z_span {
            break;
        }
        let y = bb.min.y + random.next_bounded_i32(height_space) + 1;
        if let Some(child) = generate_and_add_piece(
            pieces,
            start,
            random,
            bb.max.x + 1,
            y,
            bb.min.z + offset,
            BlockDirection::East,
            depth,
        ) {
            let cb = pieces[child].piece.bounding_box;
            push_room_entrance(
                pieces,
                room_index,
                BlockBox::new(
                    bb.max.x - 1,
                    cb.min.y,
                    cb.min.z,
                    bb.max.x,
                    cb.max.y,
                    cb.max.z,
                ),
            );
        }
        offset += 4;
    }
}

/// `MineShaftCorridor.addChildren` (l.330-412).
#[allow(clippy::too_many_lines)]
fn add_corridor_children(
    pieces: &mut Vec<MineshaftPiece>,
    bb: BlockBox,
    facing: Option<BlockDirection>,
    depth: u32,
    start: StartInfo,
    random: &mut RandomGenerator,
) {
    // l.334: 0-1 continue straight, 2 turn one way, 3 turn the other.
    let end_selection = random.next_bounded_i32(4);
    if let Some(direction) = facing {
        match direction {
            BlockDirection::South => {
                // l.350-360.
                if end_selection <= 1 {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.min.x,
                        y,
                        bb.max.z + 1,
                        direction,
                        depth,
                    );
                } else if end_selection == 2 {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.min.x - 1,
                        y,
                        bb.max.z - 3,
                        BlockDirection::West,
                        depth,
                    );
                } else {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.max.x + 1,
                        y,
                        bb.max.z - 3,
                        BlockDirection::East,
                        depth,
                    );
                }
            }
            BlockDirection::West => {
                // l.362-372.
                if end_selection <= 1 {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.min.x - 1,
                        y,
                        bb.min.z,
                        direction,
                        depth,
                    );
                } else if end_selection == 2 {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.min.x,
                        y,
                        bb.min.z - 1,
                        BlockDirection::North,
                        depth,
                    );
                } else {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.min.x,
                        y,
                        bb.max.z + 1,
                        BlockDirection::South,
                        depth,
                    );
                }
            }
            BlockDirection::East => {
                // l.374-384.
                if end_selection <= 1 {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.max.x + 1,
                        y,
                        bb.min.z,
                        direction,
                        depth,
                    );
                } else if end_selection == 2 {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.max.x - 3,
                        y,
                        bb.min.z - 1,
                        BlockDirection::North,
                        depth,
                    );
                } else {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.max.x - 3,
                        y,
                        bb.max.z + 1,
                        BlockDirection::South,
                        depth,
                    );
                }
            }
            _ => {
                // NORTH is the `default` arm in vanilla (l.338-348).
                if end_selection <= 1 {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.min.x,
                        y,
                        bb.min.z - 1,
                        direction,
                        depth,
                    );
                } else if end_selection == 2 {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.min.x - 1,
                        y,
                        bb.min.z,
                        BlockDirection::West,
                        depth,
                    );
                } else {
                    let y = bb.min.y - 1 + random.next_bounded_i32(3);
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.max.x + 1,
                        y,
                        bb.min.z,
                        BlockDirection::East,
                        depth,
                    );
                }
            }
        }
    }

    // Sideways branches every 5 blocks, 1/5 chance each side (l.387-410).
    if depth < MAX_DEPTH {
        if matches!(facing, Some(BlockDirection::North | BlockDirection::South)) {
            let mut z = bb.min.z + 3;
            while z + 3 <= bb.max.z {
                let selection = random.next_bounded_i32(5);
                if selection == 0 {
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.min.x - 1,
                        bb.min.y,
                        z,
                        BlockDirection::West,
                        depth + 1,
                    );
                } else if selection == 1 {
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        bb.max.x + 1,
                        bb.min.y,
                        z,
                        BlockDirection::East,
                        depth + 1,
                    );
                }
                z += 5;
            }
        } else {
            let mut x = bb.min.x + 3;
            while x + 3 <= bb.max.x {
                let selection = random.next_bounded_i32(5);
                if selection == 0 {
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        x,
                        bb.min.y,
                        bb.min.z - 1,
                        BlockDirection::North,
                        depth + 1,
                    );
                } else if selection == 1 {
                    generate_and_add_piece(
                        pieces,
                        start,
                        random,
                        x,
                        bb.min.y,
                        bb.max.z + 1,
                        BlockDirection::South,
                        depth + 1,
                    );
                }
                x += 5;
            }
        }
    }
}

/// `MineShaftCrossing.addChildren` (l.134-176).
#[allow(clippy::too_many_arguments)]
fn add_crossing_children(
    pieces: &mut Vec<MineshaftPiece>,
    bb: BlockBox,
    direction: BlockDirection,
    is_two_floored: bool,
    depth: u32,
    start: StartInfo,
    random: &mut RandomGenerator,
) {
    match direction {
        BlockDirection::South => {
            // l.144-148.
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x + 1,
                bb.min.y,
                bb.max.z + 1,
                BlockDirection::South,
                depth,
            );
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x - 1,
                bb.min.y,
                bb.min.z + 1,
                BlockDirection::West,
                depth,
            );
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.max.x + 1,
                bb.min.y,
                bb.min.z + 1,
                BlockDirection::East,
                depth,
            );
        }
        BlockDirection::West => {
            // l.150-154.
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x + 1,
                bb.min.y,
                bb.min.z - 1,
                BlockDirection::North,
                depth,
            );
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x + 1,
                bb.min.y,
                bb.max.z + 1,
                BlockDirection::South,
                depth,
            );
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x - 1,
                bb.min.y,
                bb.min.z + 1,
                BlockDirection::West,
                depth,
            );
        }
        BlockDirection::East => {
            // l.156-159.
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x + 1,
                bb.min.y,
                bb.min.z - 1,
                BlockDirection::North,
                depth,
            );
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x + 1,
                bb.min.y,
                bb.max.z + 1,
                BlockDirection::South,
                depth,
            );
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.max.x + 1,
                bb.min.y,
                bb.min.z + 1,
                BlockDirection::East,
                depth,
            );
        }
        _ => {
            // NORTH is the `default` arm in vanilla (l.138-142).
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x + 1,
                bb.min.y,
                bb.min.z - 1,
                BlockDirection::North,
                depth,
            );
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x - 1,
                bb.min.y,
                bb.min.z + 1,
                BlockDirection::West,
                depth,
            );
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.max.x + 1,
                bb.min.y,
                bb.min.z + 1,
                BlockDirection::East,
                depth,
            );
        }
    }
    // Upper floor exits, 50% each (l.162-175).
    if is_two_floored {
        if random.next_bool() {
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x + 1,
                bb.min.y + 3 + 1,
                bb.min.z - 1,
                BlockDirection::North,
                depth,
            );
        }
        if random.next_bool() {
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x - 1,
                bb.min.y + 3 + 1,
                bb.min.z + 1,
                BlockDirection::West,
                depth,
            );
        }
        if random.next_bool() {
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.max.x + 1,
                bb.min.y + 3 + 1,
                bb.min.z + 1,
                BlockDirection::East,
                depth,
            );
        }
        if random.next_bool() {
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x + 1,
                bb.min.y + 3 + 1,
                bb.max.z + 1,
                BlockDirection::South,
                depth,
            );
        }
    }
}

/// `MineShaftStairs.addChildren` (l.238-261).
fn add_stairs_children(
    pieces: &mut Vec<MineshaftPiece>,
    bb: BlockBox,
    facing: Option<BlockDirection>,
    depth: u32,
    start: StartInfo,
    random: &mut RandomGenerator,
) {
    let Some(direction) = facing else {
        return;
    };
    match direction {
        BlockDirection::South => {
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x,
                bb.min.y,
                bb.max.z + 1,
                BlockDirection::South,
                depth,
            );
        }
        BlockDirection::West => {
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x - 1,
                bb.min.y,
                bb.min.z,
                BlockDirection::West,
                depth,
            );
        }
        BlockDirection::East => {
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.max.x + 1,
                bb.min.y,
                bb.min.z,
                BlockDirection::East,
                depth,
            );
        }
        _ => {
            // NORTH is the `default` arm in vanilla (l.244-246).
            generate_and_add_piece(
                pieces,
                start,
                random,
                bb.min.x,
                bb.min.y,
                bb.min.z - 1,
                BlockDirection::North,
                depth,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Block placement (vanilla postProcess)
// ---------------------------------------------------------------------------

fn block_state_with(block: &'static Block, props: &[(&str, &str)]) -> &'static BlockState {
    BlockState::from_id(block.from_properties(props).to_state_id(block))
}

/// Whether the block's vanilla class extends `FallingBlock`
/// (`MineShaftCorridor.canHangChainBelow`, l.575-577). Registered subclasses in
/// Blocks.java: `SandBlock` (sand, red sand), `ColoredFallingBlock` (gravel),
/// `AnvilBlock`, `ConcretePowderBlock`, `DragonEggBlock`. `BrushableBlock`
/// (suspicious sand/gravel) only implements `Fallable` and is NOT a
/// `FallingBlock`.
fn is_falling_block(block: &Block) -> bool {
    [
        &Block::SAND,
        &Block::RED_SAND,
        &Block::GRAVEL,
        &Block::ANVIL,
        &Block::CHIPPED_ANVIL,
        &Block::DAMAGED_ANVIL,
        &Block::DRAGON_EGG,
        &Block::WHITE_CONCRETE_POWDER,
        &Block::ORANGE_CONCRETE_POWDER,
        &Block::MAGENTA_CONCRETE_POWDER,
        &Block::LIGHT_BLUE_CONCRETE_POWDER,
        &Block::YELLOW_CONCRETE_POWDER,
        &Block::LIME_CONCRETE_POWDER,
        &Block::PINK_CONCRETE_POWDER,
        &Block::GRAY_CONCRETE_POWDER,
        &Block::LIGHT_GRAY_CONCRETE_POWDER,
        &Block::CYAN_CONCRETE_POWDER,
        &Block::PURPLE_CONCRETE_POWDER,
        &Block::BLUE_CONCRETE_POWDER,
        &Block::BROWN_CONCRETE_POWDER,
        &Block::GREEN_CONCRETE_POWDER,
        &Block::RED_CONCRETE_POWDER,
        &Block::BLACK_CONCRETE_POWDER,
    ]
    .contains(&block)
}

/// Reads a block state, treating out-of-world positions as air (vanilla
/// returns void air there).
fn state_and_block_at(
    chunk: &ProtoChunk,
    x: i32,
    y: i32,
    z: i32,
) -> (&'static BlockState, &'static Block) {
    let min_y = i32::from(chunk.bottom_y());
    let max_y = min_y + i32::from(chunk.height()) - 1;
    if y < min_y || y > max_y {
        return (Block::AIR.default_state, &Block::AIR);
    }
    let id = chunk.get_block_state(&Vector3::new(x, y, z));
    (id.to_state(), id.to_block())
}

fn fill_column_between(
    chunk: &mut ProtoChunk,
    state: &'static BlockState,
    x: i32,
    z: i32,
    bottom_inclusive: i32,
    top_exclusive: i32,
) {
    // MineShaftCorridor.fillColumnBetween (l.565-569).
    for y in bottom_inclusive..top_exclusive {
        chunk.set_block_state(x, y, z, state);
    }
}

impl MineshaftPiece {
    fn wood_state(&self) -> &'static BlockState {
        self.mineshaft_type.wood_block().default_state
    }

    fn planks_state(&self) -> &'static BlockState {
        self.mineshaft_type.planks_block().default_state
    }

    fn fence_state(&self) -> &'static BlockState {
        self.mineshaft_type.fence_block().default_state
    }

    /// `MineShaftPiece.canBeReplaced` override (l.631-635): mineshaft pieces
    /// never overwrite already-placed mineshaft wood, planks, fences or chains
    /// (so intersecting corridors keep their supports).
    fn can_be_replaced(&self, chunk: &ProtoChunk, pos: &Vector3<i32>) -> bool {
        let block = chunk.get_block_state(pos).to_block();
        block != self.mineshaft_type.planks_block()
            && block != self.mineshaft_type.wood_block()
            && block != self.mineshaft_type.fence_block()
            && block != &Block::IRON_CHAIN
    }

    /// `StructurePiece.placeBlock` (StructurePiece.java l.156-178) including
    /// the mineshaft `canBeReplaced` override.
    fn place_block(
        &self,
        chunk: &mut ProtoChunk,
        state: &'static BlockState,
        x: i32,
        y: i32,
        z: i32,
        chunk_box: &BlockBox,
    ) {
        let pos = self.piece.offset_pos(x, y, z);
        if !chunk_box.contains_pos(&pos) {
            return;
        }
        if !self.can_be_replaced(chunk, &pos) {
            return;
        }
        self.piece.add_block(chunk, state, x, y, z, chunk_box);
    }

    /// `StructurePiece.generateBox` (StructurePiece.java l.210-223) routed
    /// through the mineshaft `placeBlock`. All mineshaft calls pass
    /// skipAir=false, so that parameter is omitted.
    #[allow(clippy::too_many_arguments)]
    fn generate_box(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        x0: i32,
        y0: i32,
        z0: i32,
        x1: i32,
        y1: i32,
        z1: i32,
        edge: &'static BlockState,
        fill: &'static BlockState,
    ) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                for z in z0..=z1 {
                    let border = y == y0 || y == y1 || x == x0 || x == x1 || z == z0 || z == z1;
                    self.place_block(chunk, if border { edge } else { fill }, x, y, z, chunk_box);
                }
            }
        }
    }

    /// `StructurePiece.generateMaybeBox` (StructurePiece.java l.245-258).
    /// One nextFloat is drawn per cell regardless of the outcome. All
    /// mineshaft calls pass skipAir=false, so that parameter is omitted.
    #[allow(clippy::too_many_arguments)]
    fn generate_maybe_box(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        random: &mut RandomGenerator,
        probability: f32,
        x0: i32,
        y0: i32,
        z0: i32,
        x1: i32,
        y1: i32,
        z1: i32,
        edge: &'static BlockState,
        fill: &'static BlockState,
        has_to_be_inside: bool,
    ) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                for z in z0..=z1 {
                    if random.next_f32() > probability {
                        continue;
                    }
                    if has_to_be_inside && !self.is_interior(chunk, x, y, z, chunk_box) {
                        continue;
                    }
                    let border = y == y0 || y == y1 || x == x0 || x == x1 || z == z0 || z == z1;
                    self.place_block(chunk, if border { edge } else { fill }, x, y, z, chunk_box);
                }
            }
        }
    }

    /// `StructurePiece.maybeGenerateBlock` (StructurePiece.java l.260-264).
    #[allow(clippy::too_many_arguments)]
    fn maybe_generate_block(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        random: &mut RandomGenerator,
        probability: f32,
        x: i32,
        y: i32,
        z: i32,
        state: &'static BlockState,
    ) {
        if random.next_f32() < probability {
            self.place_block(chunk, state, x, y, z, chunk_box);
        }
    }

    /// `StructurePiece.generateUpperHalfSphere` (StructurePiece.java
    /// l.266-284); the room passes skipAir=false.
    #[allow(clippy::too_many_arguments)]
    fn generate_upper_half_sphere(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        x0: i32,
        y0: i32,
        z0: i32,
        x1: i32,
        y1: i32,
        z1: i32,
        fill: &'static BlockState,
    ) {
        let diag_x = (x1 - x0 + 1) as f32;
        let diag_y = (y1 - y0 + 1) as f32;
        let diag_z = (z1 - z0 + 1) as f32;
        let center_x = x0 as f32 + diag_x / 2.0;
        let center_z = z0 as f32 + diag_z / 2.0;
        for y in y0..=y1 {
            let ny = (y - y0) as f32 / diag_y;
            for x in x0..=x1 {
                let nx = (x as f32 - center_x) / (diag_x * 0.5);
                for z in z0..=z1 {
                    let nz = (z as f32 - center_z) / (diag_z * 0.5);
                    if nx * nx + ny * ny + nz * nz <= 1.05 {
                        self.place_block(chunk, fill, x, y, z, chunk_box);
                    }
                }
            }
        }
    }

    /// `StructurePiece.isInterior` (StructurePiece.java l.192-198): the block
    /// ABOVE the given local position must be inside the chunk box and below
    /// the OCEAN_FLOOR_WG heightmap.
    fn is_interior(
        &self,
        chunk: &ProtoChunk,
        x: i32,
        y: i32,
        z: i32,
        chunk_box: &BlockBox,
    ) -> bool {
        let pos = self.piece.offset_pos(x, y + 1, z);
        if !chunk_box.contains_pos(&pos) {
            return false;
        }
        pos.y < chunk.get_top_y(&HeightMap::OceanFloorWg, pos.x, pos.z)
    }

    /// `MineShaftPiece.isSupportingBox` (l.642-648): every ceiling block above
    /// the beam must be non-air.
    fn is_supporting_box(
        &self,
        chunk: &ProtoChunk,
        chunk_box: &BlockBox,
        x0: i32,
        x1: i32,
        y1: i32,
        z: i32,
    ) -> bool {
        (x0..=x1).all(|x| {
            !self
                .piece
                .get_block_at(chunk, x, y1 + 1, z, chunk_box)
                .is_air()
        })
    }

    /// `MineShaftPiece.isInInvalidLocation` (l.650-691): skip generation in
    /// this chunk when the piece sits in a `minecraft:mineshaft_blocking`
    /// biome (deep dark) or touches liquid on any face of its expanded box.
    fn is_in_invalid_location(&self, chunk: &ProtoChunk, chunk_box: &BlockBox) -> bool {
        let bb = self.piece.bounding_box;
        let x0 = (bb.min.x - 1).max(chunk_box.min.x);
        let y0 = (bb.min.y - 1).max(chunk_box.min.y);
        let z0 = (bb.min.z - 1).max(chunk_box.min.z);
        let x1 = (bb.max.x + 1).min(chunk_box.max.x);
        let y1 = (bb.max.y + 1).min(chunk_box.max.y);
        let z1 = (bb.max.z + 1).min(chunk_box.max.z);

        // Biome check at the clamped-box center (l.659-661).
        let center_x = (x0 + x1) / 2;
        let center_y = (y0 + y1) / 2;
        let center_z = (z0 + z1) / 2;
        let biome_height = (chunk.height() >> 2) as i32;
        let biome_bottom = biome_coords::from_block(i32::from(chunk.bottom_y()));
        let biome_y =
            biome_coords::from_block(center_y).clamp(biome_bottom, biome_bottom + biome_height - 1);
        let biome_id = u16::from(chunk.get_biome_id(
            biome_coords::from_block(center_x),
            biome_y,
            biome_coords::from_block(center_z),
        ));
        if get_tag_ids(RegistryKey::WorldgenBiome, "minecraft:mineshaft_blocking")
            .is_some_and(|blocked| blocked.contains(&biome_id))
        {
            return true;
        }

        let liquid = |chunk: &ProtoChunk, x: i32, y: i32, z: i32| {
            chunk
                .get_block_state(&Vector3::new(x, y, z))
                .to_state()
                .is_liquid()
        };
        // Bottom and top faces (l.663-671).
        for x in x0..=x1 {
            for z in z0..=z1 {
                if liquid(chunk, x, y0, z) || liquid(chunk, x, y1, z) {
                    return true;
                }
            }
        }
        // North and south faces (l.672-680).
        for x in x0..=x1 {
            for y in y0..=y1 {
                if liquid(chunk, x, y, z0) || liquid(chunk, x, y, z1) {
                    return true;
                }
            }
        }
        // West and east faces (l.681-689).
        for z in z0..=z1 {
            for y in y0..=y1 {
                if liquid(chunk, x0, y, z) || liquid(chunk, x1, y, z) {
                    return true;
                }
            }
        }
        false
    }

    /// `MineShaftPiece.setPlanksBlock` (l.693-702): floor planks are only
    /// placed underground and only where the existing block has no sturdy top
    /// face.
    fn set_planks_block(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        planks: &'static BlockState,
        x: i32,
        y: i32,
        z: i32,
    ) {
        if !self.is_interior(chunk, x, y, z, chunk_box) {
            return;
        }
        let pos = self.piece.offset_pos(x, y, z);
        let (state, _) = state_and_block_at(chunk, pos.x, pos.y, pos.z);
        if !state.is_side_solid(BlockFace::Up) {
            chunk.set_block_state(pos.x, pos.y, pos.z, planks);
        }
    }

    // ---- Room --------------------------------------------------------------

    /// `MineShaftRoom.postProcess` (l.755-765).
    fn place_room(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let PieceKind::Room { entrances } = &self.kind else {
            return;
        };
        let bb = self.piece.bounding_box;
        let cave_air = Block::CAVE_AIR.default_state;
        // l.760: carve the lower 3 layers.
        self.generate_box(
            chunk,
            chunk_box,
            bb.min.x,
            bb.min.y + 1,
            bb.min.z,
            bb.max.x,
            (bb.min.y + 3).min(bb.max.y),
            bb.max.z,
            cave_air,
            cave_air,
        );
        // l.761-763: carve the recorded corridor entrances.
        for entrance in entrances {
            self.generate_box(
                chunk,
                chunk_box,
                entrance.min.x,
                entrance.max.y - 2,
                entrance.min.z,
                entrance.max.x,
                entrance.max.y,
                entrance.max.z,
                cave_air,
                cave_air,
            );
        }
        // l.764: dome the ceiling.
        self.generate_upper_half_sphere(
            chunk,
            chunk_box,
            bb.min.x,
            bb.min.y + 4,
            bb.min.z,
            bb.max.x,
            bb.max.y,
            bb.max.z,
            cave_air,
        );
    }

    // ---- Crossing ----------------------------------------------------------

    /// `MineShaftCrossing.postProcess` (l.178-204).
    fn place_crossing(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let PieceKind::Crossing { is_two_floored, .. } = &self.kind else {
            return;
        };
        let two_floored = *is_two_floored;
        let bb = self.piece.bounding_box;
        let planks = self.planks_state();
        let cave_air = Block::CAVE_AIR.default_state;
        if two_floored {
            // l.185-189.
            self.generate_box(
                chunk,
                chunk_box,
                bb.min.x + 1,
                bb.min.y,
                bb.min.z,
                bb.max.x - 1,
                bb.min.y + 3 - 1,
                bb.max.z,
                cave_air,
                cave_air,
            );
            self.generate_box(
                chunk,
                chunk_box,
                bb.min.x,
                bb.min.y,
                bb.min.z + 1,
                bb.max.x,
                bb.min.y + 3 - 1,
                bb.max.z - 1,
                cave_air,
                cave_air,
            );
            self.generate_box(
                chunk,
                chunk_box,
                bb.min.x + 1,
                bb.max.y - 2,
                bb.min.z,
                bb.max.x - 1,
                bb.max.y,
                bb.max.z,
                cave_air,
                cave_air,
            );
            self.generate_box(
                chunk,
                chunk_box,
                bb.min.x,
                bb.max.y - 2,
                bb.min.z + 1,
                bb.max.x,
                bb.max.y,
                bb.max.z - 1,
                cave_air,
                cave_air,
            );
            self.generate_box(
                chunk,
                chunk_box,
                bb.min.x + 1,
                bb.min.y + 3,
                bb.min.z + 1,
                bb.max.x - 1,
                bb.min.y + 3,
                bb.max.z - 1,
                cave_air,
                cave_air,
            );
        } else {
            // l.191-192.
            self.generate_box(
                chunk,
                chunk_box,
                bb.min.x + 1,
                bb.min.y,
                bb.min.z,
                bb.max.x - 1,
                bb.max.y,
                bb.max.z,
                cave_air,
                cave_air,
            );
            self.generate_box(
                chunk,
                chunk_box,
                bb.min.x,
                bb.min.y,
                bb.min.z + 1,
                bb.max.x,
                bb.max.y,
                bb.max.z - 1,
                cave_air,
                cave_air,
            );
        }
        // l.194-197: four corner pillars.
        self.place_support_pillar(
            chunk,
            chunk_box,
            bb.min.x + 1,
            bb.min.y,
            bb.min.z + 1,
            bb.max.y,
        );
        self.place_support_pillar(
            chunk,
            chunk_box,
            bb.min.x + 1,
            bb.min.y,
            bb.max.z - 1,
            bb.max.y,
        );
        self.place_support_pillar(
            chunk,
            chunk_box,
            bb.max.x - 1,
            bb.min.y,
            bb.min.z + 1,
            bb.max.y,
        );
        self.place_support_pillar(
            chunk,
            chunk_box,
            bb.max.x - 1,
            bb.min.y,
            bb.max.z - 1,
            bb.max.y,
        );
        // l.198-203: planks floor patch below the crossing.
        let y = bb.min.y - 1;
        for x in bb.min.x..=bb.max.x {
            for z in bb.min.z..=bb.max.z {
                self.set_planks_block(chunk, chunk_box, planks, x, y, z);
            }
        }
    }

    /// `MineShaftCrossing.placeSupportPillar` (l.206-210).
    fn place_support_pillar(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        x: i32,
        y0: i32,
        z: i32,
        y1: i32,
    ) {
        if !self
            .piece
            .get_block_at(chunk, x, y1 + 1, z, chunk_box)
            .is_air()
        {
            let planks = self.planks_state();
            let cave_air = Block::CAVE_AIR.default_state;
            self.generate_box(chunk, chunk_box, x, y0, z, x, y1, z, planks, cave_air);
        }
    }

    // ---- Stairs ------------------------------------------------------------

    /// `MineShaftStairs.postProcess` (l.263-273). Local coordinates with the
    /// piece orientation transform.
    fn place_stairs(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let cave_air = Block::CAVE_AIR.default_state;
        self.generate_box(chunk, chunk_box, 0, 5, 0, 2, 7, 1, cave_air, cave_air);
        self.generate_box(chunk, chunk_box, 0, 0, 7, 2, 2, 8, cave_air, cave_air);
        for i in 0..5 {
            let extra = i32::from(i < 4);
            self.generate_box(
                chunk,
                chunk_box,
                0,
                5 - i - extra,
                2 + i,
                2,
                7 - i,
                2 + i,
                cave_air,
                cave_air,
            );
        }
    }

    // ---- Corridor ----------------------------------------------------------

    /// `MineShaftCorridor.postProcess` (l.431-497).
    #[allow(clippy::too_many_lines)]
    fn place_corridor(
        &mut self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        chunk_box: &BlockBox,
    ) {
        let (has_rails, spider_corridor, mut has_placed_spider, num_sections) = match &self.kind {
            PieceKind::Corridor {
                has_rails,
                spider_corridor,
                has_placed_spider,
                num_sections,
            } => (
                *has_rails,
                *spider_corridor,
                *has_placed_spider,
                *num_sections,
            ),
            _ => return,
        };
        let length = num_sections * 5 - 1; // l.441
        let planks = self.planks_state();
        let cave_air = Block::CAVE_AIR.default_state;

        // l.443: carve the lower two layers.
        self.generate_box(chunk, chunk_box, 0, 0, 0, 2, 1, length, cave_air, cave_air);
        // l.444: carve the top layer with 80% per-block chance.
        self.generate_maybe_box(
            chunk, chunk_box, random, 0.8, 0, 2, 0, 2, 2, length, cave_air, cave_air, false,
        );
        // l.445-447: spider corridors get a 60% cobweb blanket (edges only,
        // interior positions only).
        if spider_corridor {
            self.generate_maybe_box(
                chunk,
                chunk_box,
                random,
                0.6,
                0,
                0,
                0,
                2,
                1,
                length,
                Block::COBWEB.default_state,
                cave_air,
                true,
            );
        }

        // l.448-476: one support/decoration pass per 5-block section.
        for section in 0..num_sections {
            let z = 2 + section * 5;
            self.place_support(chunk, chunk_box, 0, 0, z, 2, 2, random);
            // l.451-458: cobwebs around the support.
            self.maybe_place_cobweb(chunk, chunk_box, random, 0.1, 0, 2, z - 1);
            self.maybe_place_cobweb(chunk, chunk_box, random, 0.1, 2, 2, z - 1);
            self.maybe_place_cobweb(chunk, chunk_box, random, 0.1, 0, 2, z + 1);
            self.maybe_place_cobweb(chunk, chunk_box, random, 0.1, 2, 2, z + 1);
            self.maybe_place_cobweb(chunk, chunk_box, random, 0.05, 0, 2, z - 2);
            self.maybe_place_cobweb(chunk, chunk_box, random, 0.05, 2, 2, z - 2);
            self.maybe_place_cobweb(chunk, chunk_box, random, 0.05, 0, 2, z + 2);
            self.maybe_place_cobweb(chunk, chunk_box, random, 0.05, 2, 2, z + 2);
            // l.459-464: 1/100 chest minecart on either side.
            if random.next_bounded_i32(100) == 0 {
                self.create_chest_minecart(chunk, chunk_box, random, 2, 0, z - 1);
            }
            if random.next_bounded_i32(100) == 0 {
                self.create_chest_minecart(chunk, chunk_box, random, 0, 0, z + 1);
            }
            // l.465-476: one cave spider spawner per spider corridor.
            if spider_corridor && !has_placed_spider {
                let spawner_z = z - 1 + random.next_bounded_i32(3); // l.467
                let pos = self.piece.offset_pos(1, 0, spawner_z);
                if chunk_box.contains_pos(&pos)
                    && self.is_interior(chunk, 1, 0, spawner_z, chunk_box)
                {
                    has_placed_spider = true;
                    // l.471: direct setBlock, no canBeReplaced check.
                    chunk.set_block_state(pos.x, pos.y, pos.z, Block::SPAWNER.default_state);
                    // l.472-475: SpawnerBlockEntity.setEntityId(CAVE_SPIDER,
                    // random). The spawner has no spawn potentials, so vanilla
                    // draws no random values here.
                    let mut nbt = NbtCompound::new();
                    nbt.put_string("id", "minecraft:mob_spawner".to_string());
                    nbt.put_int("x", pos.x);
                    nbt.put_int("y", pos.y);
                    nbt.put_int("z", pos.z);
                    let mut entity = NbtCompound::new();
                    entity.put_string("id", "minecraft:cave_spider".to_string());
                    let mut spawn_data = NbtCompound::new();
                    spawn_data.put_compound("entity", entity);
                    nbt.put_compound("SpawnData", spawn_data);
                    chunk.add_block_entity(nbt);
                }
            }
        }

        // l.477-481: planks floor patches over holes.
        for x in 0..=2 {
            for z in 0..=length {
                self.set_planks_block(chunk, chunk_box, planks, x, -1, z);
            }
        }
        // l.482-487: log pillar down / chain up under the outer supports.
        self.place_double_lower_or_upper_support(chunk, chunk_box, 0, -1, 2);
        if num_sections > 1 {
            self.place_double_lower_or_upper_support(chunk, chunk_box, 0, -1, length - 2);
        }
        // l.488-496: rails with broken gaps.
        if has_rails {
            let rail = block_state_with(&Block::RAIL, &[("shape", "north_south")]);
            for z in 0..=length {
                let floor = self.piece.get_block_at(chunk, 1, -1, z, chunk_box);
                // l.492: vanilla `isSolidRender`; Pumpkin's closest predicate.
                if floor.is_air() || !floor.is_solid_block() {
                    continue;
                }
                // l.493: 70% underground, 90% when exposed to the sky.
                let probability = if self.is_interior(chunk, 1, 0, z, chunk_box) {
                    0.7
                } else {
                    0.9
                };
                self.maybe_generate_block(chunk, chunk_box, random, probability, 1, 0, z, rail);
            }
        }

        if let PieceKind::Corridor {
            has_placed_spider: placed,
            ..
        } = &mut self.kind
        {
            *placed = has_placed_spider;
        }
    }

    /// `MineShaftCorridor.createChest` override (l.414-429). Vanilla places a
    /// rail and spawns a chest minecart ENTITY holding the abandoned mineshaft
    /// loot table; Pumpkin queues the entity NBT on the proto chunk.
    fn create_chest_minecart(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        random: &mut RandomGenerator,
        x: i32,
        y: i32,
        z: i32,
    ) -> bool {
        let pos = self.piece.offset_pos(x, y, z);
        // l.417: needs to be inside the chunk, in air, above a non-air block.
        if !chunk_box.contains_pos(&pos) {
            return false;
        }
        if !chunk.get_block_state(&pos).to_state().is_air() {
            return false;
        }
        let (below, _) = state_and_block_at(chunk, pos.x, pos.y - 1, pos.z);
        if below.is_air() {
            return false;
        }
        // l.418: random rail orientation under the minecart.
        let shape = if random.next_bool() {
            "north_south"
        } else {
            "east_west"
        };
        self.place_block(
            chunk,
            block_state_with(&Block::RAIL, &[("shape", shape)]),
            x,
            y,
            z,
            chunk_box,
        );
        // l.420-425: chest minecart entity centered on the block, with a
        // deferred loot table seeded by nextLong.
        let mut nbt = NbtCompound::new();
        nbt.put_string("id", "minecraft:chest_minecart".to_string());
        nbt.put_list(
            "Pos",
            vec![
                NbtTag::Double(f64::from(pos.x) + 0.5),
                NbtTag::Double(f64::from(pos.y) + 0.5),
                NbtTag::Double(f64::from(pos.z) + 0.5),
            ],
        );
        nbt.put_list(
            "Motion",
            vec![
                NbtTag::Double(0.0),
                NbtTag::Double(0.0),
                NbtTag::Double(0.0),
            ],
        );
        nbt.put_string("LootTable", ABANDONED_MINESHAFT_LOOT.to_string());
        nbt.put_long("LootTableSeed", random.next_i64());
        chunk.add_entity(nbt);
        true
    }

    /// `MineShaftCorridor.placeSupport` (l.579-595): fence posts plus a planks
    /// beam, with a 1/4 chance of a broken beam and 5% wall torches otherwise.
    #[allow(clippy::too_many_arguments)]
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
        // l.580: no support without a solid ceiling; no random draws either.
        if !self.is_supporting_box(chunk, chunk_box, x0, x1, y1, z) {
            return;
        }
        let planks = self.planks_state();
        let fence = self.mineshaft_type.fence_block();
        let cave_air = Block::CAVE_AIR.default_state;
        // l.585-586: fence posts connected towards the walls.
        let fence_west = block_state_with(fence, &[("west", "true")]);
        let fence_east = block_state_with(fence, &[("east", "true")]);
        self.generate_box(
            chunk,
            chunk_box,
            x0,
            y0,
            z,
            x0,
            y1 - 1,
            z,
            fence_west,
            cave_air,
        );
        self.generate_box(
            chunk,
            chunk_box,
            x1,
            y0,
            z,
            x1,
            y1 - 1,
            z,
            fence_east,
            cave_air,
        );
        if random.next_bounded_i32(4) == 0 {
            // l.588-589: broken beam, only the two end planks.
            self.generate_box(chunk, chunk_box, x0, y1, z, x0, y1, z, planks, cave_air);
            self.generate_box(chunk, chunk_box, x1, y1, z, x1, y1, z, planks, cave_air);
        } else {
            // l.591-593: full beam with 5% wall torches on both sides.
            self.generate_box(chunk, chunk_box, x0, y1, z, x1, y1, z, planks, cave_air);
            self.maybe_generate_block(
                chunk,
                chunk_box,
                random,
                0.05,
                x0 + 1,
                y1,
                z - 1,
                block_state_with(&Block::WALL_TORCH, &[("facing", "south")]),
            );
            self.maybe_generate_block(
                chunk,
                chunk_box,
                random,
                0.05,
                x0 + 1,
                y1,
                z + 1,
                block_state_with(&Block::WALL_TORCH, &[("facing", "north")]),
            );
        }
    }

    /// `MineShaftCorridor.maybePlaceCobWeb` (l.597-601). The nextFloat draw
    /// only happens for interior positions (Java short-circuit).
    #[allow(clippy::too_many_arguments)]
    fn maybe_place_cobweb(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        random: &mut RandomGenerator,
        probability: f32,
        x: i32,
        y: i32,
        z: i32,
    ) {
        if self.is_interior(chunk, x, y, z, chunk_box)
            && random.next_f32() < probability
            && self.has_sturdy_neighbours(chunk, chunk_box, x, y, z, 2)
        {
            self.place_block(chunk, Block::COBWEB.default_state, x, y, z, chunk_box);
        }
    }

    /// `MineShaftCorridor.hasSturdyNeighbours` (l.603-614). Java
    /// `Direction.values()` order: DOWN, UP, NORTH, SOUTH, WEST, EAST.
    fn has_sturdy_neighbours(
        &self,
        chunk: &ProtoChunk,
        chunk_box: &BlockBox,
        x: i32,
        y: i32,
        z: i32,
        count: i32,
    ) -> bool {
        let base = self.piece.offset_pos(x, y, z);
        let mut sturdy = 0;
        for direction in [
            BlockFace::Down,
            BlockFace::Up,
            BlockFace::North,
            BlockFace::South,
            BlockFace::West,
            BlockFace::East,
        ] {
            let offset = direction.to_offset();
            let pos = Vector3::new(base.x + offset.x, base.y + offset.y, base.z + offset.z);
            if chunk_box.contains_pos(&pos)
                && chunk
                    .get_block_state(&pos)
                    .to_state()
                    .is_side_solid(direction.opposite())
            {
                sturdy += 1;
                if sturdy >= count {
                    return true;
                }
            }
        }
        false
    }

    /// `MineShaftCorridor.placeDoubleLowerOrUpperSupport` (l.499-508).
    fn place_double_lower_or_upper_support(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        x: i32,
        y: i32,
        z: i32,
    ) {
        let wood = self.wood_state();
        let planks_block = self.mineshaft_type.planks_block();
        let left = self.piece.get_block_at(chunk, x, y, z, chunk_box);
        if Block::from_state_id(left.id) == planks_block {
            self.fill_pillar_down_or_chain_up(chunk, wood, x, y, z, chunk_box);
        }
        let right = self.piece.get_block_at(chunk, x + 2, y, z, chunk_box);
        if Block::from_state_id(right.id) == planks_block {
            self.fill_pillar_down_or_chain_up(chunk, wood, x + 2, y, z, chunk_box);
        }
    }

    /// `MineShaftCorridor.fillPillarDownOrChainUp` (l.529-563): search down
    /// (max 20) for solid ground to build a log pillar, and up (max 50) for a
    /// ceiling to hang a fence + iron chain from.
    fn fill_pillar_down_or_chain_up(
        &self,
        chunk: &mut ProtoChunk,
        pillar: &'static BlockState,
        x: i32,
        y: i32,
        z: i32,
        chunk_box: &BlockBox,
    ) {
        let pos = self.piece.offset_pos(x, y, z);
        if !chunk_box.contains_pos(&pos) {
            return;
        }
        let world_y = pos.y;
        let min_y = i32::from(chunk.bottom_y());
        let max_y = min_y + i32::from(chunk.height()) - 1;
        let mut distance = 1;
        let mut check_below = true;
        let mut check_above = true;
        while check_below || check_above {
            if check_below {
                let below_y = world_y - distance;
                let (state, block) = state_and_block_at(chunk, pos.x, below_y, pos.z);
                // l.543: lava does not count as an empty column.
                let empty_below = StructurePiece::is_replaceable_by_structures(state, block)
                    && block != &Block::LAVA;
                // l.544: canPlaceColumnOnTopOf = isFaceSturdy(UP) (l.571-573).
                if !empty_below && state.is_side_solid(BlockFace::Up) {
                    fill_column_between(chunk, pillar, pos.x, pos.z, below_y + 1, world_y);
                    return;
                }
                // l.548.
                check_below = distance <= MAX_PILLAR_HEIGHT && empty_below && below_y > min_y + 1;
            }
            if check_above {
                let above_y = world_y + distance;
                let (state, block) = state_and_block_at(chunk, pos.x, above_y, pos.z);
                let empty_above = StructurePiece::is_replaceable_by_structures(state, block);
                // l.554: canHangChainBelow = canSupportCenter(DOWN) and not a
                // falling block (l.575-577).
                if !empty_above
                    && state.is_center_solid(BlockFace::Down)
                    && !is_falling_block(block)
                {
                    // l.555-556: fence right above the beam, iron chains up to
                    // the ceiling.
                    chunk.set_block_state(pos.x, world_y + 1, pos.z, self.fence_state());
                    fill_column_between(
                        chunk,
                        Block::IRON_CHAIN.default_state,
                        pos.x,
                        pos.z,
                        world_y + 2,
                        above_y,
                    );
                    return;
                }
                // l.559.
                check_above = distance <= MAX_CHAIN_HEIGHT && empty_above && above_y < max_y;
            }
            distance += 1;
        }
    }
}

impl StructurePieceBase for MineshaftPiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }

    /// `MineShaftRoom.move` override (l.767-773): the recorded entrance boxes
    /// shift together with the piece.
    fn translate(&mut self, x: i32, y: i32, z: i32) {
        self.piece.translate(x, y, z);
        if let PieceKind::Room { entrances } = &mut self.kind {
            for entrance in entrances {
                entrance.move_pos(x, y, z);
            }
        }
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        // Every MineShaftPiece.postProcess starts with isInInvalidLocation.
        if self.is_in_invalid_location(chunk, chunk_box) {
            return;
        }
        let tag = match &self.kind {
            PieceKind::Room { .. } => PieceTag::Room,
            PieceKind::Corridor { .. } => PieceTag::Corridor,
            PieceKind::Crossing { .. } => PieceTag::Crossing,
            PieceKind::Stairs => PieceTag::Stairs,
        };
        match tag {
            PieceTag::Room => self.place_room(chunk, chunk_box),
            PieceTag::Corridor => self.place_corridor(chunk, random, chunk_box),
            PieceTag::Crossing => self.place_crossing(chunk, chunk_box),
            PieceTag::Stairs => self.place_stairs(chunk, chunk_box),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::structure::structures::{HeightSampler, create_chunk_random};

    struct FixedHeightSampler(i32);

    impl HeightSampler for FixedHeightSampler {
        fn estimate_height(&mut self, _block_x: i32, _block_z: i32) -> i32 {
            self.0
        }
    }

    fn context(height_sampler: Option<&mut dyn HeightSampler>) -> StructureGeneratorContext<'_> {
        StructureGeneratorContext {
            seed: 123,
            chunk_x: 4,
            chunk_z: -3,
            random: create_chunk_random(123, 4, -3),
            sea_level: 63,
            min_y: -64,
            max_y: 319,
            height_sampler,
            structure_key: None,
        }
    }

    #[test]
    fn mineshaft_starts_with_a_room_and_expands() {
        let position = MineshaftGenerator { is_mesa: false }
            .get_structure_position(context(None))
            .expect("mineshaft has a generation position");
        let collector = position.collector.lock().unwrap();
        assert!(!collector.pieces.is_empty());
        let room = collector.pieces[0].get_structure_piece();
        assert_eq!(room.r#type, StructurePieceType::MineshaftRoom);
        // MineShaftRoom ctor (MineshaftPieces.java l.710): 8-13 blocks per
        // horizontal axis, 5-10 blocks tall.
        let bb = room.bounding_box;
        assert!((8..=13).contains(&(bb.max.x - bb.min.x + 1)));
        assert!((8..=13).contains(&(bb.max.z - bb.min.z + 1)));
        assert!((5..=10).contains(&(bb.max.y - bb.min.y + 1)));
    }

    #[test]
    fn normal_mineshafts_are_shifted_below_sea_level() {
        let position = MineshaftGenerator { is_mesa: false }
            .get_structure_position(context(None))
            .expect("mineshaft has a generation position");
        let bounding_box = position.get_bounding_box();

        // moveBelowSeaLevel(63, -64, random, 10) keeps the top below
        // seaLevel - 10 (StructurePiecesBuilder.java l.41-51).
        assert!(bounding_box.max.y <= 52);
        assert!(bounding_box.min.y >= -63);
        // The room sits at MAGIC_START_Y + offset, i.e. inside the final box.
        assert!(bounding_box.min.y <= position.start_pos.0.y);
        assert!(position.start_pos.0.y <= bounding_box.max.y);
    }

    #[test]
    fn mesa_mineshafts_follow_surface_height_range() {
        let mut height_sampler = FixedHeightSampler(120);
        let position = MineshaftGenerator { is_mesa: true }
            .get_structure_position(context(Some(&mut height_sampler)))
            .expect("mineshaft has a generation position");
        let bounding_box = position.get_bounding_box();
        // Vanilla BoundingBox.getCenter() formula.
        let center_y = bounding_box.min.y + (bounding_box.max.y - bounding_box.min.y + 1) / 2;

        // The center is projected into [seaLevel, surfaceHeight]
        // (MineshaftStructure.java l.70).
        assert!((63..=120).contains(&center_y));
    }
}
