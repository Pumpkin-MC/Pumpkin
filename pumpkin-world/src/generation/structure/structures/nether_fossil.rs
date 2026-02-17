use std::sync::Arc;

use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomDeriverImpl, RandomGenerator, RandomImpl},
};
use tracing::debug;

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{start_block_x, start_block_z},
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

const TEMPLATE_NAMES: [&str; 14] = [
    "nether_fossils/fossil_1",
    "nether_fossils/fossil_2",
    "nether_fossils/fossil_3",
    "nether_fossils/fossil_4",
    "nether_fossils/fossil_5",
    "nether_fossils/fossil_6",
    "nether_fossils/fossil_7",
    "nether_fossils/fossil_8",
    "nether_fossils/fossil_9",
    "nether_fossils/fossil_10",
    "nether_fossils/fossil_11",
    "nether_fossils/fossil_12",
    "nether_fossils/fossil_13",
    "nether_fossils/fossil_14",
];

/// Vanilla height provider bounds for nether fossils.
/// From nether_fossil.json: uniform(absolute=32, below_top=2).
/// Vanilla BelowTop: height - 1 + min_y - offset = 256 - 1 + 0 - 2 = 253.
const HEIGHT_MIN: i32 = 32;
const HEIGHT_MAX: i32 = 253;

pub struct NetherFossilGenerator;

impl StructureGenerator for NetherFossilGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        // Vanilla random call order:
        // 1. nextInt(16) for X offset within chunk
        // 2. nextInt(16) for Z offset within chunk
        // 3. height.get(random) for initial Y (uniform 32..254)
        // 4. Column scan (no random calls)
        // 5. BlockRotation.random(random) - nextInt(4)
        // 6. Util.getRandom(FOSSILS, random) - nextInt(14)

        let x = start_block_x(context.chunk_x) + context.random.next_bounded_i32(16);
        let z = start_block_z(context.chunk_z) + context.random.next_bounded_i32(16);

        let height_range = HEIGHT_MAX - HEIGHT_MIN + 1;
        let initial_y = HEIGHT_MIN + context.random.next_bounded_i32(height_range);

        // Column scan is deferred to place() since we don't have block data here.
        // Consume random in vanilla order for determinism.
        let rotation_index = context.random.next_bounded_i32(4) as u8;
        let rotation = BlockRotation::from_index(rotation_index);

        let template_index = context.random.next_bounded_i32(14) as usize;
        let template_name = TEMPLATE_NAMES[template_index];

        let template = get_template(template_name)?;

        let rotated_size = rotation.transform_size(template.size);

        let mut collector = StructurePiecesCollector::default();

        let piece = NetherFossilPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::NetherFossil,
                x,
                initial_y,
                z,
                rotated_size.x,
                rotated_size.y,
                rotated_size.z,
                rotation.to_axis(),
            ),
            template,
            rotation,
            initial_y,
            sea_level: context.sea_level,
        };

        collector.add_piece(Box::new(piece));

        debug!(
            "Nether fossil candidate at ({}, {}, {}), template={}, rotation={:?}",
            x, initial_y, z, template_name, rotation
        );

        Some(StructurePosition {
            start_pos: BlockPos::new(x, initial_y, z),
            collector: Arc::new(collector.into()),
        })
    }
}

#[derive(Clone)]
struct NetherFossilPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
    template: Arc<StructureTemplate>,
    rotation: BlockRotation,
    initial_y: i32,
    sea_level: i32,
}

