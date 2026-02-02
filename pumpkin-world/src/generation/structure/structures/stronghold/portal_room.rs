use pumpkin_data::{
    Block, BlockState,
    block_properties::{
        BlockProperties, EndPortalFrameLikeProperties, HorizontalFacing, OakFenceLikeProperties,
        OakStairsLikeProperties,
    },
};
use pumpkin_util::{
    BlockDirection,
    math::block_box::BlockBox,
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::structure::{
        piece::StructurePieceType,
        structures::{
            StructurePiece, StructurePieceBase, StructurePiecesCollector,
            stronghold::{
                EntranceType, PieceWeight, StoneBrickRandomizer, StrongholdPiece,
                StrongholdPieceType,
            },
        },
    },
};

#[derive(Clone)]
pub struct PortalRoomPiece {
    pub piece: StrongholdPiece,
    pub spawner_placed: bool,
}

impl PortalRoomPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        _random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bounding_box = BlockBox::rotated(x, y, z, -4, -1, 0, 11, 8, 16, &orientation);

        if !StrongholdPiece::is_in_bounds(&bounding_box)
            || collector.get_intersecting(&bounding_box).is_some()
        {
            return None;
        }

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdPortalRoom,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));

        Some(Box::new(Self {
            piece,
            spawner_placed: false,
        }))
    }
}

