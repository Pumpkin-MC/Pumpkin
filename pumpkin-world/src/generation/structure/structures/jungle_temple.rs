use std::sync::Arc;

use pumpkin_data::{
    Block, BlockState,
    block_properties::{
        AttachFace, BlockProperties, EastRedstone, Facing, HorizontalFacing, LeverLikeProperties,
        NorthRedstone, OakStairsLikeProperties, RedstoneWireLikeProperties, RepeaterLikeProperties,
        SouthRedstone, StickyPistonLikeProperties, TripwireHookLikeProperties,
        TripwireLikeProperties, VineLikeProperties, WestRedstone,
    },
};
use pumpkin_util::{
    BlockDirection, HeightMap,
    math::{block_box::BlockBox, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{start_block_x, start_block_z},
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition,
            },
        },
    },
};

/// Standard Jungle Temple dimensions
pub const WIDTH: i32 = 12;
pub const DEPTH: i32 = 15;

pub struct JungleTempleGenerator;

impl StructureGenerator for JungleTempleGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = start_block_x(context.chunk_x);
        let z = start_block_z(context.chunk_z);

        let facing = BlockDirection::get_random_horizontal_direction(&mut context.random);

        let mut piece = StructurePiece::new(
            StructurePieceType::JungleTemple,
            BlockBox::create_box(x, 64, z, facing.get_axis(), WIDTH, 10, DEPTH),
            0,
        );
        piece.set_facing(Some(facing));

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(JungleTemplePiece {
            piece,
            height_adjusted: false,
            placed_main_chest: true,
            placed_hidden_chest: true,
            placed_trap_1: true,
            placed_trap_2: true,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x + (WIDTH / 2), 64, z + (DEPTH / 2)),
            collector: Arc::new(collector.into()),
        })
    }
}
#[derive(Clone)]
pub struct JungleTemplePiece {
    // Composition: Rust uses a base struct to mimic the ScatteredFeaturePiece inheritance
    piece: StructurePiece,
    height_adjusted: bool,
    placed_main_chest: bool,
    placed_hidden_chest: bool,
    placed_trap_1: bool,
    placed_trap_2: bool,
}
impl StructurePieceBase for JungleTemplePiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
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
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        if !self.adjust_height(chunk, random) {
            return;
        }
        let bb = chunk_box;

        self.piece.fill(
            chunk,
            bb,
            0,
            -4,
            0,
            WIDTH - 1,
            0,
            DEPTH - 1,
            Self::SS.next(random),
        );
        self.piece
            .fill(chunk, bb, 2, 1, 2, 9, 2, 2, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 2, 1, 12, 9, 2, 12, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 2, 1, 3, 2, 2, 11, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 9, 1, 3, 9, 2, 11, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 1, 3, 1, 10, 6, 1, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 1, 3, 13, 10, 6, 13, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 1, 3, 2, 1, 6, 12, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 10, 3, 2, 10, 6, 12, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 2, 3, 2, 9, 3, 12, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 2, 6, 2, 9, 6, 12, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 3, 7, 3, 8, 7, 11, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 4, 8, 4, 7, 8, 10, Self::SS.next(random));
        let a = Block::AIR.default_state;
        self.piece.fill(chunk, bb, 3, 1, 3, 8, 2, 11, a);
        self.piece.fill(chunk, bb, 4, 3, 6, 7, 3, 9, a);
        self.piece.fill(chunk, bb, 2, 4, 2, 9, 5, 12, a);
        self.piece.fill(chunk, bb, 4, 6, 5, 7, 6, 9, a);
        self.piece.fill(chunk, bb, 5, 7, 6, 6, 7, 8, a);
        self.piece.fill(chunk, bb, 5, 1, 2, 6, 2, 2, a);
        self.piece.fill(chunk, bb, 5, 2, 12, 6, 2, 12, a);
        self.piece.fill(chunk, bb, 5, 5, 1, 6, 5, 1, a);
        self.piece.fill(chunk, bb, 5, 5, 13, 6, 5, 13, a);
        self.piece.add_block(chunk, a, 1, 5, 5, bb);
        self.piece.add_block(chunk, a, 10, 5, 5, bb);
        self.piece.add_block(chunk, a, 1, 5, 9, bb);
        self.piece.add_block(chunk, a, 10, 5, 9, bb);
        for z in (0..=14).step_by(14) {
            self.piece
                .fill(chunk, bb, 2, 4, z, 2, 5, z, Self::SS.next(random));
            self.piece
                .fill(chunk, bb, 4, 4, z, 4, 5, z, Self::SS.next(random));
            self.piece
                .fill(chunk, bb, 7, 4, z, 7, 5, z, Self::SS.next(random));
            self.piece
                .fill(chunk, bb, 9, 4, z, 9, 5, z, Self::SS.next(random));
        }
        self.piece
            .fill(chunk, bb, 5, 6, 0, 6, 6, 0, Self::SS.next(random));

        for x in (0..=11).step_by(11) {
            for z in (2..=12).step_by(2) {
                self.piece
                    .fill(chunk, bb, x, 4, z, x, 5, z, Self::SS.next(random));
            }

            self.piece
                .fill(chunk, bb, x, 6, 5, x, 6, 5, Self::SS.next(random));
            self.piece
                .fill(chunk, bb, x, 6, 9, x, 6, 9, Self::SS.next(random));
        }

        self.piece
            .fill(chunk, bb, 2, 7, 2, 2, 9, 2, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 9, 7, 2, 9, 9, 2, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 2, 7, 12, 2, 9, 12, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 9, 7, 12, 9, 9, 12, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 4, 9, 4, 4, 9, 4, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 7, 9, 4, 7, 9, 4, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 4, 9, 10, 4, 9, 10, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 7, 9, 10, 7, 9, 10, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 5, 9, 7, 6, 9, 7, Self::SS.next(random));
        let east_stairs = Self::cobblestone_stairs(HorizontalFacing::East);
        let west_stairs = Self::cobblestone_stairs(HorizontalFacing::West);
        let south_stairs = Self::cobblestone_stairs(HorizontalFacing::South);
        let north_stairs = Self::cobblestone_stairs(HorizontalFacing::North);
        self.piece.add_block(chunk, north_stairs, 5, 9, 6, bb);
        self.piece.add_block(chunk, north_stairs, 6, 9, 6, bb);
        self.piece.add_block(chunk, south_stairs, 5, 9, 8, bb);
        self.piece.add_block(chunk, south_stairs, 6, 9, 8, bb);
        self.piece.add_block(chunk, north_stairs, 4, 0, 0, bb);
        self.piece.add_block(chunk, north_stairs, 5, 0, 0, bb);
        self.piece.add_block(chunk, north_stairs, 6, 0, 0, bb);
        self.piece.add_block(chunk, north_stairs, 7, 0, 0, bb);
        self.piece.add_block(chunk, north_stairs, 4, 1, 8, bb);
        self.piece.add_block(chunk, north_stairs, 4, 2, 9, bb);
        self.piece.add_block(chunk, north_stairs, 4, 3, 10, bb);
        self.piece.add_block(chunk, north_stairs, 7, 1, 8, bb);
        self.piece.add_block(chunk, north_stairs, 7, 2, 9, bb);
        self.piece.add_block(chunk, north_stairs, 7, 3, 10, bb);
        self.piece
            .fill(chunk, bb, 4, 1, 9, 4, 1, 9, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 7, 1, 9, 7, 1, 9, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 4, 1, 10, 7, 2, 10, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 5, 4, 5, 6, 4, 5, Self::SS.next(random));
        self.piece.add_block(chunk, east_stairs, 4, 4, 5, bb);
        self.piece.add_block(chunk, west_stairs, 7, 4, 5, bb);
        for i in 0..4 {
            self.piece
                .add_block(chunk, south_stairs, 5, 0 - i, 6 + i, bb);
            self.piece
                .add_block(chunk, south_stairs, 6, 0 - i, 6 + i, bb);
            self.piece
                .fill(chunk, bb, 5, 0 - i, 7 + i, 6, 0 - i, 9 + i, a);
        }

        self.piece.fill(chunk, bb, 1, -3, 12, 10, -1, 13, a);
        self.piece.fill(chunk, bb, 1, -3, 1, 3, -1, 13, a);
        self.piece.fill(chunk, bb, 1, -3, 1, 9, -1, 5, a);
        for z in (1..=13).step_by(2) {
            self.piece
                .fill(chunk, bb, 1, -3, z, 1, -2, z, Self::SS.next(random));
        }
        for z in (2..=12).step_by(2) {
            self.piece
                .fill(chunk, bb, 1, -1, z, 3, -1, z, Self::SS.next(random));
        }

        self.piece
            .fill(chunk, bb, 2, -2, 1, 5, -2, 1, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 7, -2, 1, 9, -2, 1, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 6, -3, 1, 6, -3, 1, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 6, -1, 1, 6, -1, 1, Self::SS.next(random));
        self.piece.add_block(
            chunk,
            Self::attached_tripwire_hook_facing(HorizontalFacing::East),
            1,
            -3,
            8,
            bb,
        );
        self.piece.add_block(
            chunk,
            Self::attached_tripwire_hook_facing(HorizontalFacing::West),
            4,
            -3,
            8,
            bb,
        );
        self.piece.add_block(
            chunk,
            Self::redstone_wire_bidirectional(HorizontalFacing::East, HorizontalFacing::West),
            2,
            -3,
            8,
            bb,
        );
        self.piece.add_block(
            chunk,
            Self::attached_tripwire_bidirectional(HorizontalFacing::East, HorizontalFacing::West),
            3,
            -3,
            8,
            bb,
        );
        let redstone_wire_ns =
            Self::redstone_wire_bidirectional(HorizontalFacing::North, HorizontalFacing::South);
        self.piece.add_block(chunk, redstone_wire_ns, 5, -3, 7, bb);
        self.piece.add_block(chunk, redstone_wire_ns, 5, -3, 6, bb);
        self.piece.add_block(chunk, redstone_wire_ns, 5, -3, 5, bb);
        self.piece.add_block(chunk, redstone_wire_ns, 5, -3, 4, bb);
        self.piece.add_block(chunk, redstone_wire_ns, 5, -3, 3, bb);
        self.piece.add_block(chunk, redstone_wire_ns, 5, -3, 2, bb);
        self.piece.add_block(
            chunk,
            Self::redstone_wire_bidirectional(HorizontalFacing::North, HorizontalFacing::West),
            5,
            -3,
            1,
            bb,
        );
        self.piece.add_block(
            chunk,
            Self::redstone_wire_bidirectional(HorizontalFacing::East, HorizontalFacing::West),
            4,
            -3,
            1,
            bb,
        );
        self.piece
            .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 3, -3, 1, bb);
        /*if !self.placed_trap_1 {
            self.placed_trap_1 = this.createDispenser(
                chunk,
                bb,
                random,
                3,
                -2,
                1,
                Direction.NORTH,
                BuiltInLootTables.JUNGLE_TEMPLE_DISPENSER,
            );
        }*/

        self.piece
            .add_block(chunk, Self::vine_facing(Facing::South), 3, -2, 2, bb);
        self.piece.add_block(
            chunk,
            Self::attached_tripwire_hook_facing(HorizontalFacing::North),
            7,
            -3,
            1,
            bb,
        );
        self.piece.add_block(
            chunk,
            Self::attached_tripwire_hook_facing(HorizontalFacing::South),
            7,
            -3,
            5,
            bb,
        );
        self.piece.add_block(
            chunk,
            Self::attached_tripwire_bidirectional(HorizontalFacing::North, HorizontalFacing::South),
            7,
            -3,
            2,
            bb,
        );
        self.piece.add_block(
            chunk,
            Self::attached_tripwire_bidirectional(HorizontalFacing::North, HorizontalFacing::South),
            7,
            -3,
            3,
            bb,
        );
        self.piece.add_block(
            chunk,
            Self::attached_tripwire_bidirectional(HorizontalFacing::North, HorizontalFacing::South),
            7,
            -3,
            4,
            bb,
        );
        self.piece.add_block(
            chunk,
            Self::redstone_wire_bidirectional(HorizontalFacing::East, HorizontalFacing::West),
            8,
            -3,
            6,
            bb,
        );
        self.piece.add_block(
            chunk,
            Self::redstone_wire_bidirectional(HorizontalFacing::West, HorizontalFacing::South),
            9,
            -3,
            6,
            bb,
        );

        {
            let r_state = {
                let mut props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
                props.south = SouthRedstone::Up;
                props.north = NorthRedstone::Side;
                BlockState::from_id(props.to_state_id(&Block::REDSTONE_WIRE))
            };
            self.piece.add_block(chunk, r_state, 9, -3, 5, bb);
            self.piece
                .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 9, -3, 4, bb)
        };
        self.piece.add_block(chunk, redstone_wire_ns, 9, -2, 4, bb);
        /*if !self.placed_trap_2 {
            self.placed_trap_2 = this.createDispenser(
                chunk,
                bb,
                random,
                9,
                -2,
                3,
                Direction.WEST,
                BuiltInLootTables.JUNGLE_TEMPLE_DISPENSER,
            );
        }*/

        self.piece
            .add_block(chunk, Self::vine_facing(Facing::East), 8, -1, 3, bb);
        self.piece
            .add_block(chunk, Self::vine_facing(Facing::East), 8, -2, 3, bb);
        /*if !self.placed_main_chest {
            self.placed_main_chest =
                this.createChest(chunk, bb, random, 8, -3, 3, BuiltInLootTables.JUNGLE_TEMPLE);
        }*/

        self.piece
            .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 9, -3, 2, bb);
        self.piece
            .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 8, -3, 1, bb);
        self.piece
            .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 4, -3, 5, bb);
        self.piece
            .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 5, -2, 5, bb);
        self.piece
            .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 5, -1, 5, bb);
        self.piece
            .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 6, -3, 5, bb);
        self.piece
            .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 7, -2, 5, bb);
        self.piece
            .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 7, -1, 5, bb);
        self.piece
            .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 8, -3, 5, bb);
        self.piece
            .fill(chunk, bb, 9, -1, 1, 9, -1, 5, Self::SS.next(random));
        self.piece.fill(chunk, bb, 8, -3, 8, 10, -1, 10, a);
        self.piece.add_block(
            chunk,
            Block::CHISELED_STONE_BRICKS.default_state,
            8,
            -2,
            11,
            bb,
        );
        self.piece.add_block(
            chunk,
            Block::CHISELED_STONE_BRICKS.default_state,
            9,
            -2,
            11,
            bb,
        );
        self.piece.add_block(
            chunk,
            Block::CHISELED_STONE_BRICKS.default_state,
            10,
            -2,
            11,
            bb,
        );
        let lever = {
            let mut props = LeverLikeProperties::default(&Block::LEVER);
            props.facing = HorizontalFacing::North;
            props.face = AttachFace::Wall;
            BlockState::from_id(props.to_state_id(&Block::LEVER))
        };
        self.piece.add_block(chunk, lever, 8, -2, 12, bb);
        self.piece.add_block(chunk, lever, 9, -2, 12, bb);
        self.piece.add_block(chunk, lever, 10, -2, 12, bb);
        self.piece
            .fill(chunk, bb, 8, -3, 8, 8, -3, 10, Self::SS.next(random));
        self.piece
            .fill(chunk, bb, 10, -3, 8, 10, -3, 10, Self::SS.next(random));
        self.piece
            .add_block(chunk, Block::MOSSY_COBBLESTONE.default_state, 10, -2, 9, bb);
        self.piece.add_block(chunk, redstone_wire_ns, 8, -2, 9, bb);
        self.piece.add_block(chunk, redstone_wire_ns, 8, -2, 10, bb);

        let r_state = {
            let mut props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
            props.south = SouthRedstone::Up;
            props.north = NorthRedstone::Side;
            props.east = EastRedstone::Side;
            props.west = WestRedstone::Side;
            BlockState::from_id(props.to_state_id(&Block::REDSTONE_WIRE))
        };
        self.piece.add_block(chunk, r_state, 9, -3, 5, bb);
        self.piece.add_block(chunk, r_state, 10, -1, 9, bb);
        self.piece
            .add_block(chunk, Self::sticky_piston_facing(Facing::Up), 9, -2, 8, bb);
        self.piece.add_block(
            chunk,
            Self::sticky_piston_facing(Facing::West),
            10,
            -2,
            8,
            bb,
        );
        self.piece.add_block(
            chunk,
            Self::sticky_piston_facing(Facing::West),
            10,
            -1,
            8,
            bb,
        );
        let repeater_state = {
            let mut props = RepeaterLikeProperties::default(&Block::REPEATER);
            props.facing = HorizontalFacing::North;
            BlockState::from_id(props.to_state_id(&Block::REPEATER))
        };
        self.piece.add_block(chunk, repeater_state, 10, -2, 10, bb);
        /*if !self.placed_hidden_chest {
            self.placed_hidden_chest = this.createChest(
                chunk,
                bb,
                random,
                9,
                -3,
                10,
                BuiltInLootTables.JUNGLE_TEMPLE,
            );
        }*/
    }
}
const fn set_side_dir(props: &mut RedstoneWireLikeProperties, facing: HorizontalFacing) {
    match facing {
        HorizontalFacing::North => props.north = NorthRedstone::Side,
        HorizontalFacing::South => props.south = SouthRedstone::Side,
        HorizontalFacing::East => props.east = EastRedstone::Side,
        HorizontalFacing::West => props.west = WestRedstone::Side,
    }
}
impl JungleTemplePiece {
    fn adjust_height(&mut self, chunk: &ProtoChunk, random: &mut RandomGenerator) -> bool {
        if self.height_adjusted {
            return true;
        }

        let ground_offset = -(random.next_bounded_i32(3));
        let bb = self.piece.bounding_box;
        let mut lowest = i32::MAX;

        for z in bb.min.z..=bb.max.z {
            for x in bb.min.x..=bb.max.x {
                let y = chunk.get_top_y(&HeightMap::MotionBlockingNoLeaves, x, z);
                lowest = lowest.min(y);
            }
        }

        if lowest == i32::MAX {
            return false;
        }

        let shift_y = lowest - self.piece.bounding_box.min.y + ground_offset;
        self.piece.bounding_box.move_pos(0, shift_y, 0);
        self.height_adjusted = true;
        true
    }
    // This allows you to call Self::SS.next(random) SS is stoneselector
    const SS: MossStoneSelector = MossStoneSelector;
    fn cobblestone_stairs(facing: HorizontalFacing) -> &'static BlockState {
        let mut props = OakStairsLikeProperties::default(&Block::COBBLESTONE_STAIRS);
        props.facing = facing;
        BlockState::from_id(props.to_state_id(&Block::COBBLESTONE_STAIRS))
    }
    fn redstone_wire_bidirectional(
        facing_one: HorizontalFacing,
        facing_two: HorizontalFacing,
    ) -> &'static BlockState {
        let mut props = RedstoneWireLikeProperties::default(&Block::REDSTONE_WIRE);
        set_side_dir(&mut props, facing_one);
        set_side_dir(&mut props, facing_two);
        BlockState::from_id(props.to_state_id(&Block::REDSTONE_WIRE))
    }
    fn attached_tripwire_bidirectional(
        facing_one: HorizontalFacing,
        facing_two: HorizontalFacing,
    ) -> &'static BlockState {
        let mut props = TripwireLikeProperties::default(&Block::TRIPWIRE);
        match facing_one {
            HorizontalFacing::North => props.north = true,
            HorizontalFacing::South => props.south = true,
            HorizontalFacing::East => props.east = true,
            HorizontalFacing::West => props.west = true,
        }

        match facing_two {
            HorizontalFacing::North => props.north = true,
            HorizontalFacing::South => props.south = true,
            HorizontalFacing::East => props.east = true,
            HorizontalFacing::West => props.west = true,
        }
        props.attached = true;
        BlockState::from_id(props.to_state_id(&Block::TRIPWIRE))
    }
    fn sticky_piston_facing(facing: Facing) -> &'static BlockState {
        let mut props = StickyPistonLikeProperties::default(&Block::STICKY_PISTON);
        props.facing = facing;
        BlockState::from_id(props.to_state_id(&Block::STICKY_PISTON))
    }
    fn vine_facing(facing: Facing) -> &'static BlockState {
        let mut props = VineLikeProperties::default(&Block::VINE);
        match facing {
            Facing::North => props.north = true,
            Facing::East => props.east = true,
            Facing::West => props.west = true,
            Facing::South => props.south = true,
            Facing::Up => props.up = true,
            Facing::Down => {}
        }
        BlockState::from_id(props.to_state_id(&Block::VINE))
    }
    fn attached_tripwire_hook_facing(facing: HorizontalFacing) -> &'static BlockState {
        let mut props = TripwireHookLikeProperties::default(&Block::TRIPWIRE_HOOK);
        props.facing = facing;
        props.attached = true;
        BlockState::from_id(props.to_state_id(&Block::TRIPWIRE_HOOK))
    }
}
// Full implementation of the BlockSelector trait
struct MossStoneSelector;

impl MossStoneSelector {
    fn next(&self, random: &mut RandomGenerator) -> &BlockState {
        if random.next_f32() < 0.4 {
            Block::COBBLESTONE.default_state
        } else {
            Block::MOSSY_COBBLESTONE.default_state
        }
    }
}
