//! End City generation, ported from vanilla 26.2.
//!
//! Sources:
//! - `EndCityStructure.java` — start rotation, 5x5 lowest-Y probe, height gate.
//! - `EndCityPieces.java` — the recursive template generator (house tower,
//!   tower, tower bridge, fat tower section generators).
//!
//! Coordinate convention: vanilla anchors every piece at a `templatePosition`
//! and rotates template blocks around it with pivot `BlockPos.ZERO`
//! (`StructureTemplate.transform`, StructureTemplate.java:441-467), so rotated
//! pieces extend into negative X/Z relative to their anchor. Pumpkin's
//! `place_template` instead rotates blocks within `[0, rotated_size)` of the
//! origin (block_rotation.rs:52-59). Both produce identical world blocks when
//! the Pumpkin origin is the min corner of the vanilla bounding box
//! (`StructureTemplate.getBoundingBox`, StructureTemplate.java:515-529), so
//! generation tracks the vanilla `templatePosition` and placement uses
//! `bounding_box.min`.

use std::sync::Arc;

use pumpkin_data::block_rotation::Rotation;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_offset_x, get_offset_z},
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition, WorldPortalExt,
            },
            template::{StructureTemplate, get_template, place_template},
        },
    },
};

/// `EndCityPieces.MAX_GEN_DEPTH` (EndCityPieces.java:38).
const MAX_GEN_DEPTH: i32 = 8;

/// `if (startPos.getY() < 60) return Optional.empty()`
/// (EndCityStructure.java:33).
const MIN_START_Y: i32 = 60;

/// `BuiltInLootTables.END_CITY_TREASURE` (BuiltInLootTables.java:19).
const END_CITY_TREASURE_LOOT: &str = "minecraft:chests/end_city_treasure";

/// `EndCityPieces.TOWER_BRIDGES` (EndCityPieces.java:68). Order matters: one
/// `nextBoolean()` is drawn per entry in declaration order (l.89-90).
const TOWER_BRIDGES: [(Rotation, (i32, i32, i32)); 4] = [
    (Rotation::None, (1, -1, 0)),
    (Rotation::Clockwise90, (6, -1, 1)),
    (Rotation::CounterClockwise90, (0, -1, 5)),
    (Rotation::Rotate180, (5, -1, 6)),
];

/// `EndCityPieces.FAT_TOWER_BRIDGES` (EndCityPieces.java:140), same draw-order
/// significance as [`TOWER_BRIDGES`].
const FAT_TOWER_BRIDGES: [(Rotation, (i32, i32, i32)); 4] = [
    (Rotation::None, (4, -1, 0)),
    (Rotation::Clockwise90, (12, -1, 4)),
    (Rotation::CounterClockwise90, (0, -1, 8)),
    (Rotation::Rotate180, (8, -1, 12)),
];

/// Every template referenced by the vanilla generators. The bundled
/// `tower_floor.nbt` is intentionally absent: vanilla ships it but no code in
/// `EndCityPieces.java` references it.
const TEMPLATE_NAMES: [&str; 19] = [
    "base_floor",
    "base_roof",
    "second_floor_1",
    "second_floor_2",
    "second_roof",
    "third_floor_1",
    "third_floor_2",
    "third_roof",
    "tower_base",
    "tower_piece",
    "tower_top",
    "bridge_end",
    "bridge_piece",
    "bridge_steep_stairs",
    "bridge_gentle_stairs",
    "fat_tower_base",
    "fat_tower_middle",
    "fat_tower_top",
    "ship",
];

/// The four anonymous `SectionGenerator` singletons (EndCityPieces.java:39,
/// 69, 103, 141), dispatched by [`generate_section`].
#[derive(Clone, Copy)]
enum SectionGenerator {
    HouseTower,
    Tower,
    TowerBridge,
    FatTower,
}

/// Pre-loaded template set so the recursive generator never fails a lookup
/// (vanilla's `StructureTemplateManager` panics on missing templates instead).
struct CityTemplates {
    entries: Vec<(&'static str, Arc<StructureTemplate>)>,
}

impl CityTemplates {
    fn load() -> Option<Self> {
        let mut entries = Vec::with_capacity(TEMPLATE_NAMES.len());
        for name in TEMPLATE_NAMES {
            entries.push((name, get_template(&format!("end_city/{name}"))?));
        }
        Some(Self { entries })
    }

