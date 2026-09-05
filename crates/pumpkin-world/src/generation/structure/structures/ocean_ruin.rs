use std::sync::Arc;

use pumpkin_data::{
    Block, BlockState, Mirror, Rotation,
    block_properties::{ChestLikeProperties, HorizontalFacing},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    HeightMap,
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl, hash_block_pos, legacy_rand::LegacyRand},
};

use crate::{
    ProtoChunk,
    generation::structure::{
        piece::StructurePieceType,
        structures::{
            StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
            StructurePiecesCollector, StructurePosition, WorldPortalExt,
        },
        template::{BlockStateResolver, StructurePlaceSettings, StructureTemplate, get_template},
    },
};

const WARM_RUINS: &[&str] = &[
    "underwater_ruin/warm_1",
    "underwater_ruin/warm_2",
    "underwater_ruin/warm_3",
    "underwater_ruin/warm_4",
    "underwater_ruin/warm_5",
    "underwater_ruin/warm_6",
    "underwater_ruin/warm_7",
    "underwater_ruin/warm_8",
];

const BIG_WARM_RUINS: &[&str] = &[
    "underwater_ruin/big_warm_4",
    "underwater_ruin/big_warm_5",
    "underwater_ruin/big_warm_6",
    "underwater_ruin/big_warm_7",
];

const RUINS_BRICK: &[&str] = &[
    "underwater_ruin/brick_1",
    "underwater_ruin/brick_2",
    "underwater_ruin/brick_3",
    "underwater_ruin/brick_4",
    "underwater_ruin/brick_5",
    "underwater_ruin/brick_6",
    "underwater_ruin/brick_7",
    "underwater_ruin/brick_8",
];

const RUINS_CRACKED: &[&str] = &[
    "underwater_ruin/cracked_1",
    "underwater_ruin/cracked_2",
    "underwater_ruin/cracked_3",
    "underwater_ruin/cracked_4",
    "underwater_ruin/cracked_5",
    "underwater_ruin/cracked_6",
    "underwater_ruin/cracked_7",
    "underwater_ruin/cracked_8",
];

const RUINS_MOSSY: &[&str] = &[
    "underwater_ruin/mossy_1",
    "underwater_ruin/mossy_2",
    "underwater_ruin/mossy_3",
    "underwater_ruin/mossy_4",
    "underwater_ruin/mossy_5",
    "underwater_ruin/mossy_6",
    "underwater_ruin/mossy_7",
    "underwater_ruin/mossy_8",
];

const BIG_RUINS_BRICK: &[&str] = &[
    "underwater_ruin/big_brick_1",
    "underwater_ruin/big_brick_2",
    "underwater_ruin/big_brick_3",
    "underwater_ruin/big_brick_8",
];

const BIG_RUINS_CRACKED: &[&str] = &[
    "underwater_ruin/big_cracked_1",
    "underwater_ruin/big_cracked_2",
    "underwater_ruin/big_cracked_3",
    "underwater_ruin/big_cracked_8",
];

const BIG_RUINS_MOSSY: &[&str] = &[
    "underwater_ruin/big_mossy_1",
    "underwater_ruin/big_mossy_2",
    "underwater_ruin/big_mossy_3",
    "underwater_ruin/big_mossy_8",
];

pub struct OceanRuinGenerator {
    pub is_warm: bool,
}

impl StructureGenerator for OceanRuinGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let start_x = context.chunk_x << 4;
        let start_z = context.chunk_z << 4;
        let position = Vector3::new(start_x, 90, start_z);

        let rotation_idx = context.random.next_bounded_i32(4) as u8;
        let rotation = Rotation::from_index(rotation_idx);

        let mut collector = StructurePiecesCollector::default();
        add_pieces(
            &mut collector,
            position,
            rotation,
            &mut context.random,
            self.is_warm,
            0.3,
            0.7,
        );

        Some(StructurePosition {
            start_pos: BlockPos::new(start_x + 8, 64, start_z + 8),
            collector: Arc::new(collector.into()),
        })
    }
}

fn add_pieces(
    collector: &mut StructurePiecesCollector,
    position: Vector3<i32>,
    rotation: Rotation,
    random: &mut RandomGenerator,
    is_warm: bool,
    large_probability: f32,
    cluster_probability: f32,
) {
    let is_large = random.next_f32() <= large_probability;
    let base_integrity = if is_large { 0.9 } else { 0.8 };
    add_piece(
        collector,
        position,
        rotation,
        random,
        is_warm,
        is_large,
        base_integrity,
    );
    if is_large && random.next_f32() <= cluster_probability {
        add_cluster_ruins(collector, random, rotation, position, is_warm);
    }
}

