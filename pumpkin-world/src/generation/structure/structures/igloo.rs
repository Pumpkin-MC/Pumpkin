//! Igloo structure generator for snowy biomes.
//!
//! Generates igloos matching vanilla Minecraft behavior using NBT templates:
//! - Snow block dome with interior furnishings (igloo/top.nbt)
//! - Optional basement (50% chance) with ladder shaft and secret room
//!   - Ladder segments (igloo/middle.nbt) repeated 4-11 times
//!   - Basement room (igloo/bottom.nbt)

use std::sync::Arc;

use pumpkin_util::{math::position::BlockPos, math::vector3::Vector3, random::RandomGenerator};
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
            template::{BlockRotation, StructureTemplate, get_template, place_template},
        },
    },
};

use pumpkin_util::random::RandomImpl;

/// Igloo dome dimensions (from vanilla igloo/top.nbt).
const DOME_WIDTH: i32 = 7;
const DOME_HEIGHT: i32 = 5;
const DOME_DEPTH: i32 = 8;

/// Height of each ladder shaft segment (from vanilla igloo/middle.nbt).
const SHAFT_HEIGHT: i32 = 3;

/// Basement room dimensions (from vanilla igloo/bottom.nbt).
const BASEMENT_HEIGHT: i32 = 6;
const BASEMENT_DEPTH: i32 = 9;

/// Vanilla pivot offset for igloo/top template alignment.
const PIVOT_OFFSET_X: i32 = 3;
const PIVOT_OFFSET_Z: i32 = 5;

/// Offset from dome to shaft entrance (vanilla: OFFSETS_FROM_TOP for middle).
const SHAFT_OFFSET_X: i32 = 2;
const SHAFT_OFFSET_Z: i32 = 4;

/// Offset from dome to basement (vanilla: OFFSETS_FROM_TOP for bottom).
const BASEMENT_OFFSET_X: i32 = 0;
const BASEMENT_OFFSET_Z: i32 = -2;

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
        let rotation_index = context.random.next_bounded_i32(4) as u8;
        let rotation = BlockRotation::from_index(rotation_index);

        // 2. Basement check (vanilla: random.nextDouble() < 0.5)
        let has_basement = context.random.next_f64() < 0.5;

        // 3. Ladder segments: 4-11 if has basement (vanilla: random.nextInt(8) + 4)
        let ladder_segments = if has_basement {
            context.random.next_bounded_i32(8) as u8 + 4
        } else {
            0
        };

        let mut collector = StructurePiecesCollector::default();

        // Load templates from cache
        let top_template = get_template("igloo/top")?;

        let piece = IglooPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::Igloo,
                x,
                64,
                z,
                DOME_WIDTH,
                DOME_HEIGHT,
                DOME_DEPTH,
                rotation.to_axis(),
            ),
            top_template,
            middle_template: if has_basement {
                get_template("igloo/middle")
            } else {
                None
            },
            bottom_template: if has_basement {
                get_template("igloo/bottom")
            } else {
                None
            },
            rotation,
            has_basement,
            ladder_segments,
        };

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
    top_template: Arc<StructureTemplate>,
    middle_template: Option<Arc<StructureTemplate>>,
    bottom_template: Option<Arc<StructureTemplate>>,
    rotation: BlockRotation,
    has_basement: bool,
    ladder_segments: u8,
}

impl StructurePieceBase for IglooPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        let origin = self.shiftable_structure_piece.piece.bounding_box.min;

        // Vanilla samples height at the entrance position (3, 0, 5 in template space)
        let (entrance_x, entrance_z) = self.rotation.rotate_offset(3, 5);
        let surface_y = chunk.get_top_y(
            &pumpkin_util::HeightMap::WorldSurfaceWg,
            origin.x + entrance_x,
            origin.z + entrance_z,
        );

        // Place floor at surface - 1 (so the floor block is at ground level)
        let offset = surface_y - 1 - origin.y;
        self.shiftable_structure_piece.piece.bounding_box.min.y += offset;
        self.shiftable_structure_piece.piece.bounding_box.max.y += offset;

        let origin = self.shiftable_structure_piece.piece.bounding_box.min;
        let dome_floor_y = origin.y;

        // Place dome at origin (no offset)
        place_template(
            chunk,
            &self.top_template,
            origin,
            (0, 0),
            self.rotation,
            false,
        );

        // Place basement components if present
        if self.has_basement {
            // Extend bounding box to include basement area
            let basement_depth = (self.ladder_segments as i32 * SHAFT_HEIGHT) + BASEMENT_HEIGHT + 1;
            self.shiftable_structure_piece.piece.bounding_box.min.y -= basement_depth;

            let (_, bz) = self
                .rotation
                .rotate_offset(BASEMENT_OFFSET_X, BASEMENT_OFFSET_Z);
            if bz < 0 {
                self.shiftable_structure_piece.piece.bounding_box.min.z += bz;
            } else {
                self.shiftable_structure_piece.piece.bounding_box.max.z += BASEMENT_DEPTH;
            }

            // Place ladder shaft segments
            if let Some(middle) = &self.middle_template {
                for segment in 0..self.ladder_segments {
                    let segment_y = dome_floor_y - 1 - (segment as i32 * SHAFT_HEIGHT);
                    let shaft_origin = Vector3::new(origin.x, segment_y, origin.z);
                    place_template(
                        chunk,
                        middle,
                        shaft_origin,
                        (SHAFT_OFFSET_X, SHAFT_OFFSET_Z),
                        self.rotation,
                        false,
                    );
                }
            }

            // Place basement
            if let Some(bottom) = &self.bottom_template {
                let total_shaft_depth = self.ladder_segments as i32 * SHAFT_HEIGHT;
                let basement_y = dome_floor_y - total_shaft_depth - BASEMENT_HEIGHT + 1;
                let basement_origin = Vector3::new(origin.x, basement_y, origin.z);
                place_template(
                    chunk,
                    bottom,
                    basement_origin,
                    (BASEMENT_OFFSET_X, BASEMENT_OFFSET_Z),
                    self.rotation,
                    false,
                );
            }
        }
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}