    fn get(&self, name: &str) -> Arc<StructureTemplate> {
        self.entries
            .iter()
            .find(|(entry_name, _)| *entry_name == name)
            .map(|(_, template)| Arc::clone(template))
            .expect("every generator template is listed in TEMPLATE_NAMES")
    }
}

/// The parent fields `addPiece` and `recursiveChildren` read, copied by value.
/// This matches vanilla reference semantics: the only post-add mutations of a
/// piece are the batch retag (EndCityPieces.java:198) — which runs after every
/// use of that piece as a parent — and the `setGenDepth(-1)` tags
/// (l.116/l.136), which are applied to pieces that are never used as a
/// recursion parent afterwards.
#[derive(Clone, Copy)]
struct PieceRef {
    /// Vanilla `templatePosition` (rotation anchor, not the box min corner).
    position: Vector3<i32>,
    rotation: Rotation,
    gen_depth: i32,
}

/// A generated piece awaiting acceptance — vanilla
/// `EndCityPieces.EndCityPiece` (EndCityPieces.java:212-264).
struct CityPiece {
    template: Arc<StructureTemplate>,
    /// Vanilla `templatePosition`.
    position: Vector3<i32>,
    rotation: Rotation,
    /// The `OW` flag (EndCityPieces.java:222-225): `true` selects
    /// `BlockIgnoreProcessor.STRUCTURE_BLOCK` (template air overwrites
    /// terrain), `false` selects `STRUCTURE_AND_AIR` (air is skipped).
    overwrite: bool,
    bounding_box: BlockBox,
    /// `StructurePiece.genDepth`, reused as a batch tag by `recursiveChildren`
    /// for the pending-piece overlap rejection (EndCityPieces.java:196-202).
    gen_depth: i32,
}

impl CityPiece {
    fn new(
        template: Arc<StructureTemplate>,
        position: Vector3<i32>,
        rotation: Rotation,
        overwrite: bool,
    ) -> Self {
        let bounding_box = template_bounding_box(position, rotation, template.size);
        Self {
            template,
            position,
            rotation,
            overwrite,
            bounding_box,
            // Pieces are constructed with genDepth 0 (EndCityPieces.java:215).
            gen_depth: 0,
        }
    }

    const fn piece_ref(&self) -> PieceRef {
        PieceRef {
            position: self.position,
            rotation: self.rotation,
            gen_depth: self.gen_depth,
        }
    }

    fn into_structure_piece(self) -> EndCityPiece {
        EndCityPiece {
            piece: StructurePiece::new(StructurePieceType::EndCity, self.bounding_box, 0),
            template: self.template,
            rotation: self.rotation,
            overwrite: self.overwrite,
        }
    }
}

/// `StructureTemplate.getBoundingBox` (StructureTemplate.java:515-529): the
/// box spans the pivot-zero transforms of `(0, 0, 0)` and `size - 1` moved by
/// the template position, so 90-degree rotations extend into negative X/Z
/// relative to the anchor. `Rotation::rotate_offset` (block_rotation.rs:66-73)
/// is exactly the pivot-zero `StructureTemplate.transform`
/// (StructureTemplate.java:441-467).
fn template_bounding_box(
    position: Vector3<i32>,
    rotation: Rotation,
    size: Vector3<i32>,
) -> BlockBox {
    let (corner_x, corner_z) = rotation.rotate_offset(size.x - 1, size.z - 1);
    BlockBox::new(
        position.x + corner_x.min(0),
        position.y,
        position.z + corner_z.min(0),
        position.x + corner_x.max(0),
        position.y + size.y - 1,
        position.z + corner_z.max(0),
    )
}

/// Mutable generation state threaded through the section generators.
struct GenState<'a> {
    templates: &'a CityTemplates,
    random: &'a mut RandomGenerator,
    /// `TOWER_BRIDGE_GENERATOR.shipCreated` (EndCityPieces.java:104): at most
    /// one end ship per city. Reset by the `init()` calls at the start of
    /// `startHouseTower` (l.173-176); a fresh state per structure is that
    /// reset.
    ship_created: bool,
}

