//! Igloo structure generator for snowy biomes.
//!
//! Generates igloos matching vanilla Minecraft behavior, including:
//! - Snow block dome with interior furnishings
//! - Optional basement (50% chance) with ladder shaft and secret room

use std::sync::Arc;

use pumpkin_data::{
    Block, BlockState,
    block_properties::{
        BedPart, BlockHalf, BlockProperties, FurnaceLikeProperties, HorizontalFacing,
        LadderLikeProperties, OakTrapdoorLikeProperties, WhiteBedLikeProperties,
    },
};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_center_x, get_center_z},
        structure::{
            piece::StructurePieceType,
            shiftable_piece::ShiftableStructurePiece,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition,
            },
        },
    },
};

use pumpkin_util::random::RandomImpl;

/// Igloo dome width in blocks (from vanilla igloo/top.nbt).
const DOME_WIDTH: i32 = 7;

/// Igloo dome height in blocks (from vanilla igloo/top.nbt).
const DOME_HEIGHT: i32 = 5;

/// Igloo dome depth in blocks (from vanilla igloo/top.nbt).
const DOME_DEPTH: i32 = 8;

/// Height of each ladder shaft segment (from vanilla igloo/middle.nbt).
const SHAFT_HEIGHT: i32 = 3;

/// Basement room height in blocks (from vanilla igloo/bottom.nbt).
const BASEMENT_HEIGHT: i32 = 6;

/// Basement room depth in blocks (from vanilla igloo/bottom.nbt).
const BASEMENT_DEPTH: i32 = 9;

/// Vanilla pivot X offset for igloo/top template alignment.
const PIVOT_OFFSET_X: i32 = 3;

/// Vanilla pivot Z offset for igloo/top template alignment.
const PIVOT_OFFSET_Z: i32 = 5;

/// Generator for igloo structures in snowy biomes.
#[derive(Deserialize)]
pub struct IglooGenerator;

impl StructureGenerator for IglooGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let chunk_center_x = get_center_x(context.chunk_x);
        let chunk_center_z = get_center_z(context.chunk_z);

        // Apply pivot offset - structure origin is offset from chunk center
        let x = chunk_center_x - PIVOT_OFFSET_X;
        let z = chunk_center_z - PIVOT_OFFSET_Z;

        // IMPORTANT: Random call order must match vanilla for deterministic placement:
        // 1. Rotation first (vanilla: BlockRotation.random(random) calls nextInt(4))
        let facing = StructurePiece::get_random_horizontal_direction(&mut context.random);

        // 2. Basement check (vanilla: random.nextDouble() < 0.5)
        let has_basement = context.random.next_f64() < 0.5;

        // 3. Ladder segments: 4-11 if has basement (vanilla: random.nextInt(8) + 4)
        let ladder_segments = if has_basement {
            context.random.next_bounded_i32(8) as u8 + 4
        } else {
            0
        };

        // Only use dome height for the bounding box - basement extends below terrain
        // The height adjustment will place the dome at terrain level

        let mut collector = StructurePiecesCollector::default();
        let mut piece = IglooPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::Igloo,
                x,
                64,
                z,
                DOME_WIDTH,
                DOME_HEIGHT,
                DOME_DEPTH,
                facing.get_axis(),
            ),
            has_basement,
            ladder_segments,
        };
        // Set facing so coordinate transforms work correctly
        piece
            .shiftable_structure_piece
            .piece
            .set_facing(Some(facing));
        collector.add_piece(Box::new(piece));

        Some(StructurePosition {
            start_pos: BlockPos::new(chunk_center_x, 64, chunk_center_z),
            collector: Arc::new(collector.into()),
        })
    }
}

/// Single igloo structure piece containing dome and optional basement.
#[derive(Clone)]
pub struct IglooPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
    has_basement: bool,
    ladder_segments: u8,
}

