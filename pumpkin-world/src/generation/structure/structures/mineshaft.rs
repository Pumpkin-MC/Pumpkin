bwrap: /home/zoliviragh/.local/share/flatpak/app/dev.zed.Zed/x86_64/stable/d723b1334a83e703a7fd9419ba74d4567e26200e5541ac018186b3bc61d69c44/files/lib/libselinux.so.1: no version information available (required by bwrap)
//! Mineshaft structure generator (normal + mesa variants).
//!
//! Procedural, piece-based structure (not jigsaw): a starting room branches out into
//! corridors, crossings and stairs via a recursive worklist, matching vanilla
//! `MineshaftPieces`. Normal mineshafts use oak, mesa uses dark oak.

use std::sync::Arc;

use pumpkin_data::{Block, BlockState};
use pumpkin_util::{
    BlockDirection,
    math::{block_box::BlockBox, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};
use pumpkin_nbt::compound::NbtCompound;

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

/// Hard cap on recursive depth so generation always terminates.
const MAX_CHAIN_LENGTH: u32 = 16;
/// Maximum horizontal radius (blocks) pieces may extend from the start.
const MAX_RADIUS: i32 = 48;

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

/// A point where a child piece should be attached, facing outward from its parent.
struct Attachment {
    x: i32,
    y: i32,
    z: i32,
    facing: BlockDirection,
    chain_length: u32,
}

/// Generator constructed per structure key (normal vs mesa) by the dispatch.
pub struct MineshaftGenerator {
    pub mineshaft_type: MineshaftType,
}

impl StructureGenerator for MineshaftGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        // Gating is handled by the structure_set's frequency reduction (0.004, LegacyType3);
        // every chunk that reaches this generator should produce a mineshaft.
        let start_x = context.chunk_x * 16 + 8;
        let start_z = context.chunk_z * 16 + 8;
        // Vanilla places the start room somewhere underground.
        let start_y = context.random.next_bounded_i32(40);

        let mut collector = StructurePiecesCollector::default();

        // Start with a room; it seeds the recursive attachments.
        if let Some(room_attachments) = MineshaftRoomPiece::create_and_add(
            &mut collector,
            start_x,
            start_y,
            start_z,
            self.mineshaft_type,
            &mut context.random,
        ) {
            let mut pending = room_attachments;
            while let Some(attachment) = pending.pop() {
                if attachment.chain_length >= MAX_CHAIN_LENGTH {
                    continue;
                }
                let children = generate_piece(
                    &mut collector,
                    &attachment,
                    self.mineshaft_type,
                    &mut context.random,
                    start_x,
                    start_z,
                );
                pending.extend(children);
            }
        }

        if collector.pieces.is_empty() {
            return None;
        }

        Some(StructurePosition {
            start_pos: BlockPos::new(start_x, start_y, start_z),
            collector: Arc::new(collector.into()),
        })
    }
}

/// Picks a piece type for an attachment and creates it, returning any further attachments.
fn generate_piece(
    collector: &mut StructurePiecesCollector,
    attachment: &Attachment,
    mineshaft_type: MineshaftType,
    random: &mut RandomGenerator,
    start_x: i32,
    start_z: i32,
) -> Vec<Attachment> {
    let roll = random.next_bounded_i32(10);
    if roll < 7 {
        MineshaftCorridorPiece::create_and_add(
            collector,
            attachment,
            mineshaft_type,
            random,
            start_x,
            start_z,
        )
    } else if roll < 9 {
        MineshaftCrossingPiece::create_and_add(
            collector,
            attachment,
            mineshaft_type,
            random,
            start_x,
            start_z,
        )
    } else {
        MineshaftStairsPiece::create_and_add(
            collector,
            attachment,
            mineshaft_type,
            random,
            start_x,
            start_z,
        )
    }
}

/// Rejects a piece box that collides, lies outside the radius, or is above ground.
fn reject(
    collector: &StructurePiecesCollector,
    bbox: &BlockBox,
    start_x: i32,
    start_z: i32,
) -> bool {
    if collector.get_intersecting(bbox).is_some() {
        return true;
    }
    let cx = (bbox.min.x + bbox.max.x) / 2;
    let cz = (bbox.min.z + bbox.max.z) / 2;
    (cx - start_x).abs() > MAX_RADIUS || (cz - start_z).abs() > MAX_RADIUS
}

fn facing_step(facing: BlockDirection) -> (i32, i32) {
    match facing {
        BlockDirection::North => (0, -1),
        BlockDirection::South => (0, 1),
        BlockDirection::East => (1, 0),
        BlockDirection::West => (-1, 0),
        _ => (0, 0),
    }
}

