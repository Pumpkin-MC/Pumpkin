bwrap: /home/zoliviragh/.local/share/flatpak/app/dev.zed.Zed/x86_64/stable/d723b1334a83e703a7fd9419ba74d4567e26200e5541ac018186b3bc61d69c44/files/lib/libselinux.so.1: no version information available (required by bwrap)
//! Mineshaft structure generator (normal + mesa variants).
//!
//! Faithful port of vanilla `MineshaftPieces` / `MineshaftStructure`.
//! Procedural piece-based structure: a room at Y=50 branches into corridors,
//! crossings and stairs. Normal uses oak, mesa uses dark oak.

use std::sync::Arc;

use pumpkin_data::{Block, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    BlockDirection,
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
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

const MAX_DEPTH: u32 = 8;
const BOUND: i32 = 80;
const MAGIC_START_Y: i32 = 50;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MineshaftType {
    Normal,
    Mesa,
}

impl MineshaftType {
    const fn planks(self) -> &'static BlockState {
        match self {
            Self::Normal => Block::OAK_PLANKS.default_state,
            Self::Mesa => Block::DARK_OAK_PLANKS.default_state,
        }
    }
    const fn log(self) -> &'static BlockState {
        match self {
            Self::Normal => Block::OAK_LOG.default_state,
            Self::Mesa => Block::DARK_OAK_LOG.default_state,
        }
    }
    const fn fence(self) -> &'static BlockState {
        match self {
            Self::Normal => Block::OAK_FENCE.default_state,
            Self::Mesa => Block::DARK_OAK_FENCE.default_state,
        }
    }
}

struct Attachment {
    x: i32,
    y: i32,
    z: i32,
    facing: BlockDirection,
    depth: u32,
}

pub struct MineshaftGenerator {
    pub mineshaft_type: MineshaftType,
}

impl StructureGenerator for MineshaftGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        // Vanilla consumes a double here (leftover from the old probability gate).
        let _ = context.random.next_f64();

        let west = context.chunk_x * 16 + 2;
        let north = context.chunk_z * 16 + 2;
        let mut collector = StructurePiecesCollector::default();

        // Create the starting room (vanilla: fixed Y=50, random width/depth/height).
        let room_box = BlockBox::new(
            west,
            MAGIC_START_Y,
            north,
            west + 7 + context.random.next_bounded_i32(6),
            54 + context.random.next_bounded_i32(6),
            north + 7 + context.random.next_bounded_i32(6),
        );
        let start_min_x = room_box.min.x;
        let start_min_z = room_box.min.z;

        let room = MineshaftRoomPiece {
            piece: StructurePiece::new(StructurePieceType::MineshaftRoom, room_box, 0),
            mineshaft_type: self.mineshaft_type,
        };
        let room_box = room.piece.bounding_box;
        collector.add_piece(Box::new(room));

        // Recursive assembly: room spawns children on all four walls.
        let mut pending = room_attachments(&room_box, &mut context.random);
        while let Some(att) = pending.pop() {
            if att.depth >= MAX_DEPTH {
                continue;
            }
            if (att.x - start_min_x).abs() > BOUND || (att.z - start_min_z).abs() > BOUND {
                continue;
            }
            let roll = context.random.next_bounded_i32(100);
            let new_piece: Option<Box<dyn StructurePieceBase>> = if roll >= 80 {
                MineshaftCrossingPiece::create(&att, self.mineshaft_type, &mut context.random, &collector)
            } else if roll >= 70 {
                MineshaftStairsPiece::create(&att, self.mineshaft_type, &mut context.random, &collector)
            } else {
                MineshaftCorridorPiece::create(&att, self.mineshaft_type, &mut context.random, &collector)
            };
            if let Some(piece) = new_piece {
                let children = if let Some(p) = piece.as_any().downcast_ref::<MineshaftCorridorPiece>() {
                    ChildAttachments::child_attachments(p, &mut context.random)
                } else if let Some(p) = piece.as_any().downcast_ref::<MineshaftCrossingPiece>() {
                    ChildAttachments::child_attachments(p, &mut context.random)
                } else if let Some(p) = piece.as_any().downcast_ref::<MineshaftStairsPiece>() {
                    ChildAttachments::child_attachments(p, &mut context.random)
                } else {
                    Vec::new()
                };
                collector.add_piece(piece);
                pending.extend(children);
            }
        }

        if collector.pieces.is_empty() {
            return None;
        }

        Some(StructurePosition {
            start_pos: BlockPos::new(west, MAGIC_START_Y, north),
            collector: Arc::new(collector.into()),
        })
    }
}

