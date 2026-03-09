use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, OakFenceLikeProperties},
    entity::EntityType,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    BlockDirection,
    math::{block_box::BlockBox, position::BlockPos},
    random::RandomGenerator,
};

use crate::{
    ProtoChunk,
    block::entities::mob_spawner::MobSpawnerBlockEntity,
    generation::structure::{
        piece::StructurePieceType,
        structures::{
            StructurePiece, StructurePieceBase, StructurePiecesCollector,
            nether_fortress::{NetherFortressPiece, PieceWeight},
        },
    },
};

/// Exterior platform with a Blaze mob-spawner (7 × 8 × 9).
#[derive(Clone)]
pub struct BridgePlatformPiece {
    pub piece: NetherFortressPiece,
    pub has_blaze_spawner: bool,
}

impl BridgePlatformPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        x: i32,
        y: i32,
        z: i32,
        chain_length: u32,
        orientation: BlockDirection,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bbox = BlockBox::rotated(x, y, z, -2, 0, 0, 7, 8, 9, &orientation);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridgePlatform,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(orientation));
        Some(Box::new(Self {
            piece,
            has_blaze_spawner: false,
        }))
    }
}

impl StructurePieceBase for BridgePlatformPiece {
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece.piece
    }

    // Terminal piece – no children.
    fn fill_openings_nether(
        &self,
        _start: &StructurePiece,
        _random: &mut RandomGenerator,
        _bridge_pieces: &mut Vec<PieceWeight>,
        _corridor_pieces: &mut Vec<PieceWeight>,
        _collector: &mut StructurePiecesCollector,
        _pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
    ) {
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let p = &self.piece.piece;
        let nb = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;

        p.fill_with_outline(chunk, &bb, false, 0, 2, 0, 6, 7, 7, air, air);
        p.fill_with_outline(chunk, &bb, false, 1, 0, 0, 5, 1, 7, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 2, 1, 5, 2, 7, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 3, 2, 5, 3, 7, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 4, 3, 5, 4, 7, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 2, 0, 1, 4, 2, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 5, 2, 0, 5, 4, 2, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 5, 2, 1, 5, 3, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 5, 5, 2, 5, 5, 3, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 0, 5, 3, 0, 5, 8, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 6, 5, 3, 6, 5, 8, nb, nb);
        p.fill_with_outline(chunk, &bb, false, 1, 5, 8, 5, 5, 8, nb, nb);

        // Fence helpers
        let fence = |west: bool, east: bool, north: bool, south: bool| {
            let mut props = OakFenceLikeProperties::default(&Block::NETHER_BRICK_FENCE);
            props.west = west;
            props.east = east;
            props.north = north;
            props.south = south;
            pumpkin_data::BlockState::from_id(props.to_state_id(&Block::NETHER_BRICK_FENCE))
        };

        let f_ew = fence(true, true, false, false);
        let f_ns = fence(false, false, true, true);

        p.add_block(chunk, fence(true, false, false, false), 1, 6, 3, &bb);
        p.add_block(chunk, fence(false, true, false, false), 5, 6, 3, &bb);
        p.add_block(chunk, fence(false, true, true, false), 0, 6, 3, &bb);
        p.add_block(chunk, fence(true, false, true, false), 6, 6, 3, &bb);

        p.fill_with_outline(chunk, &bb, false, 0, 6, 4, 0, 6, 7, f_ns, f_ns);
        p.fill_with_outline(chunk, &bb, false, 6, 6, 4, 6, 6, 7, f_ns, f_ns);

        p.add_block(chunk, fence(false, true, false, true), 0, 6, 8, &bb);
        p.add_block(chunk, fence(true, false, false, true), 6, 6, 8, &bb);
        p.fill_with_outline(chunk, &bb, false, 1, 6, 8, 5, 6, 8, f_ew, f_ew);

        p.add_block(chunk, fence(false, true, false, false), 1, 7, 8, &bb);
        p.fill_with_outline(chunk, &bb, false, 2, 7, 8, 4, 7, 8, f_ew, f_ew);
        p.add_block(chunk, fence(true, false, false, false), 5, 7, 8, &bb);

        p.add_block(chunk, fence(false, true, false, false), 2, 8, 8, &bb);
        p.add_block(chunk, f_ew, 3, 8, 8, &bb);
        p.add_block(chunk, fence(true, false, false, false), 4, 8, 8, &bb);

        // Blaze spawner (placed once)
        if !self.has_blaze_spawner {
            let spawner_pos = p.offset_pos(3, 5, 5);
            if bb.contains_pos(&spawner_pos) {
                self.has_blaze_spawner = true;
                chunk.set_block_state(
                    spawner_pos.x,
                    spawner_pos.y,
                    spawner_pos.z,
                    Block::SPAWNER.default_state,
                );
                let spawner_block_entity =
                    MobSpawnerBlockEntity::new(BlockPos(spawner_pos), Some(&EntityType::BLAZE));
                let mut entity_nbt = NbtCompound::new();
                spawner_block_entity.write_nbt(&mut entity_nbt);
                chunk.add_block_entity(entity_nbt);
            }
        }

        for i in 0..=6i32 {
            for j in 0..=6i32 {
                p.fill_downwards(chunk, nb, i, -1, j, &bb);
            }
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }
}
