//! Igloo structure generator for snowy biomes.
//!
//! Generates igloos matching vanilla Minecraft behavior using NBT templates:
//! - Snow block dome with interior furnishings (igloo/top.nbt)
//! - Optional basement (50% chance) with ladder shaft and secret room
//!   - Ladder segments (igloo/middle.nbt) repeated 4-11 times
//!   - Basement room (igloo/bottom.nbt)

use std::sync::Arc;

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
            template::{BlockRotation, StructureTemplate, get_template},
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
const BASEMENT_WIDTH: i32 = 7;
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

impl IglooPiece {
    /// Places the igloo dome using the template.
    fn place_dome(&self, chunk: &mut ProtoChunk) {
        let origin = self.shiftable_structure_piece.piece.bounding_box.min;

        place_template(
            chunk,
            &self.top_template,
            origin.x,
            origin.y,
            origin.z,
            self.rotation,
        );
    }

    /// Places a single ladder shaft segment.
    fn place_shaft_segment(&self, chunk: &mut ProtoChunk, y: i32) {
        let Some(template) = &self.middle_template else {
            return;
        };

        let origin = self.shiftable_structure_piece.piece.bounding_box.min;

        // Apply rotation to offset
        let (offset_x, offset_z) = rotate_offset(SHAFT_OFFSET_X, SHAFT_OFFSET_Z, self.rotation);

        place_template(
            chunk,
            template,
            origin.x + offset_x,
            y,
            origin.z + offset_z,
            self.rotation,
        );
    }

    /// Places all ladder shaft segments.
    fn place_ladder_shaft(&self, chunk: &mut ProtoChunk, dome_floor_y: i32) {
        for segment in 0..self.ladder_segments {
            let segment_y = dome_floor_y - 1 - (segment as i32 * SHAFT_HEIGHT);
            self.place_shaft_segment(chunk, segment_y);
        }
    }

    /// Places the basement room.
    fn place_basement(&self, chunk: &mut ProtoChunk, dome_floor_y: i32) {
        let Some(template) = &self.bottom_template else {
            return;
        };

        let origin = self.shiftable_structure_piece.piece.bounding_box.min;
        let total_shaft_depth = self.ladder_segments as i32 * SHAFT_HEIGHT;
        let basement_y = dome_floor_y - total_shaft_depth - BASEMENT_HEIGHT + 1;

        // Apply rotation to offset
        let (offset_x, offset_z) =
            rotate_offset(BASEMENT_OFFSET_X, BASEMENT_OFFSET_Z, self.rotation);

        place_template(
            chunk,
            template,
            origin.x + offset_x,
            basement_y,
            origin.z + offset_z,
            self.rotation,
        );
    }
}