/// Room spawns corridors off all four walls at intervals (vanilla addChildren).
fn room_attachments(room: &BlockBox, random: &mut RandomGenerator) -> Vec<Attachment> {
    let mut attachments = Vec::new();
    let x_span = room.max.x - room.min.x;
    let z_span = room.max.z - room.min.z;
    let y_span = room.max.y - room.min.y;
    let height_space = (y_span - 4).max(1);

    // North and South walls.
    let mut pos = 0;
    while pos < x_span {
        pos += random.next_bounded_i32(x_span + 1);
        if pos + 3 > x_span {
            break;
        }
        attachments.push(Attachment {
            x: room.min.x + pos,
            y: room.min.y + random.next_bounded_i32(height_space) + 1,
            z: room.min.z - 1,
            facing: BlockDirection::North,
            depth: 1,
        });
        attachments.push(Attachment {
            x: room.min.x + pos,
            y: room.min.y + random.next_bounded_i32(height_space) + 1,
            z: room.max.z + 1,
            facing: BlockDirection::South,
            depth: 1,
        });
        pos += 4;
    }
    // West and East walls.
    pos = 0;
    while pos < z_span {
        pos += random.next_bounded_i32(z_span + 1);
        if pos + 3 > z_span {
            break;
        }
        attachments.push(Attachment {
            x: room.min.x - 1,
            y: room.min.y + random.next_bounded_i32(height_space) + 1,
            z: room.min.z + pos,
            facing: BlockDirection::West,
            depth: 1,
        });
        attachments.push(Attachment {
            x: room.max.x + 1,
            y: room.min.y + random.next_bounded_i32(height_space) + 1,
            z: room.min.z + pos,
            facing: BlockDirection::East,
            depth: 1,
        });
        pos += 4;
    }
    attachments
}

// ===========================================================================
// Trait for child attachment computation
// ===========================================================================

trait ChildAttachments {
    fn child_attachments(&self, random: &mut RandomGenerator) -> Vec<Attachment>;
}

// ===========================================================================
// Room
// ===========================================================================

struct MineshaftRoomPiece {
    piece: StructurePiece,
    mineshaft_type: MineshaftType,
}

impl StructurePieceBase for MineshaftRoomPiece {
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
        if is_in_liquid(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }
        let bb = self.piece.bounding_box;
        let air = Block::AIR.default_state;
        // Carve the interior (vanilla: clear lower box + dome ceiling).
        for x in bb.min.x..=bb.max.x {
            for y in (bb.min.y + 1)..=bb.max.y {
                for z in bb.min.z..=bb.max.z {
                    if chunk_box.contains(x, y, z) {
                        chunk.set_block_state(x, y, z, air);
                    }
                }
            }
        }
    }
}

impl ChildAttachments for MineshaftRoomPiece {
    fn child_attachments(&self, _random: &mut RandomGenerator) -> Vec<Attachment> {
        Vec::new() // Room's children are spawned by room_attachments before the loop.
    }
}

// ===========================================================================
// Corridor
// ===========================================================================

struct MineshaftCorridorPiece {
    piece: StructurePiece,
    mineshaft_type: MineshaftType,
    num_sections: i32,
    has_rails: bool,
    spider_corridor: bool,
    has_placed_spider: bool,
}

impl MineshaftCorridorPiece {
    fn create(
        att: &Attachment,
        mineshaft_type: MineshaftType,
        random: &mut RandomGenerator,
        collector: &StructurePiecesCollector,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let mut corridor_length = random.next_bounded_i32(3) + 2;
        while corridor_length > 0 {
            let block_length = corridor_length * 5;
            let bbox = BlockBox::rotated(
                att.x, att.y, att.z, 0, 0, 0, 3, 3, block_length, &att.facing,
            );
            if collector.get_intersecting(&bbox).is_none() {
                let mut piece = StructurePiece::new(
                    StructurePieceType::MineshaftCorridor,
                    bbox,
                    att.depth,
                );
                piece.set_facing(Some(att.facing));
                let has_rails = random.next_bounded_i32(3) == 0;
                let spider_corridor = !has_rails && random.next_bounded_i32(23) == 0;
                return Some(Box::new(Self {
                    piece,
                    mineshaft_type,
                    num_sections: corridor_length,
                    has_rails,
                    spider_corridor,
                    has_placed_spider: false,
                }));
            }
            corridor_length -= 1;
        }
        None
    }
}