/// `EndCityPieces.addPiece` (EndCityPieces.java:165-170).
/// `calculateConnectedPosition(parentSettings, offset, childSettings, ZERO)`
/// (StructureTemplate.java:224-228) reduces — pivot `ZERO`, mirror `NONE`,
/// child connection point `ZERO` — to rotating `offset` by the parent's
/// rotation (StructureTemplate.java:441-467).
fn add_piece(
    templates: &CityTemplates,
    parent: PieceRef,
    offset: Vector3<i32>,
    name: &str,
    rotation: Rotation,
    overwrite: bool,
) -> CityPiece {
    let (dx, dz) = parent.rotation.rotate_offset(offset.x, offset.z);
    let position = Vector3::new(
        parent.position.x + dx,
        parent.position.y + offset.y,
        parent.position.z + dz,
    );
    CityPiece::new(templates.get(name), position, rotation, overwrite)
}

/// `EndCityPieces.addHelper` (EndCityPieces.java:184-187).
fn add_helper(pieces: &mut Vec<CityPiece>, piece: CityPiece) -> PieceRef {
    let piece_ref = piece.piece_ref();
    pieces.push(piece);
    piece_ref
}

/// The ubiquitous vanilla combination
/// `addHelper(pieces, addPiece(manager, parent, offset, name, rotation, OW))`
/// (first used at EndCityPieces.java:51).
fn add(
    templates: &CityTemplates,
    pieces: &mut Vec<CityPiece>,
    parent: PieceRef,
    offset: (i32, i32, i32),
    name: &str,
    rotation: Rotation,
    overwrite: bool,
) -> PieceRef {
    let offset = Vector3::new(offset.0, offset.1, offset.2);
    add_helper(
        pieces,
        add_piece(templates, parent, offset, name, rotation, overwrite),
    )
}

/// `setGenDepth` on the piece just pushed by [`add`]
/// (EndCityPieces.java:116/136).
fn set_last_gen_depth(pieces: &mut [CityPiece], gen_depth: i32) {
    if let Some(piece) = pieces.last_mut() {
        piece.gen_depth = gen_depth;
    }
}

/// `EndCityPieces.startHouseTower` (EndCityPieces.java:172-182).
fn start_house_tower(
    state: &mut GenState,
    origin: Vector3<i32>,
    rotation: Rotation,
    pieces: &mut Vec<CityPiece>,
) {
    // The generator init() calls (l.173-176) only reset `shipCreated`, which a
    // fresh `GenState` already covers.
    // l.177: base_floor directly at the start position, OW = true.
    let last = add_helper(
        pieces,
        CityPiece::new(state.templates.get("base_floor"), origin, rotation, true),
    );
    // l.178-180.
    let last = add(
        state.templates,
        pieces,
        last,
        (-1, 0, -1),
        "second_floor_1",
        rotation,
        false,
    );
    let last = add(
        state.templates,
        pieces,
        last,
        (-1, 4, -1),
        "third_floor_1",
        rotation,
        false,
    );
    let last = add(
        state.templates,
        pieces,
        last,
        (-1, 8, -1),
        "third_roof",
        rotation,
        true,
    );
    // l.181: recursion starts at depth 1. Vanilla passes a null offset; only
    // the house tower reads it.
    recursive_children(
        state,
        SectionGenerator::Tower,
        1,
        last,
        Vector3::new(0, 0, 0),
        pieces,
    );
}

/// `EndCityPieces.recursiveChildren` (EndCityPieces.java:189-210).
///
/// Children are generated into a local batch; nested recursion inside
/// `generate` checks against and appends to that same batch (vanilla passes
/// `childPieces` as the nested outer list). Afterwards the whole batch is
/// retagged with one `random.nextInt()` draw and rejected if any child
/// overlaps an already-accepted piece from a different batch than the parent.
fn recursive_children(
    state: &mut GenState,
    generator: SectionGenerator,
    gen_depth: i32,
    parent: PieceRef,
    offset: Vector3<i32>,
    pieces: &mut Vec<CityPiece>,
) -> bool {
    // l.190: recursion depth limit.
    if gen_depth > MAX_GEN_DEPTH {
        return false;
    }
    let mut child_pieces = Vec::new();
    if generate_section(
        state,
        generator,
        gen_depth,
        parent,
        offset,
        &mut child_pieces,
    ) {
        let mut collision = false;
        // l.196: the tag draw happens even when the batch is later rejected.
        let child_tag = state.random.next_i32();
        for child in &mut child_pieces {
            child.gen_depth = child_tag;
            // StructurePiece.findCollisionPiece (StructurePiece.java:379-385):
            // first accepted piece whose box intersects. Overlap is tolerated
            // only with pieces carrying the parent's batch tag (l.200).
            let collision_piece = pieces
                .iter()
                .find(|piece| piece.bounding_box.intersects(&child.bounding_box));
            if collision_piece.is_some_and(|piece| piece.gen_depth != parent.gen_depth) {
                collision = true;
                break;
            }
        }
        if !collision {
            // l.204-206: accept the whole batch.
            pieces.append(&mut child_pieces);
            return true;
        }
    }
    false
}

