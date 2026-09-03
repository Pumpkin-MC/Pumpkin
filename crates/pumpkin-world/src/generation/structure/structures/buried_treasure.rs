use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    BlockDirection, HeightMap,
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
    },
};

pub struct BuriedTreasureGenerator;

impl StructureGenerator for BuriedTreasureGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let x = (context.chunk_x << 4) + 9;
        let z = (context.chunk_z << 4) + 9;

        let bounding_box = BlockBox::new(x, 90, z, x, 90, z);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(BuriedTreasurePiece {
            piece: StructurePiece::new(StructurePieceType::BuriedTreasure, bounding_box, 0),
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, 90, z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct BuriedTreasurePiece {
    piece: StructurePiece,
}

impl StructurePieceBase for BuriedTreasurePiece {
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
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = self.bounding_box();
        let y = chunk.get_top_y(&HeightMap::OceanFloorWg, bb.min.x, bb.min.z);
        let min_y = chunk.bottom_y() as i32;
        let mut cur_y = y;

        while cur_y > min_y {
            let pos = Vector3::new(bb.min.x, cur_y, bb.min.z);
            let below_pos = Vector3::new(bb.min.x, cur_y - 1, bb.min.z);

            let current_state = chunk.get_block_state(&pos).to_state();
            let below_state = chunk.get_block_state(&below_pos).to_state();
            let below_block = Block::from_state_id(below_state.id);

            if *below_block == Block::SANDSTONE
                || *below_block == Block::STONE
                || *below_block == Block::ANDESITE
                || *below_block == Block::GRANITE
                || *below_block == Block::DIORITE
            {
                let soft_state = if !current_state.is_air() && !current_state.is_liquid() {
                    current_state
                } else {
                    Block::SAND.default_state
                };

                for dir in [
                    BlockDirection::Down,
                    BlockDirection::Up,
                    BlockDirection::North,
                    BlockDirection::South,
                    BlockDirection::West,
                    BlockDirection::East,
                ] {
                    let offset = dir.to_vector();
                    let rel_pos = pos + offset;
                    let rel_state = chunk.get_block_state(&rel_pos).to_state();

                    if rel_state.is_air() || rel_state.is_liquid() {
                        let below_rel_pos = rel_pos - Vector3::new(0, 1, 0);
                        let below_rel_state = chunk.get_block_state(&below_rel_pos).to_state();

                        if (below_rel_state.is_air() || below_rel_state.is_liquid())
                            && dir != BlockDirection::Up
                        {
                            chunk.set_block_state(rel_pos.x, rel_pos.y, rel_pos.z, below_state);
                        } else {
                            chunk.set_block_state(rel_pos.x, rel_pos.y, rel_pos.z, soft_state);
                        }
                    }
                }

                self.piece.bounding_box = BlockBox::new(pos.x, pos.y, pos.z, pos.x, pos.y, pos.z);

                if chunk_box.contains_pos(&pos) {
                    chunk.set_block_state(pos.x, pos.y, pos.z, Block::CHEST.default_state);

                    let mut nbt = NbtCompound::new();
                    nbt.put_string("id", "minecraft:chest".to_string());
                    nbt.put_int("x", pos.x);
                    nbt.put_int("y", pos.y);
                    nbt.put_int("z", pos.z);
                    nbt.put_string("LootTable", "minecraft:chests/buried_treasure".to_string());

                    let mut random =
                        LegacyRand::from_seed(hash_block_pos(pos.x, pos.y, pos.z) as u64);
                    nbt.put_long("LootTableSeed", random.next_i64());

                    chunk.add_block_entity(nbt);
                }
                return;
            }

            cur_y -= 1;
        }
    }
}