impl StructurePieceBase for MineshaftCorridorPiece {
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
        seed: i64,
        chunk_box: &BlockBox,
    ) {
        if is_in_liquid(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }
        let planks = self.mineshaft_type.planks();
        let fence = self.mineshaft_type.fence();
        let air = Block::AIR.default_state;
        let cobweb = Block::COBWEB.default_state;
        let length = self.num_sections * 5 - 1;

        // Clear floor area (y=0,1) and ceiling (y=2 ~80%).
        for z in 0..=length {
            for x in 0..3 {
                self.piece.add_block(chunk, air, x, 0, z, chunk_box);
                self.piece.add_block(chunk, air, x, 1, z, chunk_box);
                if random.next_f32() < 0.8 {
                    self.piece.add_block(chunk, air, x, 2, z, chunk_box);
                }
            }
        }

        // Spider-corridor cobwebs on the floor (y=0,1 — vanilla generateMaybeBox).
        if self.spider_corridor {
            for z in 0..=length {
                for x in 0..3 {
                    if random.next_f32() < 0.6 {
                        self.piece.add_block(chunk, cobweb, x, 0, z, chunk_box);
                        self.piece.add_block(chunk, cobweb, x, 1, z, chunk_box);
                    }
                }
            }
        }

        // Per-section: supports + cobwebs + chest + spawner.
        for section in 0..self.num_sections {
            let z = 2 + section * 5;
            // Fence posts + plank beam (vanilla placeSupport).
            self.piece.add_block(chunk, fence, 0, 0, z, chunk_box);
            self.piece.add_block(chunk, fence, 0, 1, z, chunk_box);
            self.piece.add_block(chunk, fence, 2, 0, z, chunk_box);
            self.piece.add_block(chunk, fence, 2, 1, z, chunk_box);
            // Beam: 1/4 chance two separate planks, 3/4 chance full beam.
            if random.next_bounded_i32(4) == 0 {
                self.piece.add_block(chunk, planks, 0, 2, z, chunk_box);
                self.piece.add_block(chunk, planks, 2, 2, z, chunk_box);
            } else {
                for beam_x in 0..3 {
                    self.piece.add_block(chunk, planks, beam_x, 2, z, chunk_box);
                }
            }
            // Cobwebs near supports.
            for &dz in &[-1, 1] {
                let cz = z + dz;
                if (0..=length).contains(&cz) {
                    if random.next_f32() < 0.1 {
                        self.piece.add_block(chunk, cobweb, 0, 2, cz, chunk_box);
                    }
                    if random.next_f32() < 0.1 {
                        self.piece.add_block(chunk, cobweb, 2, 2, cz, chunk_box);
                    }
                }
            }
            // Chest (1% — vanilla uses chest-minecart; we use a chest block).
            if random.next_bounded_i32(100) == 0 {
                place_loot_chest(chunk, &self.piece, 2, 0, z - 1, chunk_box, seed, "minecraft:chests/abandoned_mineshaft");
            }
            if random.next_bounded_i32(100) == 0 {
                place_loot_chest(chunk, &self.piece, 0, 0, z + 1, chunk_box, seed, "minecraft:chests/abandoned_mineshaft");
            }
            // Cave-spider spawner.
            if self.spider_corridor && !self.has_placed_spider {
                let spawner_z = z - 1 + random.next_bounded_i32(3);
                if (0..=length).contains(&spawner_z) {
                    place_spawner(chunk, &self.piece, 1, 0, spawner_z, chunk_box, "minecraft:cave_spider");
                    self.has_placed_spider = true;
                }
            }
        }

        // Plank floor at y=-1 (below the box).
        for z in 0..=length {
            for x in 0..3 {
                self.piece.add_block(chunk, planks, x, -1, z, chunk_box);
            }
        }

        // Support pillars at z=2 and z=length-2 (wood pillar down to ground).
        let log_state = self.mineshaft_type.log();
        self.piece.fill_downwards(chunk, log_state, 0, -1, 2, chunk_box);
        self.piece.fill_downwards(chunk, log_state, 2, -1, 2, chunk_box);
        if self.num_sections > 1 {
            let last = length - 2;
            self.piece.fill_downwards(chunk, log_state, 0, -1, last, chunk_box);
            self.piece.fill_downwards(chunk, log_state, 2, -1, last, chunk_box);
        }

        // Rails.
        if self.has_rails {
            // Vanilla always uses NORTH_SOUTH shape; the rail block auto-connects on load.
            // Pumpkin's proto-chunk doesn't auto-update shapes, so set it explicitly per axis.
            let rail = if matches!(
                self.piece.facing,
                Some(BlockDirection::North | BlockDirection::South)
            ) {
                Block::RAIL.default_state
            } else {
                let props = Block::RAIL.from_properties(&[("shape", "east_west")]);
                BlockState::from_id(props.to_state_id(&Block::RAIL))
            };
            for z in 0..=length {
                self.piece.add_block(chunk, rail, 1, 0, z, chunk_box);
            }
        }
    }
}

