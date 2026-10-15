use std::sync::Arc;

use pumpkin_data::{Block, BlockDirection, BlockState, Mirror, Rotation};
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{
        RandomDeriverImpl, RandomGenerator, RandomImpl, hash_block_pos, legacy_rand::LegacyRand,
    },
};

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{start_block_x, start_block_z},
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition, WorldPortalExt,
            },
            template::{
                BlockStateResolver, StructurePlaceSettings, StructureTemplate, get_template,
                processor::{IgnoredBlock, ProcessorContext, StructureProcessor},
            },
        },
    },
};

pub const FOSSILS: [&str; 14] = [
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
/// From `nether_fossil.json`: uniform(absolute=32, `below_top=2`).
/// In vanilla Nether, generator gen depth is 128:
/// `below_top=2`: height - 1 + `min_y` - offset = 128 - 1 + 0 - 2 = 125.
const HEIGHT_MIN: i32 = 32;
const HEIGHT_MAX: i32 = 125;

pub struct NetherFossilGenerator;

impl StructureGenerator for NetherFossilGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        // Vanilla random call order (NetherFossilStructure.java):
        // 1. nextInt(16) for X offset within chunk
        // 2. nextInt(16) for Z offset within chunk
        // 3. height.sample(random, generationContext) for initial Y (uniform 32..125)
        // 4. Column scan downward to seaLevel (no random calls)
        //    If y <= seaLevel: return empty
        // 5. Rotation.getRandom(random) - nextInt(4)
        // 6. Util.getRandom(FOSSILS, random) - nextInt(14)

        let x = start_block_x(context.chunk_x) + context.random.next_bounded_i32(16);
        let z = start_block_z(context.chunk_z) + context.random.next_bounded_i32(16);

        let structure = context
            .structure_key
            .map(|key| pumpkin_data::structures::Structure::get(&key));

        let initial_y = if let Some(hp) = structure.and_then(|s| s.start_height) {
            hp.get(&mut context.random, context.min_y as i8, context.height)
        } else {
            let height_range = HEIGHT_MAX - HEIGHT_MIN + 1;
            HEIGHT_MIN + context.random.next_bounded_i32(height_range)
        };

        let mut y = initial_y;
        if let Some(sampler) = context.height_sampler.as_deref_mut() {
            let mut checked_column = false;
            while y > context.sea_level {
                let Some(current) = sampler.sample_column_block(x, z, y) else {
                    break;
                };
                checked_column = true;
                y -= 1;
                let below = sampler
                    .sample_column_block(x, z, y)
                    .unwrap_or(Block::AIR.default_state);
                if current.is_air()
                    && (Block::from_state_id(below.id) == &Block::SOUL_SAND
                        || below.is_side_solid(BlockDirection::Up))
                {
                    break;
                }
            }

            if checked_column && y <= context.sea_level {
                return None;
            }
        }

        let rotation_index = context.random.next_bounded_i32(4) as u8;
        let rotation = Rotation::from_index(rotation_index);

        let template_index = context.random.next_bounded_i32(FOSSILS.len() as i32) as usize;
        let template_name = FOSSILS[template_index];

        let template = get_template(template_name)?;
        let position = Vector3::new(x, y, z);

        let mut collector = StructurePiecesCollector::default();

        let piece = NetherFossilPiece::new(template, template_name.to_string(), position, rotation);

        collector.add_piece(Box::new(piece));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, y, z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct NetherFossilPiece {
    pub piece: StructurePiece,
    pub template: Arc<StructureTemplate>,
    pub template_name: String,
    pub place_settings: StructurePlaceSettings,
    pub template_position: Vector3<i32>,
}

impl NetherFossilPiece {
    #[must_use]
    pub fn new(
        template: Arc<StructureTemplate>,
        template_name: String,
        template_position: Vector3<i32>,
        rotation: Rotation,
    ) -> Self {
        let place_settings = make_settings(rotation);
        let bounding_box = template.get_bounding_box(&place_settings, template_position);

        Self {
            piece: StructurePiece::new(StructurePieceType::NetherFossil, bounding_box, 0),
            template,
            template_name,
            place_settings,
            template_position,
        }
    }

    fn place_blocks(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let rotation = self.place_settings.get_rotation();
        let mirror = self.place_settings.get_mirror();
        let pivot = self.place_settings.get_rotation_pivot();

        let mut context_rng = LegacyRand::from_seed(hash_block_pos(
            self.template_position.x,
            self.template_position.y,
            self.template_position.z,
        ) as u64);
        let mut context = ProcessorContext::new(
            self.template_position,
            self.place_settings.get_processors(),
            &mut context_rng,
        );

        for block in &self.template.blocks {
            let palette_entry = &self.template.palette[block.state as usize];

            let mut block_entity_nbt = block.nbt.clone();
            let placed_entry = palette_entry.clone();

            let Some(state) = BlockStateResolver::resolve(&placed_entry, rotation, mirror) else {
                continue;
            };

            let local_pos =
                StructureTemplate::transform_block_pos(block.pos, mirror, rotation, pivot);
            let world_pos = self.template_position + local_pos;

            if !chunk_box.contains_pos(&world_pos) {
                continue;
            }

            let mut processed_state = Some(state);
            let mut capped_idx = 0;
            for processor in self.place_settings.get_processors() {
                let Some(current_state) = processed_state else {
                    break;
                };
                processed_state = processor.process_with_context(
                    chunk,
                    world_pos,
                    current_state,
                    &mut block_entity_nbt,
                    &mut context,
                    &mut capped_idx,
                    &mut context_rng,
                );
            }

            let Some(final_state) = processed_state else {
                continue;
            };

            chunk.set_block_state(world_pos.x, world_pos.y, world_pos.z, final_state);
        }
    }

    fn place_dried_ghast(
        chunk: &mut ProtoChunk,
        seed: i64,
        fossil_bb: &BlockBox,
        chunk_bb: &BlockBox,
    ) {
        use pumpkin_util::random::xoroshiro128::Xoroshiro;

        let center_x = i32::midpoint(fossil_bb.min.x, fossil_bb.max.x);
        let center_y = i32::midpoint(fossil_bb.min.y, fossil_bb.max.y);
        let center_z = i32::midpoint(fossil_bb.min.z, fossil_bb.max.z);

        let mut rng = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed as u64));
        let splitter = rng.next_splitter();
        let mut positional_random = splitter.split_pos(center_x, center_y, center_z);

        if positional_random.next_f32() < 0.5 {
            let x_span = (fossil_bb.max.x - fossil_bb.min.x + 1).max(1);
            let z_span = (fossil_bb.max.z - fossil_bb.min.z + 1).max(1);
            let x = fossil_bb.min.x + positional_random.next_bounded_i32(x_span);
            let y = fossil_bb.min.y;
            let z = fossil_bb.min.z + positional_random.next_bounded_i32(z_span);
            let random_pos = Vector3::new(x, y, z);

            let block_at = chunk.get_block_state(&random_pos);
            if BlockState::from_id(block_at).is_air() && chunk_bb.contains_pos(&random_pos) {
                let rot_idx = positional_random.next_bounded_i32(4) as u8;
                let rot = Rotation::from_index(rot_idx);
                let state = Block::DRIED_GHAST.default_state.rotate(rot);
                chunk.set_block_state(random_pos.x, random_pos.y, random_pos.z, state);
            }
        }
    }
}