impl IglooPiece {
    /// Places the igloo dome structure.
    fn place_dome(&self, chunk: &mut ProtoChunk) {
        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        let snow = Block::SNOW_BLOCK.default_state;
        let air = Block::AIR.default_state;
        let white_carpet = Block::WHITE_CARPET.default_state;
        let light_gray_carpet = Block::LIGHT_GRAY_CARPET.default_state;
        let ice = Block::ICE.default_state;

        // Layer Y=0 (floor) - from vanilla NBT
        // Z=0 row
        p.add_block(chunk, snow, 2, 0, 0, &box_limit);
        p.add_block(chunk, snow, 3, 0, 0, &box_limit);
        p.add_block(chunk, snow, 4, 0, 0, &box_limit);
        // Z=1 row
        p.add_block(chunk, snow, 2, 0, 1, &box_limit);
        p.add_block(chunk, snow, 3, 0, 1, &box_limit);
        p.add_block(chunk, snow, 4, 0, 1, &box_limit);
        // Z=2 row
        for x in 1..=5 {
            p.add_block(chunk, snow, x, 0, 2, &box_limit);
        }
        // Z=3 row
        for x in 0..=6 {
            p.add_block(chunk, snow, x, 0, 3, &box_limit);
        }
        // Z=4 row
        for x in 0..=6 {
            p.add_block(chunk, snow, x, 0, 4, &box_limit);
        }
        // Z=5 row - trapdoor position
        for x in 0..=6 {
            if x == 3 && self.has_basement {
                // Oak trapdoor for basement access
                let mut trapdoor_props = OakTrapdoorLikeProperties::default(&Block::OAK_TRAPDOOR);
                trapdoor_props.facing = HorizontalFacing::North;
                trapdoor_props.half = BlockHalf::Top;
                trapdoor_props.open = false;
                trapdoor_props.powered = false;
                trapdoor_props.waterlogged = false;
                let trapdoor =
                    BlockState::from_id(trapdoor_props.to_state_id(&Block::OAK_TRAPDOOR));
                p.add_block(chunk, trapdoor, x, 0, 5, &box_limit);
            } else {
                p.add_block(chunk, snow, x, 0, 5, &box_limit);
            }
        }
        // Z=6 row
        for x in 1..=5 {
            p.add_block(chunk, snow, x, 0, 6, &box_limit);
        }
        // Z=7 row
        for x in 2..=4 {
            p.add_block(chunk, snow, x, 0, 7, &box_limit);
        }

        // Layer Y=1 (main interior floor with furnishings)
        // Z=0 row - entrance
        p.add_block(chunk, snow, 2, 1, 0, &box_limit);
        p.add_block(chunk, air, 3, 1, 0, &box_limit); // entrance
        p.add_block(chunk, snow, 4, 1, 0, &box_limit);
        // Z=1 row
        p.add_block(chunk, snow, 2, 1, 1, &box_limit);
        p.add_block(chunk, air, 3, 1, 1, &box_limit);
        p.add_block(chunk, snow, 4, 1, 1, &box_limit);
        // Z=2 row
        p.add_block(chunk, snow, 1, 1, 2, &box_limit);
        p.add_block(chunk, air, 2, 1, 2, &box_limit);
        p.add_block(chunk, air, 3, 1, 2, &box_limit);
        p.add_block(chunk, air, 4, 1, 2, &box_limit);
        p.add_block(chunk, snow, 5, 1, 2, &box_limit);
        // Z=3 row - furnace
        p.add_block(chunk, snow, 0, 1, 3, &box_limit);
        // Furnace facing east
        let mut furnace_props = FurnaceLikeProperties::default(&Block::FURNACE);
        furnace_props.facing = HorizontalFacing::East;
        furnace_props.lit = false;
        let furnace = BlockState::from_id(furnace_props.to_state_id(&Block::FURNACE));
        p.add_block(chunk, furnace, 1, 1, 3, &box_limit);
        p.add_block(chunk, white_carpet, 2, 1, 3, &box_limit);
        p.add_block(chunk, white_carpet, 3, 1, 3, &box_limit);
        p.add_block(chunk, white_carpet, 4, 1, 3, &box_limit);
        p.add_block(chunk, air, 5, 1, 3, &box_limit);
        p.add_block(chunk, snow, 6, 1, 3, &box_limit);
        // Z=4 row - ice, redstone torch, bed foot
        p.add_block(chunk, ice, 0, 1, 4, &box_limit);
        p.add_block(
            chunk,
            Block::REDSTONE_TORCH.default_state,
            1,
            1,
            4,
            &box_limit,
        );
        p.add_block(chunk, white_carpet, 2, 1, 4, &box_limit);
        p.add_block(chunk, white_carpet, 3, 1, 4, &box_limit);
        p.add_block(chunk, white_carpet, 4, 1, 4, &box_limit);
        // Bed foot
        let mut bed_foot_props = WhiteBedLikeProperties::default(&Block::RED_BED);
        bed_foot_props.facing = HorizontalFacing::South;
        bed_foot_props.part = BedPart::Foot;
        bed_foot_props.occupied = false;
        let bed_foot = BlockState::from_id(bed_foot_props.to_state_id(&Block::RED_BED));
        p.add_block(chunk, bed_foot, 5, 1, 4, &box_limit);
        p.add_block(chunk, ice, 6, 1, 4, &box_limit);
        // Z=5 row - crafting table, bed head
        p.add_block(chunk, snow, 0, 1, 5, &box_limit);
        p.add_block(
            chunk,
            Block::CRAFTING_TABLE.default_state,
            1,
            1,
            5,
            &box_limit,
        );
        p.add_block(chunk, white_carpet, 2, 1, 5, &box_limit);
        p.add_block(chunk, white_carpet, 3, 1, 5, &box_limit);
        p.add_block(chunk, white_carpet, 4, 1, 5, &box_limit);
        // Bed head
        let mut bed_head_props = WhiteBedLikeProperties::default(&Block::RED_BED);
        bed_head_props.facing = HorizontalFacing::South;
        bed_head_props.part = BedPart::Head;
        bed_head_props.occupied = false;
        let bed_head = BlockState::from_id(bed_head_props.to_state_id(&Block::RED_BED));
        p.add_block(chunk, bed_head, 5, 1, 5, &box_limit);
        p.add_block(chunk, snow, 6, 1, 5, &box_limit);
        // Z=6 row - light gray carpet
        p.add_block(chunk, snow, 1, 1, 6, &box_limit);
        p.add_block(chunk, light_gray_carpet, 2, 1, 6, &box_limit);
        p.add_block(chunk, light_gray_carpet, 3, 1, 6, &box_limit);
        p.add_block(chunk, light_gray_carpet, 4, 1, 6, &box_limit);
        p.add_block(chunk, snow, 5, 1, 6, &box_limit);
        // Z=7 row
        for x in 2..=4 {
            p.add_block(chunk, snow, x, 1, 7, &box_limit);
        }

        // Layer Y=2 (walls)
        // Z=0 row - entrance opening
        p.add_block(chunk, snow, 2, 2, 0, &box_limit);
        p.add_block(chunk, air, 3, 2, 0, &box_limit); // entrance
        p.add_block(chunk, snow, 4, 2, 0, &box_limit);
        // Z=1 row
        p.add_block(chunk, snow, 2, 2, 1, &box_limit);
        p.add_block(chunk, air, 3, 2, 1, &box_limit);
        p.add_block(chunk, snow, 4, 2, 1, &box_limit);
        // Z=2 row
        p.add_block(chunk, snow, 1, 2, 2, &box_limit);
        for x in 2..=4 {
            p.add_block(chunk, air, x, 2, 2, &box_limit);
        }
        p.add_block(chunk, snow, 5, 2, 2, &box_limit);
        // Z=3-5 rows
        for z in 3..=5 {
            p.add_block(chunk, snow, 0, 2, z, &box_limit);
            for x in 1..=5 {
                p.add_block(chunk, air, x, 2, z, &box_limit);
            }
            p.add_block(chunk, snow, 6, 2, z, &box_limit);
        }
        // Z=6 row
        p.add_block(chunk, snow, 1, 2, 6, &box_limit);
        for x in 2..=4 {
            p.add_block(chunk, air, x, 2, 6, &box_limit);
        }
        p.add_block(chunk, snow, 5, 2, 6, &box_limit);
        // Z=7 row
        for x in 2..=4 {
            p.add_block(chunk, snow, x, 2, 7, &box_limit);
        }

        // Layer Y=3 (dome narrowing)
        p.add_block(chunk, snow, 3, 3, 0, &box_limit);
        p.add_block(chunk, snow, 3, 3, 1, &box_limit);
        for x in 2..=4 {
            p.add_block(chunk, snow, x, 3, 2, &box_limit);
        }
        // Z=3-5 - walls
        for z in 3..=5 {
            p.add_block(chunk, snow, 1, 3, z, &box_limit);
            for x in 2..=4 {
                p.add_block(chunk, air, x, 3, z, &box_limit);
            }
            p.add_block(chunk, snow, 5, 3, z, &box_limit);
        }
        for x in 2..=4 {
            p.add_block(chunk, snow, x, 3, 6, &box_limit);
        }

        // Layer Y=4 (roof)
        for z in 3..=5 {
            for x in 2..=4 {
                p.add_block(chunk, snow, x, 4, z, &box_limit);
            }
        }
    }