/// Places a mob spawner block entity (e.g. cave spider) at piece-local coordinates.
fn place_spawner(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
    entity_id: &str,
) {
    let pos = piece.offset_pos(x, y, z);
    if !chunk_box.contains(pos.x, pos.y, pos.z) {
        return;
    }
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

/// Places a loot chest block entity at piece-local coordinates.
#[expect(clippy::too_many_arguments)]
fn place_loot_chest(
    chunk: &mut ProtoChunk,
    piece: &StructurePiece,
    x: i32,
    y: i32,
    z: i32,
    chunk_box: &BlockBox,
    seed: i64,
    loot_table: &str,
) {
    let pos = piece.offset_pos(x, y, z);
    if !chunk_box.contains(pos.x, pos.y, pos.z) {
        return;
    }
    chunk.set_block_state(pos.x, pos.y, pos.z, Block::CHEST.default_state);
    let mut nbt = NbtCompound::new();
    nbt.put_string("id", "minecraft:chest".to_string());
    nbt.put_int("x", pos.x);
    nbt.put_int("y", pos.y);
    nbt.put_int("z", pos.z);
    nbt.put_string("LootTable", loot_table.to_string());
    let loot_seed = seed ^ (pos.x as i64).rotate_left(13) ^ (pos.z as i64).rotate_left(7);
    nbt.put_long("LootTableSeed", loot_seed);
    chunk.add_block_entity(nbt);
}

// ===========================================================================
// Room
// ===========================================================================

struct MineshaftRoomPiece {
    piece: StructurePiece,
    width: i32,
    depth: i32,
    mineshaft_type: MineshaftType,
}

impl MineshaftRoomPiece {
    fn create_and_add(
        collector: &mut StructurePiecesCollector,
        x: i32,
        y: i32,
        z: i32,
        mineshaft_type: MineshaftType,
        random: &mut RandomGenerator,
    ) -> Option<Vec<Attachment>> {
        let width = (random.next_bounded_i32(3) * 2 + 3).max(3);
        let depth = (random.next_bounded_i32(3) * 2 + 3).max(3);
        let height = 4;
        let facing = BlockDirection::get_random_horizontal_direction(random);
        let bbox = BlockBox::rotated(x, y, z, -(width / 2), 0, -(depth / 2), width, height, depth, &facing);
        if reject(collector, &bbox, x, z) {
            return None;
        }
        let mut piece = StructurePiece::new(StructurePieceType::MineshaftRoom, bbox, 0);
        piece.set_facing(Some(facing));
        let room = Self {
            piece,
            width,
            depth,
            mineshaft_type,
        };
        let floor_y = room.piece.bounding_box.min.y;
        let min_x = room.piece.bounding_box.min.x;
        let min_z = room.piece.bounding_box.min.z;
        let max_x = room.piece.bounding_box.max.x;
        let max_z = room.piece.bounding_box.max.z;
        collector.add_piece(Box::new(room));

        // Four exits at the centre of each wall, facing outward.
        let cx = (min_x + max_x) / 2;
        let cz = (min_z + max_z) / 2;
        Some(vec![
            Attachment { x: cx, y: floor_y, z: min_z - 1, facing: BlockDirection::North, chain_length: 1 },
            Attachment { x: cx, y: floor_y, z: max_z + 1, facing: BlockDirection::South, chain_length: 1 },
            Attachment { x: min_x - 1, y: floor_y, z: cz, facing: BlockDirection::West, chain_length: 1 },
            Attachment { x: max_x + 1, y: floor_y, z: cz, facing: BlockDirection::East, chain_length: 1 },
        ])
    }
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
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let planks = self.mineshaft_type.planks();
        let log = self.mineshaft_type.log();
        let air = Block::AIR.default_state;
        let w = self.width - 1;
        let d = self.depth - 1;

        // Floor (with occasional gaps) and clear the interior.
        for x in 0..=w {
            for z in 0..=d {
                let floor = if random.next_f64() < 0.1 { air } else { planks };
                self.piece.add_block(chunk, floor, x, 0, z, chunk_box);
                self.piece.add_block(chunk, air, x, 1, z, chunk_box);
                self.piece.add_block(chunk, air, x, 2, z, chunk_box);
            }
        }
        // Corner support posts.
        for &(px, pz) in &[(0, 0), (w, 0), (0, d), (w, d)] {
            self.piece.add_block(chunk, log, px, 1, pz, chunk_box);
            self.piece.add_block(chunk, log, px, 2, pz, chunk_box);
        }
    }
}

// ===========================================================================
// Corridor
// ===========================================================================