impl StructurePieceBase for IglooPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        // Vanilla samples height at the entrance position (3, 0, 5 in template space)
        // and places the floor at surface_height - 1
        let origin = self.shiftable_structure_piece.piece.bounding_box.min;

        // The entrance is at template position (3, 0, 5), apply rotation
        let (entrance_offset_x, entrance_offset_z) = rotate_offset(3, 5, self.rotation);
        let sample_x = origin.x + entrance_offset_x;
        let sample_z = origin.z + entrance_offset_z;

        // Get surface height at entrance position
        let surface_y =
            chunk.get_top_y(&pumpkin_util::HeightMap::WorldSurfaceWg, sample_x, sample_z);

        // Place floor at surface - 1 (so the floor block is at ground level)
        let target_y = surface_y - 1;
        let current_y = origin.y;
        let offset = target_y - current_y;

        self.shiftable_structure_piece.piece.bounding_box.min.y += offset;
        self.shiftable_structure_piece.piece.bounding_box.max.y += offset;

        let dome_floor_y = self.shiftable_structure_piece.piece.bounding_box.min.y;

        // Place the dome
        self.place_dome(chunk);

        // Place basement components if present
        if self.has_basement {
            // Extend bounding box to include basement area for block placement bounds checking
            let basement_depth = (self.ladder_segments as i32 * SHAFT_HEIGHT) + BASEMENT_HEIGHT + 1;
            self.shiftable_structure_piece.piece.bounding_box.min.y -= basement_depth;

            // Extend for basement Z offset
            let (_, offset_z) = rotate_offset(BASEMENT_OFFSET_X, BASEMENT_OFFSET_Z, self.rotation);
            if offset_z < 0 {
                self.shiftable_structure_piece.piece.bounding_box.min.z += offset_z;
            } else {
                self.shiftable_structure_piece.piece.bounding_box.max.z += BASEMENT_DEPTH;
            }

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

/// Rotates an X/Z offset according to the block rotation.
fn rotate_offset(x: i32, z: i32, rotation: BlockRotation) -> (i32, i32) {
    match rotation {
        BlockRotation::None => (x, z),
        BlockRotation::Clockwise90 => (-z, x),
        BlockRotation::Rotate180 => (-x, -z),
        BlockRotation::CounterClockwise90 => (z, -x),
    }
}

/// Places a template at the given world position.
fn place_template(
    chunk: &mut ProtoChunk,
    template: &StructureTemplate,
    world_x: i32,
    world_y: i32,
    world_z: i32,
    rotation: BlockRotation,
) {
    use crate::generation::structure::template::BlockStateResolver;
    use pumpkin_nbt::compound::NbtCompound;

    for block in &template.blocks {
        let palette_entry = &template.palette[block.state as usize];

        // Skip structure void blocks
        if palette_entry.name == "minecraft:structure_void" {
            continue;
        }

        // Resolve block state with rotation
        let Some(state) = BlockStateResolver::resolve(palette_entry, rotation, Default::default())
        else {
            continue;
        };

        // Transform position
        let local_pos = rotation.transform_pos(block.pos, template.size);

        // Calculate world position
        let wx = world_x + local_pos.x;
        let wy = world_y + local_pos.y;
        let wz = world_z + local_pos.z;

        // Place the block
        chunk.set_block_state(wx, wy, wz, state);

        // Handle block entities (furnaces, chests, etc.)
        let block_entity_id = get_block_entity_id(&palette_entry.name);
        if block.nbt.is_some() || block_entity_id.is_some() {
            let block_entity_id = block_entity_id.unwrap_or(&palette_entry.name);
            let mut block_entity_nbt = NbtCompound::new();

            // Set position
            block_entity_nbt.put_int("x", wx);
            block_entity_nbt.put_int("y", wy);
            block_entity_nbt.put_int("z", wz);

            // Set block entity ID
            block_entity_nbt.put_string("id", block_entity_id.to_string());

            // Copy over template NBT data if present
            if let Some(template_nbt) = &block.nbt {
                for (key, value) in &template_nbt.child_tags {
                    // Skip position fields as we've already set them
                    if key != "x" && key != "y" && key != "z" && key != "id" {
                        block_entity_nbt
                            .child_tags
                            .push((key.clone(), value.clone()));
                    }
                }
            }

            chunk.add_pending_block_entity(block_entity_nbt);
        }
    }
}

/// Returns the block entity ID for blocks that require one, or None if not needed.
fn get_block_entity_id(block_name: &str) -> Option<&'static str> {
    match block_name {
        "minecraft:furnace" => Some("minecraft:furnace"),
        "minecraft:chest" => Some("minecraft:chest"),
        "minecraft:trapped_chest" => Some("minecraft:trapped_chest"),
        "minecraft:barrel" => Some("minecraft:barrel"),
        "minecraft:hopper" => Some("minecraft:hopper"),
        "minecraft:dropper" => Some("minecraft:dropper"),
        "minecraft:dispenser" => Some("minecraft:dispenser"),
        "minecraft:brewing_stand" => Some("minecraft:brewing_stand"),
        "minecraft:blast_furnace" => Some("minecraft:blast_furnace"),
        "minecraft:smoker" => Some("minecraft:smoker"),
        "minecraft:shulker_box" => Some("minecraft:shulker_box"),
        "minecraft:bed" => Some("minecraft:bed"),
        "minecraft:sign"
        | "minecraft:oak_sign"
        | "minecraft:spruce_sign"
        | "minecraft:birch_sign"
        | "minecraft:jungle_sign"
        | "minecraft:acacia_sign"
        | "minecraft:dark_oak_sign"
        | "minecraft:mangrove_sign"
        | "minecraft:cherry_sign"
        | "minecraft:bamboo_sign"
        | "minecraft:crimson_sign"
        | "minecraft:warped_sign" => Some("minecraft:sign"),
        "minecraft:hanging_sign" => Some("minecraft:hanging_sign"),
        _ => None,
    }
}

impl BlockRotation {
    /// Converts rotation to axis for ShiftableStructurePiece.
    fn to_axis(self) -> pumpkin_util::math::vector3::Axis {
        match self {
            Self::None | Self::Rotate180 => pumpkin_util::math::vector3::Axis::Z,
            Self::Clockwise90 | Self::CounterClockwise90 => pumpkin_util::math::vector3::Axis::X,
        }
    }
}
