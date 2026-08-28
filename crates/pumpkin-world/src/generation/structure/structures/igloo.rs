use std::sync::Arc;

use pumpkin_data::{Block, BlockState, Mirror, Rotation};
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl, hash_block_pos, legacy_rand::LegacyRand},
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

pub const GENERATION_HEIGHT: i32 = 90;
pub const STRUCTURE_LOCATION_IGLOO: &str = "igloo/top";
pub const STRUCTURE_LOCATION_LADDER: &str = "igloo/middle";
pub const STRUCTURE_LOCATION_LABORATORY: &str = "igloo/bottom";

#[must_use]
pub const fn get_pivot(location: &str) -> Vector3<i32> {
    if const_str_eq(location, STRUCTURE_LOCATION_IGLOO) {
        Vector3::new(3, 5, 5)
    } else if const_str_eq(location, STRUCTURE_LOCATION_LADDER) {
        Vector3::new(1, 3, 1)
    } else if const_str_eq(location, STRUCTURE_LOCATION_LABORATORY) {
        Vector3::new(3, 6, 7)
    } else {
        Vector3::new(0, 0, 0)
    }
}

#[must_use]
pub const fn get_offset(location: &str) -> Vector3<i32> {
    if const_str_eq(location, STRUCTURE_LOCATION_IGLOO) {
        Vector3::new(0, 0, 0)
    } else if const_str_eq(location, STRUCTURE_LOCATION_LADDER) {
        Vector3::new(2, -3, 4)
    } else if const_str_eq(location, STRUCTURE_LOCATION_LABORATORY) {
        Vector3::new(0, -3, -2)
    } else {
        Vector3::new(0, 0, 0)
    }
}