/// Dispatch over the four `SectionGenerator.generate` implementations. The
/// `offset` parameter is only read by the house tower (vanilla passes `null`
/// everywhere else, EndCityPieces.java:58/63/92/157/181).
fn generate_section(
    state: &mut GenState,
    generator: SectionGenerator,
    gen_depth: i32,
    parent: PieceRef,
    offset: Vector3<i32>,
    pieces: &mut Vec<CityPiece>,
) -> bool {
    match generator {
        SectionGenerator::HouseTower => {
            generate_house_tower(state, gen_depth, parent, offset, pieces)
        }
        SectionGenerator::Tower => generate_tower(state, gen_depth, parent, pieces),
        SectionGenerator::TowerBridge => generate_tower_bridge(state, gen_depth, parent, pieces),
        SectionGenerator::FatTower => generate_fat_tower(state, gen_depth, parent, pieces),
    }
}

/// `HOUSE_TOWER_GENERATOR` (EndCityPieces.java:39-67).
fn generate_house_tower(
    state: &mut GenState,
    gen_depth: i32,
    parent: PieceRef,
    offset: Vector3<i32>,
    pieces: &mut Vec<CityPiece>,
) -> bool {
    // l.47.
    if gen_depth > MAX_GEN_DEPTH {
        return false;
    }
    // l.50.
    let rotation = parent.rotation;
    // l.51: base_floor at the caller-provided offset, OW = true.
    let last = add(
        state.templates,
        pieces,
        parent,
        (offset.x, offset.y, offset.z),
        "base_floor",
        rotation,
        true,
    );
    // l.52.
    let num_floors = state.random.next_bounded_i32(3);
    if num_floors == 0 {
        // l.54.
        add(
            state.templates,
            pieces,
            last,
            (-1, 4, -1),
            "base_roof",
            rotation,
            true,
        );
    } else {
        // Cases 1 and 2 both start with second_floor_2 (l.56/l.60) and end
        // with a tower recursion (l.58/l.63); no draws differ.
        let last = add(
            state.templates,
            pieces,
            last,
            (-1, 0, -1),
            "second_floor_2",
            rotation,
            false,
        );
        let last = if num_floors == 1 {
            // l.57.
            add(
                state.templates,
                pieces,
                last,
                (-1, 8, -1),
                "second_roof",
                rotation,
                false,
            )
        } else {
            // numFloors == 2 (l.61-62).
            let last = add(
                state.templates,
                pieces,
                last,
                (-1, 4, -1),
                "third_floor_2",
                rotation,
                false,
            );
            add(
                state.templates,
                pieces,
                last,
                (-1, 8, -1),
                "third_roof",
                rotation,
                true,
            )
        };
        recursive_children(
            state,
            SectionGenerator::Tower,
            gen_depth + 1,
            last,
            Vector3::new(0, 0, 0),
            pieces,
        );
    }
    // l.65.
    true
}