impl StructurePieceBase for NetherFossilPiece {
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
        seed: i64,
        chunk_box: &BlockBox,
    ) {
        let fossil_bb = self.piece.bounding_box;
        let mut enlarged_box = *chunk_box;
        enlarged_box.encompass(&fossil_bb);

        self.place_blocks(chunk, &enlarged_box);
        Self::place_dried_ghast(chunk, seed, &fossil_bb, chunk_box);
    }
}

fn make_settings(rotation: Rotation) -> StructurePlaceSettings {
    StructurePlaceSettings::new()
        .set_rotation(rotation)
        .set_mirror(Mirror::None)
        .add_processor(StructureProcessor::BlockIgnore(vec![
            IgnoredBlock {
                block_id: Block::STRUCTURE_BLOCK.id,
                properties: None,
            },
            IgnoredBlock {
                block_id: Block::AIR.id,
                properties: None,
            },
        ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::structure::structures::HeightSampler;
    use pumpkin_data::Block;

    struct MockColumnSampler {
        ground_y: i32,
    }

    impl HeightSampler for MockColumnSampler {
        fn estimate_height(&mut self, _block_x: i32, _block_z: i32) -> i32 {
            self.ground_y
        }

        fn sample_column_block(
            &mut self,
            _block_x: i32,
            _block_z: i32,
            y: i32,
        ) -> Option<&'static BlockState> {
            if y > self.ground_y {
                Some(Block::AIR.default_state)
            } else {
                Some(Block::NETHERRACK.default_state)
            }
        }
    }

    #[test]
    fn nether_fossil_never_places_on_roof() {
        for seed in 0..100 {
            let mut sampler = MockColumnSampler { ground_y: 60 };
            let context = StructureGeneratorContext {
                seed,
                chunk_x: 0,
                chunk_z: 0,
                random: crate::generation::structure::structures::create_chunk_random(seed, 0, 0),
                sea_level: 32,
                min_y: 0,
                height: 128,
                height_sampler: Some(&mut sampler),
                structure_key: Some(pumpkin_data::structures::StructureKeys::NetherFossil),
            };
            if let Some(pos) = NetherFossilGenerator.get_structure_position(context) {
                assert!(
                    pos.start_pos.0.y <= 125,
                    "Fossil placed at {}, above logical nether height!",
                    pos.start_pos.0.y
                );
                assert!(
                    pos.start_pos.0.y >= 32,
                    "Fossil placed at {}, below sea level!",
                    pos.start_pos.0.y
                );
            }
        }
    }

    #[test]
    fn nether_fossil_rejects_when_no_ground_above_sea_level() {
        let mut sampler = MockColumnSampler { ground_y: 20 }; // below sea_level (32)
        let context = StructureGeneratorContext {
            seed: 42,
            chunk_x: 0,
            chunk_z: 0,
            random: crate::generation::structure::structures::create_chunk_random(42, 0, 0),
            sea_level: 32,
            min_y: 0,
            height: 128,
            height_sampler: Some(&mut sampler),
            structure_key: Some(pumpkin_data::structures::StructureKeys::NetherFossil),
        };
        assert!(
            NetherFossilGenerator
                .get_structure_position(context)
                .is_none()
        );
    }
}