fn add_cluster_ruins(
    collector: &mut StructurePiecesCollector,
    random: &mut RandomGenerator,
    rotation: Rotation,
    p: Vector3<i32>,
    is_warm: bool,
) {
    let parent_pos = Vector3::new(p.x, 90, p.z);
    let parent_corner = StructureTemplate::transform_block_pos(
        Vector3::new(15, 0, 15),
        Mirror::None,
        rotation,
        Vector3::new(0, 0, 0),
    ) + parent_pos;

    let parent_bb = BlockBox::new(
        parent_pos.x.min(parent_corner.x),
        parent_pos.y.min(parent_corner.y),
        parent_pos.z.min(parent_corner.z),
        parent_pos.x.max(parent_corner.x),
        parent_pos.y.max(parent_corner.y),
        parent_pos.z.max(parent_corner.z),
    );

    let parent_bottom_left = Vector3::new(
        parent_pos.x.min(parent_corner.x),
        parent_pos.y,
        parent_pos.z.min(parent_corner.z),
    );

    let mut all_positions = generate_cluster_positions(random, parent_bottom_left);
    let ruins_count = random.next_bounded_i32(5) + 4;

    for _ in 0..ruins_count {
        if all_positions.is_empty() {
            break;
        }
        let idx = random.next_bounded_i32(all_positions.len() as i32) as usize;
        let pos = all_positions.swap_remove(idx);
        let next_rotation = Rotation::from_index(random.next_bounded_i32(4) as u8);
        let next_corner = StructureTemplate::transform_block_pos(
            Vector3::new(5, 0, 6),
            Mirror::None,
            next_rotation,
            Vector3::new(0, 0, 0),
        ) + pos;

        let next_bb = BlockBox::new(
            pos.x.min(next_corner.x),
            pos.y.min(next_corner.y),
            pos.z.min(next_corner.z),
            pos.x.max(next_corner.x),
            pos.y.max(next_corner.y),
            pos.z.max(next_corner.z),
        );

        if !next_bb.intersects(&parent_bb) {
            add_piece(collector, pos, next_rotation, random, is_warm, false, 0.8);
        }
    }
}

fn generate_cluster_positions(
    random: &mut RandomGenerator,
    origin: Vector3<i32>,
) -> Vec<Vector3<i32>> {
    vec![
        origin
            + Vector3::new(
                -16 + random.next_bounded_i32(8) + 1,
                0,
                16 + random.next_bounded_i32(7) + 1,
            ),
        origin
            + Vector3::new(
                -16 + random.next_bounded_i32(8) + 1,
                0,
                random.next_bounded_i32(7) + 1,
            ),
        origin
            + Vector3::new(
                -16 + random.next_bounded_i32(8) + 1,
                0,
                -16 + random.next_bounded_i32(5) + 4,
            ),
        origin
            + Vector3::new(
                random.next_bounded_i32(7) + 1,
                0,
                16 + random.next_bounded_i32(7) + 1,
            ),
        origin
            + Vector3::new(
                random.next_bounded_i32(7) + 1,
                0,
                -16 + random.next_bounded_i32(3) + 4,
            ),
        origin
            + Vector3::new(
                16 + random.next_bounded_i32(7) + 1,
                0,
                16 + random.next_bounded_i32(6) + 3,
            ),
        origin
            + Vector3::new(
                16 + random.next_bounded_i32(7) + 1,
                0,
                random.next_bounded_i32(7) + 1,
            ),
        origin
            + Vector3::new(
                16 + random.next_bounded_i32(7) + 1,
                0,
                -16 + random.next_bounded_i32(5) + 4,
            ),
    ]
}