/// `TOWER_GENERATOR` (EndCityPieces.java:69-102).
fn generate_tower(
    state: &mut GenState,
    gen_depth: i32,
    parent: PieceRef,
    pieces: &mut Vec<CityPiece>,
) -> bool {
    // l.77.
    let rotation = parent.rotation;
    // l.79: `new BlockPos(3 + nextInt(2), -3, 3 + nextInt(2))` — Java
    // evaluates constructor arguments left to right: X draw, then Z draw.
    let base_x = 3 + state.random.next_bounded_i32(2);
    let base_z = 3 + state.random.next_bounded_i32(2);
    let mut last = add(
        state.templates,
        pieces,
        parent,
        (base_x, -3, base_z),
        "tower_base",
        rotation,
        true,
    );
    // l.80.
    last = add(
        state.templates,
        pieces,
        last,
        (0, 7, 0),
        "tower_piece",
        rotation,
        true,
    );
    // l.81: 1-in-3 chance to bridge from the first tower piece.
    let mut bridge_from = (state.random.next_bounded_i32(3) == 0).then_some(last);
    // l.82.
    let tower_height = 1 + state.random.next_bounded_i32(3);
    // l.83-87: stack tower pieces; below the top, a coin flip may move the
    // bridge attachment up to the current piece (no draw on the last one).
    for i in 0..tower_height {
        last = add(
            state.templates,
            pieces,
            last,
            (0, 4, 0),
            "tower_piece",
            rotation,
            true,
        );
        if i < tower_height - 1 && state.random.next_bool() {
            bridge_from = Some(last);
        }
    }
    // l.98: without a bridge attachment — and below the depth-7 cap of
    // l.95 — the tower turns into a fat tower instead of being topped.
    if bridge_from.is_none() && gen_depth != 7 {
        return recursive_children(
            state,
            SectionGenerator::FatTower,
            gen_depth + 1,
            last,
            Vector3::new(0, 0, 0),
            pieces,
        );
    }
    if let Some(bridge_from) = bridge_from {
        // l.89-93: one coin flip per TOWER_BRIDGES entry; on success the
        // bridge_end is rotated by the entry rotation
        // (`Rotation.getRotated`, Rotation.java:44-102 == `Rotation::then`)
        // and a tower-bridge batch is grown from it.
        for (bridge_rotation, bridge_offset) in TOWER_BRIDGES {
            if state.random.next_bool() {
                let bridge_start = add(
                    state.templates,
                    pieces,
                    bridge_from,
                    bridge_offset,
                    "bridge_end",
                    rotation.then(bridge_rotation),
                    true,
                );
                recursive_children(
                    state,
                    SectionGenerator::TowerBridge,
                    gen_depth + 1,
                    bridge_start,
                    Vector3::new(0, 0, 0),
                    pieces,
                );
            }
        }
    }
    // l.94/l.96: both remaining branches cap the tower identically.
    add(
        state.templates,
        pieces,
        last,
        (-1, 4, -1),
        "tower_top",
        rotation,
        true,
    );
    // l.100.
    true
}

/// `TOWER_BRIDGE_GENERATOR` (EndCityPieces.java:103-139).
fn generate_tower_bridge(
    state: &mut GenState,
    gen_depth: i32,
    parent: PieceRef,
    pieces: &mut Vec<CityPiece>,
) -> bool {
    // l.113.
    let rotation = parent.rotation;
    // l.114.
    let bridge_length = state.random.next_bounded_i32(4) + 1;
    // l.115.
    let mut last = add(
        state.templates,
        pieces,
        parent,
        (0, 0, -4),
        "bridge_piece",
        rotation,
        true,
    );
    // l.116: tag the opening piece -1 so nested batches treat overlaps with it
    // as foreign and reject.
    set_last_gen_depth(pieces, -1);
    // l.117.
    let mut next_y = 0;
    // l.118-126: flat piece (coin flip) keeps the level; otherwise a second
    // coin flip picks steep stairs at Z -4 or gentle stairs at Z -8, raising
    // the following piece by 4.
    for _ in 0..bridge_length {
        if state.random.next_bool() {
            last = add(
                state.templates,
                pieces,
                last,
                (0, next_y, -4),
                "bridge_piece",
                rotation,
                true,
            );
            next_y = 0;
        } else {
            last = if state.random.next_bool() {
                add(
                    state.templates,
                    pieces,
                    last,
                    (0, next_y, -4),
                    "bridge_steep_stairs",
                    rotation,
                    true,
                )
            } else {
                add(
                    state.templates,
                    pieces,
                    last,
                    (0, next_y, -8),
                    "bridge_gentle_stairs",
                    rotation,
                    true,
                )
            };
            next_y = 4;
        }
    }
    // l.127: `this.shipCreated || random.nextInt(10 - genDepth) != 0` — the
    // draw is short-circuited away once a ship exists. At the maximum depth of
    // 8 this is nextInt(2), i.e. a 50% ship chance.
    if state.ship_created || state.random.next_bounded_i32(10 - gen_depth) != 0 {
        // l.128: grow a house tower anchored at (-3, nextY + 1, -11); if its
        // batch is rejected the whole bridge is rejected.
        if !recursive_children(
            state,
            SectionGenerator::HouseTower,
            gen_depth + 1,
            last,
            Vector3::new(-3, next_y + 1, -11),
            pieces,
        ) {
            return false;
        }
    } else {
        // l.132: `new BlockPos(-8 + nextInt(8), nextY, -70 + nextInt(10))` —
        // X draw then Z draw. The ship is added but `lastPiece` stays on the
        // bridge, so the closing bridge_end still parents off the bridge.
        let ship_x = -8 + state.random.next_bounded_i32(8);
        let ship_z = -70 + state.random.next_bounded_i32(10);
        add(
            state.templates,
            pieces,
            last,
            (ship_x, next_y, ship_z),
            "ship",
            rotation,
            true,
        );
        // l.133: once-only flag.
        state.ship_created = true;
    }
    // l.135: closing bridge end, rotated 180 degrees.
    add(
        state.templates,
        pieces,
        last,
        (4, next_y, 0),
        "bridge_end",
        rotation.then(Rotation::Rotate180),
        true,
    );
    // l.136.
    set_last_gen_depth(pieces, -1);
    true
}