    /// Places a single ladder shaft segment (vanilla: igloo/middle.nbt - 3x3x3)
    fn place_ladder_segment(
        &self,
        chunk: &mut ProtoChunk,
        base_y: i32,
        offset_x: i32,
        offset_z: i32,
    ) {
        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        let stone_brick = Block::STONE_BRICKS.default_state;

        // Create ladder facing north
        let mut ladder_props = LadderLikeProperties::default(&Block::LADDER);
        ladder_props.facing = HorizontalFacing::North;
        ladder_props.waterlogged = false;
        let ladder = BlockState::from_id(ladder_props.to_state_id(&Block::LADDER));

        // Each segment is 3 blocks tall, cross shape with ladder in center
        for dy in 0..SHAFT_HEIGHT {
            let y = base_y - dy;
            // Cross pattern: center at (1,1) in local coords
            // North: (1, 0)
            p.add_block(chunk, stone_brick, offset_x + 1, y, offset_z, &box_limit);
            // West: (0, 1)
            p.add_block(chunk, stone_brick, offset_x, y, offset_z + 1, &box_limit);
            // Center: ladder
            p.add_block(chunk, ladder, offset_x + 1, y, offset_z + 1, &box_limit);
            // East: (2, 1)
            p.add_block(
                chunk,
                stone_brick,
                offset_x + 2,
                y,
                offset_z + 1,
                &box_limit,
            );
            // South: (1, 2)
            p.add_block(
                chunk,
                stone_brick,
                offset_x + 1,
                y,
                offset_z + 2,
                &box_limit,
            );
        }
    }