fn add_piece(
    collector: &mut StructurePiecesCollector,
    position: Vector3<i32>,
    rotation: Rotation,
    random: &mut RandomGenerator,
    is_warm: bool,
    is_large: bool,
    base_integrity: f32,
) {
    if is_warm {
        let pool = if is_large { BIG_WARM_RUINS } else { WARM_RUINS };
        let template_idx = random.next_bounded_i32(pool.len() as i32) as usize;
        let template_name = pool[template_idx];
        if let Some(template) = get_template(template_name) {
            collector.add_piece(Box::new(OceanRuinPiece::new(
                template,
                template_name.to_string(),
                position,
                rotation,
                base_integrity,
                is_warm,
                is_large,
            )));
        }
    } else {
        let (bricks, cracked, mossy) = if is_large {
            (BIG_RUINS_BRICK, BIG_RUINS_CRACKED, BIG_RUINS_MOSSY)
        } else {
            (RUINS_BRICK, RUINS_CRACKED, RUINS_MOSSY)
        };
        let idx = random.next_bounded_i32(bricks.len() as i32) as usize;
        if let Some(template) = get_template(bricks[idx]) {
            collector.add_piece(Box::new(OceanRuinPiece::new(
                template,
                bricks[idx].to_string(),
                position,
                rotation,
                base_integrity,
                is_warm,
                is_large,
            )));
        }
        if let Some(template) = get_template(cracked[idx]) {
            collector.add_piece(Box::new(OceanRuinPiece::new(
                template,
                cracked[idx].to_string(),
                position,
                rotation,
                0.7,
                is_warm,
                is_large,
            )));
        }
        if let Some(template) = get_template(mossy[idx]) {
            collector.add_piece(Box::new(OceanRuinPiece::new(
                template,
                mossy[idx].to_string(),
                position,
                rotation,
                0.5,
                is_warm,
                is_large,
            )));
        }
    }
}

pub struct OceanRuinPiece {
    pub piece: StructurePiece,
    pub template: Arc<StructureTemplate>,
    pub template_name: String,
    pub template_position: Vector3<i32>,
    pub rotation: Rotation,
    pub integrity: f32,
    pub is_warm: bool,
    pub is_large: bool,
}

impl OceanRuinPiece {
    #[must_use]
    pub fn new(
        template: Arc<StructureTemplate>,
        template_name: String,
        position: Vector3<i32>,
        rotation: Rotation,
        integrity: f32,
        is_warm: bool,
        is_large: bool,
    ) -> Self {
        let place_settings = StructurePlaceSettings::new()
            .set_rotation(rotation)
            .set_mirror(Mirror::None);
        let bounding_box = template.get_bounding_box(&place_settings, position);
        Self {
            piece: StructurePiece::new(StructurePieceType::OceanTemple, bounding_box, 0),
            template,
            template_name,
            template_position: position,
            rotation,
            integrity,
            is_warm,
            is_large,
        }
    }

    fn calculate_height(pos: Vector3<i32>, corner: Vector3<i32>, chunk: &ProtoChunk) -> i32 {
        let mut new_y = pos.y;
        let mut min_y = 512;
        let top_y = new_y - 1;
        let mut area = 0;
        let min_x = pos.x.min(corner.x);
        let max_x = pos.x.max(corner.x);
        let min_z = pos.z.min(corner.z);
        let max_z = pos.z.max(corner.z);

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let floor_y = chunk.get_top_y(&HeightMap::OceanFloorWg, x, z) - 1;
                min_y = min_y.min(floor_y);
                if floor_y < top_y - 2 {
                    area += 1;
                }
            }
        }

        let width = (pos.x - corner.x).abs();
        if top_y - min_y > 2 && area > width - 2 {
            new_y = min_y + 1;
        }

        new_y
    }
}

impl StructurePieceBase for OceanRuinPiece {
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
        self.template_position += Vector3::new(x, y, z);
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
        let height = chunk.get_top_y(
            &HeightMap::OceanFloorWg,
            self.template_position.x,
            self.template_position.z,
        );
        let mut pos = Vector3::new(self.template_position.x, height, self.template_position.z);
        let size = self.template.size;
        let corner = StructureTemplate::transform_block_pos(
            Vector3::new(size.x - 1, 0, size.z - 1),
            Mirror::None,
            self.rotation,
            Vector3::new(0, 0, 0),
        ) + pos;

        pos.y = Self::calculate_height(pos, corner, chunk);
        self.template_position = pos;
        let place_settings = StructurePlaceSettings::new()
            .set_rotation(self.rotation)
            .set_mirror(Mirror::None);
        self.piece.bounding_box = self
            .template
            .get_bounding_box(&place_settings, self.template_position);
        let box_limit = self.piece.bounding_box;

        let mut suspicious_count = 0;