impl NetherFossilPiece {
    /// Vanilla column scan: search downward from initial_y for air above (soul sand OR solid block).
    /// Returns the Y of the support block, or None if no valid position found above sea level.
    ///
    /// Mirrors vanilla's pre-decrement loop:
    /// ```java
    /// while (l > k) {
    ///     BlockState lv5 = lv3.getState(l);
    ///     BlockState lv6 = lv3.getState(--l);  // pre-decrement
    ///     if (lv5.isAir() && (lv6.isOf(SOUL_SAND) || lv6.isSideSolidFullSquare(...))) break;
    /// }
    /// if (l <= k) return empty;
    /// ```
    /// After the loop, l is the support block Y. Vanilla rejects if l <= sea_level.
    fn find_placement_y(&self, chunk: &ProtoChunk) -> Option<i32> {
        let origin = self.shiftable_structure_piece.piece.bounding_box.min;
        let mut y = self.initial_y;

        while y > self.sea_level {
            let upper = chunk.get_block_state(&Vector3::new(origin.x, y, origin.z));
            // Pre-decrement: y now points to the lower block (vanilla's --l)
            y -= 1;
            let lower = chunk.get_block_state(&Vector3::new(origin.x, y, origin.z));

            let upper_state = BlockState::from_id(upper.0);
            let lower_state = BlockState::from_id(lower.0);

            if upper_state.is_air()
                && (Block::from_state_id(lower.0) == &Block::SOUL_SAND
                    || lower_state.is_side_solid(BlockDirection::Up))
            {
                break;
            }
        }

        // Vanilla: if (l <= k) return empty
        if y <= self.sea_level { None } else { Some(y) }
    }
}

impl StructurePieceBase for NetherFossilPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, seed: i64) {
        // Vanilla column scan: find air above soul sand or solid block
        let Some(placement_y) = self.find_placement_y(chunk) else {
            debug!(
                "Nether fossil at ({}, {}) failed column scan from Y={}, no valid placement",
                self.shiftable_structure_piece.piece.bounding_box.min.x,
                self.shiftable_structure_piece.piece.bounding_box.min.z,
                self.initial_y
            );
            return;
        };

        // Adjust bounding box to placement Y
        let current_y = self.shiftable_structure_piece.piece.bounding_box.min.y;
        let offset = placement_y - current_y;
        self.shiftable_structure_piece.piece.bounding_box.min.y += offset;
        self.shiftable_structure_piece.piece.bounding_box.max.y += offset;

        let origin = self.shiftable_structure_piece.piece.bounding_box.min;

        debug!(
            "Placing nether fossil at ({}, {}, {}), rotation={:?}",
            origin.x, origin.y, origin.z, self.rotation
        );

        // Vanilla uses IGNORE_AIR_AND_STRUCTURE_BLOCKS processor
        place_template(chunk, &self.template, origin, (0, 0), self.rotation, true);

        // Vanilla: 50% chance to place a dried ghast block at the fossil base.
        // Uses a deterministic random seeded from world seed + bounding box center.
        self.try_place_dried_ghast(chunk, seed);
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}

impl NetherFossilPiece {
    fn try_place_dried_ghast(&self, chunk: &mut ProtoChunk, seed: i64) {
        use pumpkin_util::random::xoroshiro128::Xoroshiro;

        let bbox = self.shiftable_structure_piece.piece.bounding_box;
        let center_x = (bbox.min.x + bbox.max.x) / 2;
        let center_y = (bbox.min.y + bbox.max.y) / 2;
        let center_z = (bbox.min.z + bbox.max.z) / 2;

        // Vanilla: Random.create(world.getSeed()).nextSplitter().split(box.getCenter())
        let mut rng = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed as u64));
        let splitter = rng.next_splitter();
        let mut rng = splitter.split_pos(center_x, center_y, center_z);

        if rng.next_f32() >= 0.5 {
            return;
        }

        let block_count_x = (bbox.max.x - bbox.min.x + 1).max(1);
        let block_count_z = (bbox.max.z - bbox.min.z + 1).max(1);
        let x = bbox.min.x + rng.next_bounded_i32(block_count_x);
        let y = bbox.min.y;
        let z = bbox.min.z + rng.next_bounded_i32(block_count_z);

        // Vanilla: chunkBox.contains(pos) - only place if within current chunk
        let chunk_x = start_block_x(chunk.x);
        let chunk_z = start_block_z(chunk.z);
        if x < chunk_x || x > chunk_x + 15 || z < chunk_z || z > chunk_z + 15 {
            return;
        }

        let block_at = chunk.get_block_state(&Vector3::new(x, y, z));
        if !BlockState::from_id(block_at.0).is_air() {
            return;
        }

        // Place dried ghast with random rotation (vanilla: Blocks.DRIED_GHAST.getDefaultState().rotate())
        // Dried ghast default state is sufficient since rotation is cosmetic
        chunk.set_block_state(x, y, z, Block::DRIED_GHAST.default_state);
    }
}