const fn const_str_eq(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

pub fn add_pieces(
    collector: &mut StructurePiecesCollector,
    position: Vector3<i32>,
    rotation: Rotation,
    random: &mut RandomGenerator,
) {
    if random.next_f64() < 0.5 {
        let depth = random.next_bounded_i32(8) + 4;
        if let Some(template) = get_template(STRUCTURE_LOCATION_LABORATORY) {
            collector.add_piece(Box::new(IglooPiece::new(
                template,
                STRUCTURE_LOCATION_LABORATORY.to_string(),
                position,
                rotation,
                depth * 3,
            )));
        }

        for i in 0..(depth - 1) {
            if let Some(template) = get_template(STRUCTURE_LOCATION_LADDER) {
                collector.add_piece(Box::new(IglooPiece::new(
                    template,
                    STRUCTURE_LOCATION_LADDER.to_string(),
                    position,
                    rotation,
                    i * 3,
                )));
            }
        }
    }

    if let Some(template) = get_template(STRUCTURE_LOCATION_IGLOO) {
        collector.add_piece(Box::new(IglooPiece::new(
            template,
            STRUCTURE_LOCATION_IGLOO.to_string(),
            position,
            rotation,
            0,
        )));
    }
}

pub struct IglooGenerator;

impl StructureGenerator for IglooGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let block_pos = Vector3::new(
            start_block_x(context.chunk_x),
            GENERATION_HEIGHT,
            start_block_z(context.chunk_z),
        );
        let rotation_idx = context.random.next_bounded_i32(4) as u8;
        let rotation = Rotation::from_index(rotation_idx);

        let mut collector = StructurePiecesCollector::default();
        add_pieces(&mut collector, block_pos, rotation, &mut context.random);

        Some(StructurePosition {
            start_pos: BlockPos::new(block_pos.x, block_pos.y, block_pos.z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct IglooPiece {
    pub piece: StructurePiece,
    pub template: Arc<StructureTemplate>,
    pub template_name: String,
    pub place_settings: StructurePlaceSettings,
    pub template_position: Vector3<i32>,
}

impl IglooPiece {
    #[must_use]
    pub fn new(
        template: Arc<StructureTemplate>,
        template_name: String,
        position: Vector3<i32>,
        rotation: Rotation,
        depth: i32,
    ) -> Self {
        let place_settings = make_settings(rotation, &template_name);
        let template_position = make_position(&template_name, position, depth);
        let bounding_box = template.get_bounding_box(&place_settings, template_position);

        Self {
            piece: StructurePiece::new(StructurePieceType::Igloo, bounding_box, 0),
            template,
            template_name,
            place_settings,
            template_position,
        }
    }

    #[expect(clippy::too_many_lines)]
    fn place_blocks(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        random: &mut RandomGenerator,
    ) {
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

            let local_pos =
                StructureTemplate::transform_block_pos(block.pos, mirror, rotation, pivot);
            let world_pos = self.template_position + local_pos;

            if !chunk_box.contains_pos(&world_pos) {
                continue;
            }

            if palette_entry.name == "minecraft:structure_block"
                && let Some(nbt) = &block.nbt
            {
                let mode = nbt.get_string("mode").unwrap_or("");
                let metadata = nbt.get_string("metadata").unwrap_or("");
                if mode == "DATA" && metadata == "chest" {
                    chunk.set_block_state(
                        world_pos.x,
                        world_pos.y,
                        world_pos.z,
                        Block::AIR.default_state,
                    );
                    let below_pos = world_pos - Vector3::new(0, 1, 0);
                    let mut chest_nbt = pumpkin_nbt::compound::NbtCompound::new();
                    chest_nbt.put_string("id", "minecraft:chest".to_string());
                    chest_nbt.put_int("x", below_pos.x);
                    chest_nbt.put_int("y", below_pos.y);
                    chest_nbt.put_int("z", below_pos.z);
                    chest_nbt.put_string("LootTable", "minecraft:chests/igloo_chest".to_string());
                    chest_nbt.put_long("LootTableSeed", random.next_i64());
                    chunk.add_block_entity(chest_nbt);
                }
                continue;
            }

            let mut block_entity_nbt = block.nbt.clone();
            let placed_entry = palette_entry.clone();

            let Some(state) = BlockStateResolver::resolve(&placed_entry, rotation, mirror) else {
                continue;
            };

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

            let final_block = Block::from_id(final_state.id.to_block_id());
            let block_entity_id =
                crate::generation::structure::template::get_block_entity_id(final_block.name);
            if block_entity_nbt.is_some() || block_entity_id.is_some() {
                let fallback_id = block_entity_id.unwrap_or(final_block.name);
                let mut placed_nbt = pumpkin_nbt::compound::NbtCompound::new();

                placed_nbt.put_string("id", fallback_id.to_string());
                placed_nbt.put_int("x", world_pos.x);
                placed_nbt.put_int("y", world_pos.y);
                placed_nbt.put_int("z", world_pos.z);

                if let Some(template_nbt) = &block_entity_nbt {
                    for (key, value) in &template_nbt.child_tags {
                        if key.as_ref() != "x"
                            && key.as_ref() != "y"
                            && key.as_ref() != "z"
                            && key.as_ref() != "id"
                        {
                            placed_nbt.child_tags.insert(key.clone(), value.clone());
                        }
                    }
                }

                if placed_nbt.get_string("LootTable").is_some()
                    && placed_nbt.get_long("LootTableSeed").is_none()
                {
                    let mut rng =
                        LegacyRand::from_seed(
                            hash_block_pos(world_pos.x, world_pos.y, world_pos.z) as u64,
                        );
                    placed_nbt.put_long("LootTableSeed", rng.next_i64());
                }

                chunk.add_block_entity(placed_nbt);
            }
        }

        crate::generation::structure::template::place_template_entities(
            chunk,
            &self.template,
            self.template_position,
            rotation,
            chunk_box,
        );
    }
}

impl StructurePieceBase for IglooPiece {
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
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let settings = make_settings(self.place_settings.get_rotation(), &self.template_name);
        let offset = get_offset(&self.template_name);
        let entrance_rel = StructureTemplate::calculate_relative_position(
            &settings,
            Vector3::new(3 - offset.x, 0, -offset.z),
        );
        let entrance_pos = self.template_position + entrance_rel;
        let height = chunk.get_top_y(
            &pumpkin_util::HeightMap::WorldSurfaceWg,
            entrance_pos.x,
            entrance_pos.z,
        );

        let old_template_pos = self.template_position;
        self.template_position.y += height - GENERATION_HEIGHT - 1;
        self.piece.bounding_box = self
            .template
            .get_bounding_box(&self.place_settings, self.template_position);

        self.place_blocks(chunk, chunk_box, random);

        if self.template_name == STRUCTURE_LOCATION_IGLOO {
            let trap_door_rel =
                StructureTemplate::calculate_relative_position(&settings, Vector3::new(3, 0, 5));
            let trap_door_pos = self.template_position + trap_door_rel;
            let below_pos = trap_door_pos - Vector3::new(0, 1, 0);
            let below_state = BlockState::from_id(chunk.get_block_state(&below_pos));
            if !below_state.is_air()
                && below_state.id.to_block_id() != Block::LADDER.id
                && chunk_box.contains_pos(&trap_door_pos)
            {
                chunk.set_block_state(
                    trap_door_pos.x,
                    trap_door_pos.y,
                    trap_door_pos.z,
                    Block::SNOW_BLOCK.default_state,
                );
            }
        }

        self.template_position = old_template_pos;
    }
}

fn make_settings(rotation: Rotation, template_location: &str) -> StructurePlaceSettings {
    StructurePlaceSettings::new()
        .set_rotation(rotation)
        .set_mirror(Mirror::None)
        .set_rotation_pivot(get_pivot(template_location))
        .add_processor(StructureProcessor::BlockIgnore(vec![IgnoredBlock {
            block_id: Block::STRUCTURE_BLOCK.id,
            properties: None,
        }]))
}

fn make_position(template_location: &str, position: Vector3<i32>, depth: i32) -> Vector3<i32> {
    position + get_offset(template_location) - Vector3::new(0, depth, 0)
}
