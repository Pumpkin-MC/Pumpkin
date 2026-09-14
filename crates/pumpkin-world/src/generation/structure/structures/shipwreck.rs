use std::sync::Arc;

use pumpkin_data::{Mirror, Rotation};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    HeightMap,
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl, hash_block_pos, legacy_rand::LegacyRand},
};

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_center_x, get_center_z, start_block_x, start_block_z},
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition, WorldPortalExt,
            },
            template::{BlockStateResolver, StructureTemplate, get_template},
        },
    },
};

const NUMBER_OF_BLOCKS_ALLOWED_IN_WORLD_GEN_REGION: i32 = 32;
pub const PIVOT: Vector3<i32> = Vector3::new(4, 0, 15);

const STRUCTURE_LOCATION_BEACHED: &[&str] = &[
    "shipwreck/with_mast",
    "shipwreck/sideways_full",
    "shipwreck/sideways_fronthalf",
    "shipwreck/sideways_backhalf",
    "shipwreck/rightsideup_full",
    "shipwreck/rightsideup_fronthalf",
    "shipwreck/rightsideup_backhalf",
    "shipwreck/with_mast_degraded",
    "shipwreck/rightsideup_full_degraded",
    "shipwreck/rightsideup_fronthalf_degraded",
    "shipwreck/rightsideup_backhalf_degraded",
];

const STRUCTURE_LOCATION_OCEAN: &[&str] = &[
    "shipwreck/with_mast",
    "shipwreck/upsidedown_full",
    "shipwreck/upsidedown_fronthalf",
    "shipwreck/upsidedown_backhalf",
    "shipwreck/sideways_full",
    "shipwreck/sideways_fronthalf",
    "shipwreck/sideways_backhalf",
    "shipwreck/rightsideup_full",
    "shipwreck/rightsideup_fronthalf",
    "shipwreck/rightsideup_backhalf",
    "shipwreck/with_mast_degraded",
    "shipwreck/upsidedown_full_degraded",
    "shipwreck/upsidedown_fronthalf_degraded",
    "shipwreck/upsidedown_backhalf_degraded",
    "shipwreck/sideways_full_degraded",
    "shipwreck/sideways_fronthalf_degraded",
    "shipwreck/sideways_backhalf_degraded",
    "shipwreck/rightsideup_full_degraded",
    "shipwreck/rightsideup_fronthalf_degraded",
    "shipwreck/rightsideup_backhalf_degraded",
];

const MARKERS_TO_LOOT: &[(&str, &str)] = &[
    ("map_chest", "minecraft:chests/shipwreck_map"),
    ("treasure_chest", "minecraft:chests/shipwreck_treasure"),
    ("supply_chest", "minecraft:chests/shipwreck_supply"),
];

pub struct ShipwreckGenerator {
    pub is_beached: bool,
}