impl StructurePieceBase for PortalRoomPiece {
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece.piece
    }

    fn fill_openings(
        &self,
        _start: &StructurePiece,
        _random: &mut RandomGenerator,
        // TODO: this is only for Stronghold and should not be here
        _weights: &mut Vec<PieceWeight>,
        _last_piece_type: &mut Option<StrongholdPieceType>,
        has_portal_room: &mut bool,

        _collector: &mut StructurePiecesCollector,
        _pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
    ) {
        *has_portal_room = true
    }

    fn place(&mut self, chunk: &mut ProtoChunk, random: &mut RandomGenerator, _seed: i64) {
        let randomizer = StoneBrickRandomizer;
        let box_limit = self.piece.piece.bounding_box;
        let p = &self.piece;
        let inner = &p.piece;

        // 1. Main Shell
        inner.fill_outline_random(
            0,
            0,
            0,
            10,
            7,
            15,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );

        // 2. Entrance
        p.generate_entrance(chunk, &box_limit, EntranceType::Grates, 4, 1, 0);

        // 3. Upper Walkways
        inner.fill_outline_random(
            1,
            6,
            1,
            1,
            6,
            14,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            9,
            6,
            1,
            9,
            6,
            14,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            2,
            6,
            1,
            8,
            6,
            2,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            2,
            6,
            14,
            8,
            6,
            14,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );

        // 4. Lava Pools
        // Side Stone Brick borders
        inner.fill_outline_random(
            1,
            1,
            1,
            2,
            1,
            4,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            8,
            1,
            1,
            9,
            1,
            4,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );

        // Side Lava
        let lava = Block::LAVA.default_state;
        inner.fill_with_outline(chunk, &box_limit, false, 1, 1, 1, 1, 1, 3, lava, lava);
        inner.fill_with_outline(chunk, &box_limit, false, 9, 1, 1, 9, 1, 3, lava, lava);

        // Center Stone Brick platform
        inner.fill_outline_random(
            3,
            1,
            8,
            7,
            1,
            12,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        // Center Lava
        inner.fill_with_outline(chunk, &box_limit, false, 4, 1, 9, 6, 1, 11, lava, lava);

        // 5. Iron Bars
        let mut props = OakFenceLikeProperties::default(&Block::IRON_BARS);

        // North-South facing bars
        props.north = true;
        props.south = true;
        let bar_ns = BlockState::from_id(props.to_state_id(&Block::IRON_BARS));

        // West-East facing bars
        props.north = false;
        props.south = false;
        props.west = true;
        props.east = true;
        let bar_we = BlockState::from_id(props.to_state_id(&Block::IRON_BARS));

        // Bars along Z axis (North/South) at X=0 and X=10
        for j in (3..14).step_by(2) {
            inner.fill_with_outline(chunk, &box_limit, false, 0, 3, j, 0, 4, j, bar_ns, bar_ns);
            inner.fill_with_outline(chunk, &box_limit, false, 10, 3, j, 10, 4, j, bar_ns, bar_ns);
        }

        // Bars along X axis (West/East) at Z=15
        for j in (2..9).step_by(2) {
            inner.fill_with_outline(chunk, &box_limit, false, j, 3, 15, j, 4, 15, bar_we, bar_we);
        }

        // 6. Stairs Platform (Stone Brick backing)
        inner.fill_outline_random(
            4,
            1,
            5,
            6,
            1,
            7,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            4,
            2,
            6,
            6,
            2,
            7,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            4,
            3,
            7,
            6,
            3,
            7,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );

        // Stairs Blocks
        let mut props = OakStairsLikeProperties::default(&Block::STONE_BRICK_STAIRS);
        props.facing = HorizontalFacing::North;
        let stairs_n = BlockState::from_id(props.to_state_id(&Block::STONE_BRICK_STAIRS));

        for k in 4..=6 {
            inner.add_block(chunk, stairs_n, k, 1, 4, &box_limit);
            inner.add_block(chunk, stairs_n, k, 2, 5, &box_limit);
            inner.add_block(chunk, stairs_n, k, 3, 6, &box_limit);
        }

        // 7. End Portal Frames
        // Pre-calculate properties for base directions
        let mut props = EndPortalFrameLikeProperties::default(&Block::END_PORTAL_FRAME);
        props.facing = HorizontalFacing::North;
        let f_north = props;
        props.facing = HorizontalFacing::South;
        let f_south = props;
        props.facing = HorizontalFacing::East;
        let f_east = props;
        props.facing = HorizontalFacing::West;
        let f_west = props;

        // Randomize eyes
        let eyes: [bool; 12] = std::array::from_fn(|_| random.next_f32() > 0.9);
        let all_eyes = eyes.iter().all(|&e| e);

        // Helper to apply eye state
        let with_eye = |mut p: EndPortalFrameLikeProperties, has_eye: bool| {
            p.eye = has_eye;
            BlockState::from_id(p.to_state_id(&Block::END_PORTAL_FRAME))
        };

        // North Row
        inner.add_block(chunk, with_eye(f_north, eyes[0]), 4, 3, 8, &box_limit);
        inner.add_block(chunk, with_eye(f_north, eyes[1]), 5, 3, 8, &box_limit);
        inner.add_block(chunk, with_eye(f_north, eyes[2]), 6, 3, 8, &box_limit);

        // South Row
        inner.add_block(chunk, with_eye(f_south, eyes[3]), 4, 3, 12, &box_limit);
        inner.add_block(chunk, with_eye(f_south, eyes[4]), 5, 3, 12, &box_limit);
        inner.add_block(chunk, with_eye(f_south, eyes[5]), 6, 3, 12, &box_limit);

        // East Row
        inner.add_block(chunk, with_eye(f_east, eyes[6]), 3, 3, 9, &box_limit);
        inner.add_block(chunk, with_eye(f_east, eyes[7]), 3, 3, 10, &box_limit);
        inner.add_block(chunk, with_eye(f_east, eyes[8]), 3, 3, 11, &box_limit);

        // West Row
        inner.add_block(chunk, with_eye(f_west, eyes[9]), 7, 3, 9, &box_limit);
        inner.add_block(chunk, with_eye(f_west, eyes[10]), 7, 3, 10, &box_limit);
        inner.add_block(chunk, with_eye(f_west, eyes[11]), 7, 3, 11, &box_limit);

        // Fill Portal Liquid
        if all_eyes {
            let portal = Block::END_PORTAL.default_state;
            inner.fill_with_outline(chunk, &box_limit, false, 4, 3, 9, 6, 3, 11, portal, portal);
        }

        // 8. Spawner (Silverfish)
        if !self.spawner_placed {
            let pos = inner.offset_pos(5, 3, 6);
            if box_limit.contains_pos(&pos) {
                self.spawner_placed = true;
                let spawner = Block::SPAWNER.default_state;
                inner.add_block(chunk, spawner, 5, 3, 6, &box_limit);
                // Note: You must check your engine's API to set the BlockEntity (Silverfish).
                // Example: chunk.set_block_entity(pos, BlockEntity::Spawner(EntityType::Silverfish));
            }
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }
}