impl ChildAttachments for MineshaftCorridorPiece {
    fn child_attachments(&self, random: &mut RandomGenerator) -> Vec<Attachment> {
        let bb = self.piece.bounding_box;
        let mut children = Vec::new();

        // End child (vanilla: one of straight/left/right at the far end).
        let end_selection = random.next_bounded_i32(4);
        let y_offset = random.next_bounded_i32(3) - 1;
        let (end_x, end_z, end_facing) = match self.piece.facing {
            Some(BlockDirection::North) => {
                if end_selection <= 1 { (bb.min.x, bb.min.z - 1, BlockDirection::North) }
                else if end_selection == 2 { (bb.min.x - 1, bb.min.z, BlockDirection::West) }
                else { (bb.max.x + 1, bb.min.z, BlockDirection::East) }
            }
            Some(BlockDirection::South) => {
                if end_selection <= 1 { (bb.min.x, bb.max.z + 1, BlockDirection::South) }
                else if end_selection == 2 { (bb.min.x - 1, bb.max.z, BlockDirection::West) }
                else { (bb.max.x + 1, bb.max.z, BlockDirection::East) }
            }
            Some(BlockDirection::West) => {
                if end_selection <= 1 { (bb.min.x - 1, bb.min.z, BlockDirection::West) }
                else if end_selection == 2 { (bb.min.x, bb.min.z - 1, BlockDirection::North) }
                else { (bb.min.x, bb.max.z + 1, BlockDirection::South) }
            }
            _ => {
                if end_selection <= 1 { (bb.max.x + 1, bb.min.z, BlockDirection::East) }
                else if end_selection == 2 { (bb.max.x, bb.min.z - 1, BlockDirection::North) }
                else { (bb.max.x, bb.max.z + 1, BlockDirection::South) }
            }
        };
        children.push(Attachment {
            x: end_x,
            y: bb.min.y + y_offset,
            z: end_z,
            facing: end_facing,
            depth: self.piece.chain_length + 1,
        });

        // Side children every 5 blocks (vanilla: 40% each side).
        if self.piece.chain_length < MAX_DEPTH {
            let is_ns = matches!(self.piece.facing, Some(BlockDirection::North | BlockDirection::South));
            if is_ns {
                let mut z = bb.min.z + 3;
                while z + 3 <= bb.max.z {
                    let sel = random.next_bounded_i32(5);
                    if sel == 0 {
                        children.push(Attachment { x: bb.min.x - 1, y: bb.min.y, z, facing: BlockDirection::West, depth: self.piece.chain_length + 1 });
                    } else if sel == 1 {
                        children.push(Attachment { x: bb.max.x + 1, y: bb.min.y, z, facing: BlockDirection::East, depth: self.piece.chain_length + 1 });
                    }
                    z += 5;
                }
            } else {
                let mut x = bb.min.x + 3;
                while x + 3 <= bb.max.x {
                    let sel = random.next_bounded_i32(5);
                    if sel == 0 {
                        children.push(Attachment { x, y: bb.min.y, z: bb.min.z - 1, facing: BlockDirection::North, depth: self.piece.chain_length + 1 });
                    } else if sel == 1 {
                        children.push(Attachment { x, y: bb.min.y, z: bb.max.z + 1, facing: BlockDirection::South, depth: self.piece.chain_length + 1 });
                    }
                    x += 5;
                }
            }
        }
        children
    }
}

// ===========================================================================
// Crossing
// ===========================================================================

struct MineshaftCrossingPiece {
    piece: StructurePiece,
    mineshaft_type: MineshaftType,
}