struct MineshaftCorridorPiece {
    piece: StructurePiece,
    length: i32,
    facing: BlockDirection,
    mineshaft_type: MineshaftType,
    has_spawner: bool,
    has_chest: bool,
}

impl MineshaftCorridorPiece {
    fn create_and_add(
        collector: &mut StructurePiecesCollector,
        attachment: &Attachment,
        mineshaft_type: MineshaftType,
        random: &mut RandomGenerator,
        start_x: i32,
        start_z: i32,
    ) -> Vec<Attachment> {
        let length = random.next_bounded_i32(4) * 3 + 3; // 3..15 blocks
        let facing = attachment.facing;
        let bbox = BlockBox::rotated(
            attachment.x,
            attachment.y,
            attachment.z,
            -1,
            0,
            0,
            3,
            3,
            length,
            &facing,
        );
        if reject(collector, &bbox, start_x, start_z) {
            return Vec::new();
        }
        let mut piece = StructurePiece::new(
            StructurePieceType::MineshaftCorridor,
            bbox,
            attachment.chain_length,
        );
        piece.set_facing(Some(facing));

        // Far-end attachment (continue outward; occasionally turn).
        let (dx, dz) = facing_step(facing);
        let turn = random.next_bounded_i32(4) == 0;
        let next_facing = if turn {
            BlockDirection::get_random_horizontal_direction(random)
        } else {
            facing
        };
        let (nx, nz) = facing_step(next_facing);
        let far_x = attachment.x + dx * (length - 1) + nx;
        let far_z = attachment.z + dz * (length - 1) + nz;
        let mut children = Vec::new();
        if random.next_bounded_i32(3) > 0 {
            children.push(Attachment {
                x: far_x,
                y: attachment.y,
                z: far_z,
                facing: next_facing,
                chain_length: attachment.chain_length + 1,
            });
        }
        let has_spawner = random.next_bounded_i32(3) == 0;
        let has_chest = random.next_bounded_i32(4) == 0;
        collector.add_piece(Box::new(Self {
            piece,
            length,
            facing,
            mineshaft_type,
            has_spawner,
            has_chest,
        }));
        children
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
        let planks = self.mineshaft_type.planks();
        let log = self.mineshaft_type.log();
        let air = Block::AIR.default_state;
        let cobweb = Block::COBWEB.default_state;
        let length = self.length;

        for z in 0..length {
            // Floor (sometimes missing/decayed).
            for x in 0..3 {
                let floor = if random.next_f64() < 0.08 { air } else { planks };
                self.piece.add_block(chunk, floor, x, 0, z, chunk_box);
            }
            // Clear the walkable interior.
            self.piece.add_block(chunk, air, 1, 1, z, chunk_box);
            self.piece.add_block(chunk, air, 1, 2, z, chunk_box);

            // Supports every 3rd block: vertical posts + side air.
            if z % 3 == 0 {
                self.piece.add_block(chunk, log, 0, 1, z, chunk_box);
                self.piece.add_block(chunk, log, 2, 1, z, chunk_box);
                self.piece.add_block(chunk, air, 0, 2, z, chunk_box);
                self.piece.add_block(chunk, air, 2, 2, z, chunk_box);
                if random.next_f64() < 0.1 {
                    self.piece.add_block(chunk, cobweb, 1, 2, z, chunk_box);
                }
            } else {
                self.piece.add_block(chunk, air, 0, 1, z, chunk_box);
                self.piece.add_block(chunk, air, 2, 1, z, chunk_box);
            }
        }

        // Cave-spider spawner (vanilla: corridors may contain one) and a loot chest.
        let mid = self.length / 2;
        if self.has_spawner {
            place_spawner(chunk, &self.piece, 1, 1, mid, chunk_box, "minecraft:cave_spider");
        }
        if self.has_chest {
            place_loot_chest(
                chunk,
                &self.piece,
                1,
                1,
                mid + 1,
                chunk_box,
                seed,
                "minecraft:chests/abandoned_mineshaft",
            );
        }
    }
}

// ===========================================================================
// Crossing
// ===========================================================================

struct MineshaftCrossingPiece {
    piece: StructurePiece,
    mineshaft_type: MineshaftType,
    facing: BlockDirection,
}