    /// Places the ladder shaft connecting dome to basement
    fn place_ladder_shaft(&self, chunk: &mut ProtoChunk, dome_floor_y: i32) {
        // Shaft offset from vanilla: OFFSETS_FROM_TOP for middle is (2, -3, 4)
        // Meaning the shaft starts at dome position + (2, 0, 4) and goes down
        let offset_x = 2;
        let offset_z = 4;

        for segment in 0..self.ladder_segments {
            let segment_base_y = dome_floor_y - 1 - (segment as i32 * SHAFT_HEIGHT);
            self.place_ladder_segment(chunk, segment_base_y, offset_x, offset_z);
        }
    }

    /// Places the basement room (vanilla: igloo/bottom.nbt - 7x6x9)
    /// This is a simplified version - vanilla uses many block variants
    fn place_basement(&self, chunk: &mut ProtoChunk, dome_floor_y: i32) {
        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        // Basement offset from vanilla: OFFSETS_FROM_TOP for bottom is (0, -3, -2)
        // Total Y offset from dome floor
        let total_shaft_depth = self.ladder_segments as i32 * SHAFT_HEIGHT;
        let basement_top_y = dome_floor_y - total_shaft_depth;

        // Offset in X/Z from vanilla
        let offset_x = 0;
        let offset_z = -2;

        let stone_brick = Block::STONE_BRICKS.default_state;
        let mossy_stone_brick = Block::MOSSY_STONE_BRICKS.default_state;
        let cracked_stone_brick = Block::CRACKED_STONE_BRICKS.default_state;
        let air = Block::AIR.default_state;
        let red_carpet = Block::RED_CARPET.default_state;
        let iron_bars = Block::IRON_BARS.default_state;

        // Create ladder facing north
        let mut ladder_props = LadderLikeProperties::default(&Block::LADDER);
        ladder_props.facing = HorizontalFacing::North;
        ladder_props.waterlogged = false;
        let ladder = BlockState::from_id(ladder_props.to_state_id(&Block::LADDER));

        // Layer Y=0 (floor) - mix of stone variants
        for z in 0..BASEMENT_DEPTH {
            for x in 1..=5 {
                let bx = offset_x + x;
                let bz = offset_z + z;
                // Use stone_bricks as base, with some variation
                let block = if (x + z) % 3 == 0 {
                    cracked_stone_brick
                } else {
                    stone_brick
                };
                p.add_block(chunk, block, bx, basement_top_y - 5, bz, &box_limit);
            }
        }

        // Layer Y=1-3 (walls)
        for dy in 1..=3 {
            let y = basement_top_y - 5 + dy;

            // North and south walls (Z=0 and Z=8)
            for x in 1..=5 {
                p.add_block(chunk, stone_brick, offset_x + x, y, offset_z, &box_limit);
                if dy <= 2 {
                    p.add_block(
                        chunk,
                        mossy_stone_brick,
                        offset_x + x,
                        y,
                        offset_z + 8,
                        &box_limit,
                    );
                } else {
                    p.add_block(
                        chunk,
                        stone_brick,
                        offset_x + x,
                        y,
                        offset_z + 8,
                        &box_limit,
                    );
                }
            }

            // East and west walls
            for z in 1..=7 {
                // Cell walls with iron bars at z=2
                if z == 2 && dy <= 2 {
                    // Iron bars for cell dividers
                    p.add_block(chunk, iron_bars, offset_x + 2, y, offset_z + z, &box_limit);
                    p.add_block(chunk, iron_bars, offset_x + 4, y, offset_z + z, &box_limit);
                }

                // Outer walls
                if z >= 3 {
                    let wall_block = if dy == 1 {
                        mossy_stone_brick
                    } else {
                        stone_brick
                    };
                    p.add_block(chunk, wall_block, offset_x, y, offset_z + z, &box_limit);
                    p.add_block(chunk, wall_block, offset_x + 6, y, offset_z + z, &box_limit);
                }
            }

            // Interior air
            for z in 3..=7 {
                for x in 1..=5 {
                    if dy == 1 && x == 1 && (z == 3 || z == 4) {
                        // Red carpet on floor
                        p.add_block(chunk, red_carpet, offset_x + x, y, offset_z + z, &box_limit);
                    } else if !(z == 2 && (x == 2 || x == 4)) {
                        p.add_block(chunk, air, offset_x + x, y, offset_z + z, &box_limit);
                    }
                }
            }
        }

        // Layer Y=4 (ceiling)
        for z in 1..=8 {
            for x in 1..=5 {
                // Ladder continues through ceiling
                if x == 3 && z == 7 {
                    p.add_block(
                        chunk,
                        ladder,
                        offset_x + x,
                        basement_top_y - 1,
                        offset_z + z,
                        &box_limit,
                    );
                } else {
                    p.add_block(
                        chunk,
                        stone_brick,
                        offset_x + x,
                        basement_top_y - 1,
                        offset_z + z,
                        &box_limit,
                    );
                }
            }
        }

        // Furnishings
        let y_floor = basement_top_y - 4;

        // Chest at (1, 6) - TODO: loot table
        p.add_block(
            chunk,
            Block::CHEST.default_state,
            offset_x + 1,
            y_floor,
            offset_z + 6,
            &box_limit,
        );

        // Brewing stand at (5, 6)
        p.add_block(
            chunk,
            Block::BREWING_STAND.default_state,
            offset_x + 5,
            y_floor + 1,
            offset_z + 6,
            &box_limit,
        );

        // Water cauldron at (5, 4)
        p.add_block(
            chunk,
            Block::WATER_CAULDRON.default_state,
            offset_x + 5,
            y_floor,
            offset_z + 4,
            &box_limit,
        );

        // Potted cactus at (5, 7) - the golden apple hint!
        p.add_block(
            chunk,
            Block::POTTED_CACTUS.default_state,
            offset_x + 5,
            y_floor + 1,
            offset_z + 7,
            &box_limit,
        );

        // Ladder in shaft position going up
        for dy in 0..=4 {
            p.add_block(
                chunk,
                ladder,
                offset_x + 3,
                basement_top_y - 1 - dy,
                offset_z + 7,
                &box_limit,
            );
        }

        // TODO: Entity spawning (villager at x=2, zombie villager at x=4)
        // TODO: Loot table for chest
        // TODO: Sign with brewing instructions
    }
}

impl StructurePieceBase for IglooPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        if !self
            .shiftable_structure_piece
            .adjust_to_average_height(chunk)
        {
            return;
        }

        let dome_floor_y = self.shiftable_structure_piece.piece.bounding_box.min.y;

        // Place the dome (uses bounding box for coordinate transforms)
        self.place_dome(chunk);

        // Place basement if exists - extend bounding box temporarily for basement blocks
        if self.has_basement {
            // Calculate how deep the basement goes
            let basement_depth = (self.ladder_segments as i32 * SHAFT_HEIGHT) + BASEMENT_HEIGHT + 1;

            // Extend bounding box to include basement area
            self.shiftable_structure_piece.piece.bounding_box.min.y -= basement_depth;
            self.shiftable_structure_piece.piece.bounding_box.min.z -= 2; // basement offset_z = -2

            self.place_ladder_shaft(chunk, dome_floor_y);
            self.place_basement(chunk, dome_floor_y);
        }
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}