/// `FAT_TOWER_GENERATOR` (EndCityPieces.java:141-163).
fn generate_fat_tower(
    state: &mut GenState,
    gen_depth: i32,
    parent: PieceRef,
    pieces: &mut Vec<CityPiece>,
) -> bool {
    // l.149.
    let rotation = parent.rotation;
    // l.150.
    let mut last = add(
        state.templates,
        pieces,
        parent,
        (-3, 4, -3),
        "fat_tower_base",
        rotation,
        true,
    );
    // l.151.
    last = add(
        state.templates,
        pieces,
        last,
        (0, 4, 0),
        "fat_tower_middle",
        rotation,
        true,
    );
    // l.152: `for (int i = 0; i < 2 && random.nextInt(3) != 0; ++i)` — the
    // 1-in-3 stop chance is drawn before each of the two possible extra tiers.
    let mut tier = 0;
    while tier < 2 && state.random.next_bounded_i32(3) != 0 {
        // l.153.
        last = add(
            state.templates,
            pieces,
            last,
            (0, 8, 0),
            "fat_tower_middle",
            rotation,
            true,
        );
        // l.154-158: one coin flip per FAT_TOWER_BRIDGES entry.
        for (bridge_rotation, bridge_offset) in FAT_TOWER_BRIDGES {
            if state.random.next_bool() {
                let bridge_start = add(
                    state.templates,
                    pieces,
                    last,
                    bridge_offset,
                    "bridge_end",
                    rotation.then(bridge_rotation),
                    true,
                );
                recursive_children(
                    state,
                    SectionGenerator::TowerBridge,
                    gen_depth + 1,
                    bridge_start,
                    Vector3::new(0, 0, 0),
                    pieces,
                );
            }
        }
        tier += 1;
    }
    // l.160.
    add(
        state.templates,
        pieces,
        last,
        (-2, 8, -2),
        "fat_tower_top",
        rotation,
        true,
    );
    // l.161.
    true
}

pub struct EndCityGenerator;