        for block in &self.template.blocks {
            let palette_entry = &self.template.palette[block.state as usize];

            if palette_entry.name == "minecraft:structure_void"
                || palette_entry.name == "structure_void"
            {
                continue;
            }

            let rel_pos = StructureTemplate::transform_block_pos(
                block.pos,
                Mirror::None,
                self.rotation,
                Vector3::new(0, 0, 0),
            );
            let world_pos = self.template_position + rel_pos;

            if !box_limit.contains_pos(&world_pos) || !chunk_box.contains_pos(&world_pos) {
                continue;
            }

            if palette_entry.name == "minecraft:structure_block" {
                if let Some(nbt) = &block.nbt {
                    let mode = nbt.get_string("mode").unwrap_or("");
                    if mode == "DATA" {
                        let metadata = nbt.get_string("metadata").unwrap_or("");
                        if metadata == "chest" {
                            let is_water =
                                chunk.get_block_state(&world_pos).to_block_id() == Block::WATER.id;
                            let mut props = ChestLikeProperties::default(&Block::CHEST);
                            props.facing = HorizontalFacing::North;
                            props.waterlogged = is_water;
                            let chest_state = BlockState::from_id(props.to_state_id(&Block::CHEST));
                            chunk.set_block_state(
                                world_pos.x,
                                world_pos.y,
                                world_pos.z,
                                chest_state,
                            );

                            let mut chest_nbt = NbtCompound::new();
                            chest_nbt.put_string("id", "minecraft:chest".to_string());
                            chest_nbt.put_int("x", world_pos.x);
                            chest_nbt.put_int("y", world_pos.y);
                            chest_nbt.put_int("z", world_pos.z);
                            let loot_table = if self.is_large {
                                "minecraft:chests/underwater_ruin_big"
                            } else {
                                "minecraft:chests/underwater_ruin_small"
                            };
                            chest_nbt.put_string("LootTable", loot_table.to_string());
                            let mut rng = LegacyRand::from_seed(hash_block_pos(
                                world_pos.x,
                                world_pos.y,
                                world_pos.z,
                            )
                                as u64);
                            chest_nbt.put_long("LootTableSeed", rng.next_i64());
                            chunk.add_block_entity(chest_nbt);
                        } else if metadata == "drowned" {
                            let replacement = if world_pos.y > chunk.get_sea_level() {
                                Block::AIR.default_state
                            } else {
                                Block::WATER.default_state
                            };
                            chunk.set_block_state(
                                world_pos.x,
                                world_pos.y,
                                world_pos.z,
                                replacement,
                            );
                        }
                    }
                }
                continue;
            }

            if self.integrity < 0.999 && random.next_f32() > self.integrity {
                continue;
            }

            if palette_entry.name == "minecraft:air" || palette_entry.name == "air" {
                continue;
            }

            if self.is_warm
                && palette_entry.name == "minecraft:sand"
                && suspicious_count < 5
                && random.next_f32() < 0.45
            {
                suspicious_count += 1;
                chunk.set_block_state(
                    world_pos.x,
                    world_pos.y,
                    world_pos.z,
                    Block::SUSPICIOUS_SAND.default_state,
                );

                let mut nbt = NbtCompound::new();
                nbt.put_string("id", "minecraft:brushable_block".to_string());
                nbt.put_int("x", world_pos.x);
                nbt.put_int("y", world_pos.y);
                nbt.put_int("z", world_pos.z);
                nbt.put_string(
                    "LootTable",
                    "minecraft:archaeology/ocean_ruin_warm".to_string(),
                );
                let mut rng =
                    LegacyRand::from_seed(
                        hash_block_pos(world_pos.x, world_pos.y, world_pos.z) as u64
                    );
                nbt.put_long("LootTableSeed", rng.next_i64());
                chunk.add_block_entity(nbt);
                continue;
            } else if !self.is_warm
                && palette_entry.name == "minecraft:gravel"
                && suspicious_count < 5
                && random.next_f32() < 0.45
            {
                suspicious_count += 1;
                chunk.set_block_state(
                    world_pos.x,
                    world_pos.y,
                    world_pos.z,
                    Block::SUSPICIOUS_GRAVEL.default_state,
                );

                let mut nbt = NbtCompound::new();
                nbt.put_string("id", "minecraft:brushable_block".to_string());
                nbt.put_int("x", world_pos.x);
                nbt.put_int("y", world_pos.y);
                nbt.put_int("z", world_pos.z);
                nbt.put_string(
                    "LootTable",
                    "minecraft:archaeology/ocean_ruin_cold".to_string(),
                );
                let mut rng =
                    LegacyRand::from_seed(
                        hash_block_pos(world_pos.x, world_pos.y, world_pos.z) as u64
                    );
                nbt.put_long("LootTableSeed", rng.next_i64());
                chunk.add_block_entity(nbt);
                continue;
            }

            let Some(mut state) =
                BlockStateResolver::resolve(palette_entry, self.rotation, Mirror::None)
            else {
                continue;
            };

            if chunk.get_block_state(&world_pos).to_block_id() == Block::WATER.id {
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

            let final_block = Block::from_id(state.id.to_block_id());
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
    }
}