impl MineshaftCrossingPiece {
    fn create(
        att: &Attachment,
        mineshaft_type: MineshaftType,
        random: &mut RandomGenerator,
        collector: &StructurePiecesCollector,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let y1 = if random.next_bounded_i32(4) == 0 { 6 } else { 2 };
        let bbox = BlockBox::rotated(att.x, att.y, att.z, -1, 0, 0, 5, y1 + 1, 5, &att.facing);
        if collector.get_intersecting(&bbox).is_some() {
            return None;
        }
        let mut piece = StructurePiece::new(StructurePieceType::MineshaftCrossing, bbox, att.depth);
        piece.set_facing(Some(att.facing));
        Some(Box::new(Self { piece, mineshaft_type }))
    }
}

impl StructurePieceBase for MineshaftCrossingPiece {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_structure_piece(&self) -> &StructurePiece { &self.piece }
    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece { &mut self.piece }
    fn place(&mut self, chunk: &mut ProtoChunk, _br: &dyn WorldPortalExt, _r: &mut RandomGenerator, _s: i64, chunk_box: &BlockBox) {
        if is_in_liquid(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }
        let planks = self.mineshaft_type.planks();
        let air = Block::AIR.default_state;
        // Clear interior.
        for x in 0..=4 {
            for y in 0..=4 {
                for z in 0..=4 {
                    self.piece.add_block(chunk, air, x, y, z, chunk_box);
                }
            }
        }
        // Corner support pillars.
        let log = self.mineshaft_type.log();
        for &(px, pz) in &[(0, 0), (4, 0), (0, 4), (4, 4)] {
            for py in 0..=4 {
                self.piece.add_block(chunk, log, px, py, pz, chunk_box);
            }
        }
        // Plank floor at y=-1.
        for x in 0..=4 {
            for z in 0..=4 {
                self.piece.add_block(chunk, planks, x, -1, z, chunk_box);
            }
        }
    }
}

impl ChildAttachments for MineshaftCrossingPiece {
    fn child_attachments(&self, random: &mut RandomGenerator) -> Vec<Attachment> {
        let bb = self.piece.bounding_box;
        let depth = self.piece.chain_length;
        let mut children = Vec::new();
        // Spawn on 3 of 4 sides (skip the entry side = facing).
        for (dir, x, z) in [
            (BlockDirection::North, bb.min.x + 1, bb.min.z - 1),
            (BlockDirection::South, bb.min.x + 1, bb.max.z + 1),
            (BlockDirection::West, bb.min.x - 1, bb.min.z + 1),
            (BlockDirection::East, bb.max.x + 1, bb.min.z + 1),
        ] {
            if dir != self.piece.facing.unwrap_or(BlockDirection::North) && random.next_bounded_i32(2) == 0 {
                children.push(Attachment { x, y: bb.min.y, z, facing: dir, depth: depth + 1 });
            }
        }
        children
    }
}

// ===========================================================================
// Stairs
// ===========================================================================

struct MineshaftStairsPiece {
    piece: StructurePiece,
    mineshaft_type: MineshaftType,
}

impl MineshaftStairsPiece {
    fn create(
        att: &Attachment,
        mineshaft_type: MineshaftType,
        _random: &mut RandomGenerator,
        collector: &StructurePiecesCollector,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bbox = BlockBox::rotated(att.x, att.y, att.z, 0, -5, 0, 3, 8, 9, &att.facing);
        if collector.get_intersecting(&bbox).is_some() {
            return None;
        }
        let mut piece = StructurePiece::new(StructurePieceType::MineshaftStairs, bbox, att.depth);
        piece.set_facing(Some(att.facing));
        Some(Box::new(Self { piece, mineshaft_type }))
    }
}

impl StructurePieceBase for MineshaftStairsPiece {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_structure_piece(&self) -> &StructurePiece { &self.piece }
    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece { &mut self.piece }
    fn place(&mut self, chunk: &mut ProtoChunk, _br: &dyn WorldPortalExt, _r: &mut RandomGenerator, _s: i64, chunk_box: &BlockBox) {
        if is_in_liquid(chunk, &self.piece.bounding_box, chunk_box) {
            return;
        }
        let air = Block::AIR.default_state;
        // Carve upper landing.
        self.piece.fill(chunk, chunk_box, 0, 5, 0, 2, 7, 1, air);
        // Carve lower landing.
        self.piece.fill(chunk, chunk_box, 0, 0, 7, 2, 2, 8, air);
        // Descending steps.
        for i in 0..5 {
            let y = 5 - i - if i < 4 { 1 } else { 0 };
            self.piece.fill(chunk, chunk_box, 0, y, 2 + i, 2, 7 - i, 2 + i, air);
        }
    }
}