impl StructureGenerator for EndCityGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        // Rotation.getRandom (Rotation.java:131-133) → Util.getRandom
        // (Util.java:756-758): `values()[random.nextInt(4)]` over the
        // declaration order NONE, CLOCKWISE_90, CLOCKWISE_180,
        // COUNTERCLOCKWISE_90 (Rotation.java:26-29).
        let rotation = match context.random.next_bounded_i32(4) {
            0 => Rotation::None,
            1 => Rotation::Clockwise90,
            2 => Rotation::Rotate180,
            _ => Rotation::CounterClockwise90,
        };

        // getLowestYIn5by5BoxOffset7Blocks (Structure.java:165-180): a 5x5
        // box anchored at chunk-local (7, 7), extended toward the quadrant the
        // rotated footprint occupies.
        let (offset_x, offset_z) = match rotation {
            Rotation::None => (5, 5),
            Rotation::Clockwise90 => (-5, 5),
            Rotation::Rotate180 => (-5, -5),
            Rotation::CounterClockwise90 => (5, -5),
        };
        // ChunkPos.getBlockX(7) / getBlockZ(7) (Structure.java:177-178).
        let block_x = get_offset_x(context.chunk_x, 7);
        let block_z = get_offset_z(context.chunk_z, 7);

        // getCornerHeights (Structure.java:140-145) probes WORLD_SURFACE_WG
        // first-occupied heights at the four box corners and getLowestY
        // (Structure.java:159-162) takes the minimum. `estimate_height` is
        // Pumpkin's stand-in for `ChunkGenerator.getFirstOccupiedHeight` (see
        // the same mapping in mineshaft.rs:144-148); without a sampler the
        // probe — and therefore the structure — is impossible.
        let sampler = context.height_sampler.as_mut()?;
        let corners = [
            (block_x, block_z),
            (block_x, block_z + offset_z),
            (block_x + offset_x, block_z),
            (block_x + offset_x, block_z + offset_z),
        ];
        let mut lowest_y = i32::MAX;
        for (x, z) in corners {
            lowest_y = lowest_y.min(sampler.estimate_height(x, z));
        }
        // EndCityStructure.findGenerationPoint (EndCityStructure.java:32-35).
        if lowest_y < MIN_START_Y {
            return None;
        }

        let templates = CityTemplates::load()?;
        let origin = Vector3::new(block_x, lowest_y, block_z);
        let mut pieces = Vec::new();
        let mut state = GenState {
            templates: &templates,
            random: &mut context.random,
            ship_created: false,
        };
        // EndCityStructure.generatePieces (EndCityStructure.java:39-43).
        start_house_tower(&mut state, origin, rotation, &mut pieces);

        let mut collector = StructurePiecesCollector::default();
        for piece in pieces {
            collector.add_piece(Box::new(piece.into_structure_piece()));
        }

        Some(StructurePosition {
            start_pos: BlockPos::new(block_x, lowest_y, block_z),
            collector: Arc::new(collector.into()),
        })
    }
}

/// A placed End City template piece (vanilla `EndCityPieces.EndCityPiece`,
/// EndCityPieces.java:212-264 — one `StructurePieceType.END_CITY_PIECE` for
/// every template).
pub struct EndCityPiece {
    piece: StructurePiece,
    template: Arc<StructureTemplate>,
    rotation: Rotation,
    /// See [`CityPiece::overwrite`].
    overwrite: bool,
}

impl StructurePieceBase for EndCityPiece {
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
        // `makeSettings` (EndCityPieces.java:222-225): OW = true keeps
        // template air (only structure blocks are ignored), OW = false skips
        // air as well. `place_template` always skips structure blocks
        // (template/mod.rs:141-146), so only the air behavior varies.
        // The origin is the bounding-box min corner — see the module docs for
        // why that reproduces vanilla's pivot-zero placement exactly.
        place_template(
            chunk,
            &self.template,
            self.piece.bounding_box.min,
            (0, 0),
            self.rotation,
            !self.overwrite,
            false,
            &[],
            Some(chunk_box),
            seed,
        );
        self.handle_data_markers(chunk, random, chunk_box);
    }
}

impl EndCityPiece {
    /// `TemplateStructurePiece` dispatches structure blocks in DATA mode to
    /// `handleDataMarker` (EndCityPieces.java:244-263). `place_template`
    /// drops structure blocks without reading their metadata
    /// (template/mod.rs:141-146), so the markers are re-walked here with the
    /// same rotation transform used for block placement.
    fn handle_data_markers(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        chunk_box: &BlockBox,
    ) {
        let origin = self.piece.bounding_box.min;
        for block in &self.template.blocks {
            let entry = &self.template.palette[block.state as usize];
            if entry.name != "minecraft:structure_block" {
                continue;
            }
            let Some(marker) = block
                .nbt
                .as_ref()
                .filter(|nbt| nbt.get_string("mode") == Some("DATA"))
                .and_then(|nbt| nbt.get_string("metadata"))
            else {
                continue;
            };
            let local = self.rotation.transform_pos(block.pos, self.template.size);
            let pos = Vector3::new(origin.x + local.x, origin.y + local.y, origin.z + local.z);
            // Marker prefixes per EndCityPieces.java:245/251/257.
            if marker.starts_with("Chest") {
                place_treasure_chest(chunk, random, chunk_box, pos);
            } else if chunk_box.contains_pos(&pos) {
                // `Level.isInSpawnableBounds` (EndCityPieces.java:250) is
                // implied by the chunk box spanning the full build height.
                if marker.starts_with("Sentry") {
                    place_sentry(chunk, pos);
                } else if marker.starts_with("Elytra") {
                    self.place_elytra_frame(chunk, pos);
                }
            }
        }
    }