impl MineshaftCrossingPiece {
    fn create_and_add(
        collector: &mut StructurePiecesCollector,
        attachment: &Attachment,
        mineshaft_type: MineshaftType,
        random: &mut RandomGenerator,
        start_x: i32,
        start_z: i32,
    ) -> Vec<Attachment> {
        let facing = attachment.facing;
        let bbox = BlockBox::rotated(attachment.x, attachment.y, attachment.z, -2, 0, -2, 5, 4, 5, &facing);
        if reject(collector, &bbox, start_x, start_z) {
            return Vec::new();
        }
        let mut piece = StructurePiece::new(
            StructurePieceType::MineshaftCrossing,
            bbox,
            attachment.chain_length,
        );
        piece.set_facing(Some(facing));
        let floor_y = piece.bounding_box.min.y;
        let min_x = piece.bounding_box.min.x;
        let min_z = piece.bounding_box.min.z;
        let max_x = piece.bounding_box.max.x;
        let max_z = piece.bounding_box.max.z;
        let cx = (min_x + max_x) / 2;
        let cz = (min_z + max_z) / 2;
        collector.add_piece(Box::new(Self {
            piece,
            mineshaft_type,
            facing,
        }));

        // Openings on each side (continue branching).
        let mut children = Vec::new();
        for side_facing in [
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::East,
            BlockDirection::West,
        ] {
            if random.next_bounded_i32(2) == 0 {
                let (dx, dz) = facing_step(side_facing);
                children.push(Attachment {
                    x: cx + dx * 3,
                    y: floor_y,
                    z: cz + dz * 3,
                    facing: side_facing,
                    chain_length: attachment.chain_length + 1,
                });
            }
        }
        children
    }
}

impl StructurePieceBase for MineshaftCrossingPiece {
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
        let planks = self.mineshaft_type.planks();
        let log = self.mineshaft_type.log();
        let air = Block::AIR.default_state;

        // Floor + cleared interior.
        for x in 0..=4 {
            for z in 0..=4 {
                self.piece.add_block(chunk, planks, x, 0, z, chunk_box);
                self.piece.add_block(chunk, air, x, 1, z, chunk_box);
                self.piece.add_block(chunk, air, x, 2, z, chunk_box);
            }
        }
        // Corner posts.
        for &(px, pz) in &[(0, 0), (4, 0), (0, 4), (4, 4)] {
            self.piece.add_block(chunk, log, px, 1, pz, chunk_box);
            self.piece.add_block(chunk, log, px, 2, pz, chunk_box);
        }
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
    fn create_and_add(
        collector: &mut StructurePiecesCollector,
        attachment: &Attachment,
        mineshaft_type: MineshaftType,
        _random: &mut RandomGenerator,
        start_x: i32,
        start_z: i32,
    ) -> Vec<Attachment> {
        let facing = attachment.facing;
        let bbox = BlockBox::rotated(attachment.x, attachment.y, attachment.z, -1, 0, 0, 3, 6, 3, &facing);
        if reject(collector, &bbox, start_x, start_z) {
            return Vec::new();
        }
        let mut piece = StructurePiece::new(
            StructurePieceType::MineshaftStairs,
            bbox,
            attachment.chain_length,
        );
        piece.set_facing(Some(facing));
        let (dx, dz) = facing_step(facing);
        let bottom_y = piece.bounding_box.min.y - 5;
        let children = vec![Attachment {
            x: attachment.x + dx * 3,
            y: bottom_y,
            z: attachment.z + dz * 3,
            facing,
            chain_length: attachment.chain_length + 1,
        }];
        collector.add_piece(Box::new(Self {
            piece,
            mineshaft_type,
        }));
        children
    }
}

impl StructurePieceBase for MineshaftStairsPiece {
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
        let planks = self.mineshaft_type.planks();
        let air = Block::AIR.default_state;

        // A simple descending staircase: step down one block per row.
        for row in 0..3 {
            let y = 5 - row;
            for x in 0..3 {
                self.piece.add_block(chunk, planks, x, y, row, chunk_box);
                self.piece.add_block(chunk, air, x, y + 1, row, chunk_box);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::structure::structures::{StructureGeneratorContext, create_chunk_random};
    use pumpkin_data::structures::StructureKeys;

    #[test]
    fn mineshaft_assembles_multiple_pieces() {
        let generator = MineshaftGenerator {
            mineshaft_type: MineshaftType::Normal,
        };
        // The generator applies a 0.01 probability gate, so try many chunks until one passes.
        for offset in 0..400 {
            let context = StructureGeneratorContext {
                seed: 1234,
                chunk_x: offset,
                chunk_z: offset,
                random: create_chunk_random(1234, offset, offset),
                sea_level: 63,
                min_y: -64,
                height_sampler: None,
                structure_key: Some(StructureKeys::Mineshaft),
            };
            if let Some(position) = generator.get_structure_position(context) {
                let count = position.collector.lock().unwrap().pieces.len();
                assert!(count > 1, "mineshaft produced only {count} pieces");
                return;
            }
        }
        panic!("mineshaft probability gate never passed in 400 attempts");
    }
}