impl ChildAttachments for MineshaftStairsPiece {
    fn child_attachments(&self, _random: &mut RandomGenerator) -> Vec<Attachment> {
        let bb = self.piece.bounding_box;
        let facing = self.piece.facing.unwrap_or(BlockDirection::North);
        let (x, z) = match facing {
            BlockDirection::North => (bb.min.x, bb.min.z - 1),
            BlockDirection::South => (bb.min.x, bb.max.z + 1),
            BlockDirection::West => (bb.min.x - 1, bb.min.z),
            _ => (bb.max.x + 1, bb.min.z),
        };
        vec![Attachment { x, y: bb.min.y, z, facing, depth: self.piece.chain_length + 1 }]
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Checks whether the piece's floor area contains liquid (vanilla `isInInvalidLocation`).
fn is_in_liquid(chunk: &ProtoChunk, bb: &BlockBox, chunk_box: &BlockBox) -> bool {
    let cx = (bb.min.x + bb.max.x) / 2;
    let cz = (bb.min.z + bb.max.z) / 2;
    for &(x, z) in &[(bb.min.x, bb.min.z), (bb.max.x, bb.max.z), (cx, cz)] {
        if chunk_box.contains(x, bb.min.y, z)
            && chunk
                .get_block_state(&Vector3::new(x, bb.min.y, z))
                .to_state()
                .is_liquid()
        {
            return true;
        }
    }
    false
}

fn place_spawner(chunk: &mut ProtoChunk, piece: &StructurePiece, x: i32, y: i32, z: i32, chunk_box: &BlockBox, entity_id: &str) {
    let pos = piece.offset_pos(x, y, z);
    if !chunk_box.contains(pos.x, pos.y, pos.z) { return; }
    chunk.set_block_state(pos.x, pos.y, pos.z, Block::SPAWNER.default_state);
    let mut nbt = NbtCompound::new();
    nbt.put_string("id", "minecraft:mob_spawner".to_string());
    nbt.put_int("x", pos.x);
    nbt.put_int("y", pos.y);
    nbt.put_int("z", pos.z);
    let mut spawn_data = NbtCompound::new();
    let mut entity = NbtCompound::new();
    entity.put_string("id", entity_id.to_string());
    spawn_data.put_compound("entity", entity);
    nbt.put_compound("SpawnData", spawn_data);
    chunk.add_block_entity(nbt);
}

#[expect(clippy::too_many_arguments)]
fn place_loot_chest(chunk: &mut ProtoChunk, piece: &StructurePiece, x: i32, y: i32, z: i32, chunk_box: &BlockBox, seed: i64, loot_table: &str) {
    let pos = piece.offset_pos(x, y, z);
    if !chunk_box.contains(pos.x, pos.y, pos.z) { return; }
    chunk.set_block_state(pos.x, pos.y, pos.z, Block::CHEST.default_state);
    let mut nbt = NbtCompound::new();
    nbt.put_string("id", "minecraft:chest".to_string());
    nbt.put_int("x", pos.x);
    nbt.put_int("y", pos.y);
    nbt.put_int("z", pos.z);
    nbt.put_string("LootTable", loot_table.to_string());
    nbt.put_long("LootTableSeed", seed ^ (pos.x as i64).rotate_left(13) ^ (pos.z as i64).rotate_left(7));
    chunk.add_block_entity(nbt);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::structure::structures::{StructureGeneratorContext, create_chunk_random};
    use pumpkin_data::structures::StructureKeys;

    #[test]
    fn mineshaft_assembles_multiple_pieces() {
        let generator = MineshaftGenerator { mineshaft_type: MineshaftType::Normal };
        let context = StructureGeneratorContext {
            seed: 42,
            chunk_x: 0,
            chunk_z: 0,
            random: create_chunk_random(42, 0, 0),
            sea_level: 63,
            min_y: -64,
            height_sampler: None,
            structure_key: Some(StructureKeys::Mineshaft),
        };
        let position = generator.get_structure_position(context).expect("mineshaft should generate");
        let count = position.collector.lock().unwrap().pieces.len();
        assert!(count > 2, "mineshaft generated only {count} pieces");
    }
}