    /// "Elytra" marker (EndCityPieces.java:257-261): an item frame holding an
    /// elytra, facing `rotation.rotate(Direction.SOUTH)`. Queued through the
    /// generic pending-entity path; the NBT follows the vanilla save format
    /// (`block_pos` from BlockAttachedEntity.java:140-141, `Facing`/`Item`
    /// from ItemFrame.java:338-348), so fidelity of orientation and contents
    /// depends on the item-frame entity honoring those fields on load.
    fn place_elytra_frame(&self, chunk: &mut ProtoChunk, pos: Vector3<i32>) {
        // `Rotation.rotate(Direction)` (Rotation.java:110-120) applied to
        // SOUTH, encoded as the legacy 3D direction id used by
        // `Direction.LEGACY_ID_CODEC` (ItemFrame.java:346): down = 0, up = 1,
        // north = 2, south = 3, west = 4, east = 5.
        let facing = match self.rotation {
            Rotation::None => 3,               // south
            Rotation::Clockwise90 => 4,        // west
            Rotation::Rotate180 => 2,          // north
            Rotation::CounterClockwise90 => 5, // east
        };
        let mut nbt = NbtCompound::new();
        nbt.put_string("id", "minecraft:item_frame".to_string());
        nbt.put_list(
            "Pos",
            vec![
                NbtTag::Double(f64::from(pos.x) + 0.5),
                NbtTag::Double(f64::from(pos.y) + 0.5),
                NbtTag::Double(f64::from(pos.z) + 0.5),
            ],
        );
        nbt.put("block_pos", NbtTag::IntArray(vec![pos.x, pos.y, pos.z]));
        nbt.put_int("Facing", facing);
        let mut item = NbtCompound::new();
        item.put_string("id", "minecraft:elytra".to_string());
        item.put_int("count", 1);
        nbt.put_compound("Item", item);
        chunk.add_entity(nbt);
    }
}

/// "Chest" marker (EndCityPieces.java:245-249): the chest one block below the
/// marker (placed by the template itself) gets the END_CITY_TREASURE loot
/// table via `RandomizableContainer.setBlockEntityLootTable`
/// (RandomizableContainer.java:51-57), seeded with `random.nextLong()`.
fn place_treasure_chest(
    chunk: &mut ProtoChunk,
    random: &mut RandomGenerator,
    chunk_box: &BlockBox,
    marker_pos: Vector3<i32>,
) {
    // l.246: `position.below()`.
    let chest_pos = Vector3::new(marker_pos.x, marker_pos.y - 1, marker_pos.z);
    // l.247: only the chunk containing the chest applies the loot table.
    if !chunk_box.contains_pos(&chest_pos) {
        return;
    }
    // Pending block entities are keyed by position at chunk finalization
    // (chunk_system/chunk_state.rs:303-312), so this entry replaces the
    // loot-less chest block entity queued by `place_template`.
    let mut nbt = NbtCompound::new();
    nbt.put_string("id", "minecraft:chest".to_string());
    nbt.put_int("x", chest_pos.x);
    nbt.put_int("y", chest_pos.y);
    nbt.put_int("z", chest_pos.z);
    nbt.put_string("LootTable", END_CITY_TREASURE_LOOT.to_string());
    nbt.put_long("LootTableSeed", random.next_i64());
    chunk.add_block_entity(nbt);
}

/// "Sentry" marker (EndCityPieces.java:251-256): a shulker at the marker
/// position (`setPos(x + 0.5, y, z + 0.5)`, l.254), queued as a worldgen
/// entity like the mineshaft chest minecart (mineshaft.rs:1796-1817).
fn place_sentry(chunk: &mut ProtoChunk, pos: Vector3<i32>) {
    let mut nbt = NbtCompound::new();
    nbt.put_string("id", "minecraft:shulker".to_string());
    nbt.put_list(
        "Pos",
        vec![
            NbtTag::Double(f64::from(pos.x) + 0.5),
            NbtTag::Double(f64::from(pos.y)),
            NbtTag::Double(f64::from(pos.z) + 0.5),
        ],
    );
    nbt.put_list(
        "Motion",
        vec![
            NbtTag::Double(0.0),
            NbtTag::Double(0.0),
            NbtTag::Double(0.0),
        ],
    );
    chunk.add_entity(nbt);
}