impl StructureGenerator for ShipwreckGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let center_x = get_center_x(context.chunk_x);
        let center_z = get_center_z(context.chunk_z);

        let center_y = if self.is_beached {
            context
                .height_sampler
                .as_deref_mut()
                .map_or(64, |s| s.estimate_height(center_x, center_z))
        } else {
            context
                .height_sampler
                .as_deref_mut()
                .map_or(context.sea_level, |s| {
                    s.estimate_ocean_floor_height(center_x, center_z)
                })
        };

        let rotation_idx = context.random.next_bounded_i32(4) as u8;
        let rotation = Rotation::from_index(rotation_idx);

        let start_x = start_block_x(context.chunk_x);
        let start_z = start_block_z(context.chunk_z);
        let position = Vector3::new(start_x, 90, start_z);

        let pool = if self.is_beached {
            STRUCTURE_LOCATION_BEACHED
        } else {
            STRUCTURE_LOCATION_OCEAN
        };
        let template_idx = context.random.next_bounded_i32(pool.len() as i32) as usize;
        let template_name = pool[template_idx];
        let template = get_template(template_name)?;

        let mut piece = ShipwreckPiece::new(template, position, rotation, self.is_beached);

        if piece.is_too_big_to_fit_in_world_gen_region() {
            let bb = piece.piece.bounding_box;
            let height = if self.is_beached {
                let min_y = context.height_sampler.as_deref_mut().map_or(64, |sampler| {
                    let c0 = sampler.estimate_height(bb.min.x, bb.min.z);
                    let c1 = sampler.estimate_height(bb.min.x, bb.max.z);
                    let c2 = sampler.estimate_height(bb.max.x, bb.min.z);
                    let c3 = sampler.estimate_height(bb.max.x, bb.max.z);
                    c0.min(c1).min(c2).min(c3)
                });
                piece.calculate_beached_position(min_y, &mut context.random)
            } else if let Some(sampler) = context.height_sampler.as_deref_mut() {
                let c0 = sampler.estimate_height(bb.min.x, bb.min.z);
                let c1 = sampler.estimate_height(bb.min.x, bb.max.z);
                let c2 = sampler.estimate_height(bb.max.x, bb.min.z);
                let c3 = sampler.estimate_height(bb.max.x, bb.max.z);
                (c0 + c1 + c2 + c3) / 4
            } else {
                context.sea_level
            };

            piece.adjust_position_height(height);
        }

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(piece));

        Some(StructurePosition {
            start_pos: BlockPos::new(center_x, center_y, center_z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct ShipwreckPiece {
    piece: StructurePiece,
    template: Arc<StructureTemplate>,
    rotation: Rotation,
    template_position: Vector3<i32>,
    is_beached: bool,
    height_adjusted: bool,
}

impl ShipwreckPiece {
    #[must_use]
    pub fn new(
        template: Arc<StructureTemplate>,
        position: Vector3<i32>,
        rotation: Rotation,
        is_beached: bool,
    ) -> Self {
        let bounding_box = StructureTemplate::get_bounding_box_static(
            position,
            rotation,
            PIVOT,
            Mirror::None,
            template.size,
        );

        Self {
            piece: StructurePiece::new(StructurePieceType::Shipwreck, bounding_box, 0),
            template,
            rotation,
            template_position: position,
            is_beached,
            height_adjusted: false,
        }
    }

    #[must_use]
    pub fn is_too_big_to_fit_in_world_gen_region(&self) -> bool {
        self.template.size.x > NUMBER_OF_BLOCKS_ALLOWED_IN_WORLD_GEN_REGION
            || self.template.size.y > NUMBER_OF_BLOCKS_ALLOWED_IN_WORLD_GEN_REGION
    }

    #[must_use]
    pub fn calculate_beached_position(&self, min_y: i32, random: &mut RandomGenerator) -> i32 {
        min_y - self.template.size.y / 2 - random.next_bounded_i32(3)
    }

    pub fn adjust_position_height(&mut self, new_height: i32) {
        self.height_adjusted = true;
        self.template_position.y = new_height;
        self.piece.bounding_box = StructureTemplate::get_bounding_box_static(
            self.template_position,
            self.rotation,
            PIVOT,
            Mirror::None,
            self.template.size,
        );
    }

    fn handle_data_marker(
        marker_id: &str,
        position: Vector3<i32>,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
    ) {
        let Some((_, loot_table)) = MARKERS_TO_LOOT.iter().find(|(k, _)| *k == marker_id) else {
            return;
        };

        let chest_pos = Vector3::new(position.x, position.y - 1, position.z);
        if !chunk_box.contains_pos(&chest_pos) {
            return;
        }

        if let Some(existing) = chunk.pending_block_entities.iter_mut().find(|nbt| {
            nbt.get_int("x") == Some(chest_pos.x)
                && nbt.get_int("y") == Some(chest_pos.y)
                && nbt.get_int("z") == Some(chest_pos.z)
        }) {
            existing.put_string("LootTable", (*loot_table).to_string());
            let mut rng =
                LegacyRand::from_seed(hash_block_pos(chest_pos.x, chest_pos.y, chest_pos.z) as u64);
            existing.put_long("LootTableSeed", rng.next_i64());
        } else {
            let mut chest_nbt = NbtCompound::new();
            chest_nbt.put_string("id", "minecraft:chest".to_string());
            chest_nbt.put_int("x", chest_pos.x);
            chest_nbt.put_int("y", chest_pos.y);
            chest_nbt.put_int("z", chest_pos.z);
            chest_nbt.put_string("LootTable", (*loot_table).to_string());
            let mut rng =
                LegacyRand::from_seed(hash_block_pos(chest_pos.x, chest_pos.y, chest_pos.z) as u64);
            chest_nbt.put_long("LootTableSeed", rng.next_i64());
            chunk.add_block_entity(chest_nbt);
        }
    }
}

impl StructurePieceBase for ShipwreckPiece {
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
        _block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        if !self.height_adjusted && !self.is_too_big_to_fit_in_world_gen_region() {
            let heightmap_type = if self.is_beached {
                HeightMap::WorldSurfaceWg
            } else {
                HeightMap::OceanFloorWg
            };

            let template_size = self.template.size;
            let base_size = template_size.x * template_size.z;

            let (mean, min_y) = if base_size == 0 {
                let h = chunk.get_top_y(
                    &heightmap_type,
                    self.template_position.x,
                    self.template_position.z,
                );
                (h, h)
            } else {
                let mut min_y = i32::MAX;
                let mut total = 0;
                for x in self.template_position.x..self.template_position.x + template_size.x {
                    for z in self.template_position.z..self.template_position.z + template_size.z {
                        let h = chunk.get_top_y(&heightmap_type, x, z);
                        total += h;
                        min_y = min_y.min(h);
                    }
                }
                (total / base_size, min_y)
            };

            let target_y = if self.is_beached {
                self.calculate_beached_position(min_y, random)
            } else {
                mean
            };

            self.adjust_position_height(target_y);
        }

        // Place non-marker blocks
        let box_limit = self.piece.bounding_box;
        for block in &self.template.blocks {
            let palette_entry = &self.template.palette[block.state as usize];

            if palette_entry.name == "minecraft:structure_block"
                || palette_entry.name == "minecraft:air"
                || palette_entry.name == "minecraft:structure_void"
                || palette_entry.name == "structure_void"
            {
                continue;
            }

            let rel_pos = StructureTemplate::transform_block_pos(
                block.pos,
                Mirror::None,
                self.rotation,
                PIVOT,
            );
            let world_pos = self.template_position + rel_pos;

            if !box_limit.contains_pos(&world_pos) || !chunk_box.contains_pos(&world_pos) {
                continue;
            }

            let Some(mut state) =
                BlockStateResolver::resolve(palette_entry, self.rotation, Mirror::None)
            else {
                continue;
            };

            if chunk.get_block_state(&world_pos).to_block_id() == pumpkin_data::Block::WATER.id {
                let mut modified_entry = palette_entry.clone();
                if let Some((_, waterlogged)) = modified_entry
                    .properties
                    .iter_mut()
                    .find(|(name, _)| name == "waterlogged")
                {
                    *waterlogged = "true".to_string();
                    if let Some(waterlogged_state) =
                        BlockStateResolver::resolve(&modified_entry, self.rotation, Mirror::None)
                    {
                        state = waterlogged_state;
                    }
                }
            }

            chunk.set_block_state(world_pos.x, world_pos.y, world_pos.z, state);

            let final_block = pumpkin_data::Block::from_id(state.id.to_block_id());
            let block_entity_id =
                crate::generation::structure::template::get_block_entity_id(final_block.name);
            if block.nbt.is_some() || block_entity_id.is_some() {
                let fallback_id = block_entity_id.unwrap_or(final_block.name);
                let mut placed_nbt = NbtCompound::new();

                placed_nbt.put_string("id", fallback_id.to_string());
                placed_nbt.put_int("x", world_pos.x);
                placed_nbt.put_int("y", world_pos.y);
                placed_nbt.put_int("z", world_pos.z);

                if let Some(template_nbt) = &block.nbt {
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

        // Process data markers
        for block in &self.template.blocks {
            let palette_entry = &self.template.palette[block.state as usize];
            if palette_entry.name == "minecraft:structure_block"
                && let Some(nbt) = &block.nbt
            {
                let mode = nbt.get_string("mode").unwrap_or("");
                if mode == "DATA" {
                    let metadata = nbt.get_string("metadata").unwrap_or("");
                    let rel_pos = StructureTemplate::transform_block_pos(
                        block.pos,
                        Mirror::None,
                        self.rotation,
                        PIVOT,
                    );
                    let world_pos = self.template_position + rel_pos;
                    Self::handle_data_marker(metadata, world_pos, chunk, chunk_box);
                }
            }
        }
    }
}
