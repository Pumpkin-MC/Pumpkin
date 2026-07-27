//! NBT Structure Template System
//!
//! This module provides functionality for loading and placing Minecraft structure
//! templates from `.nbt` files. This enables exact vanilla structure matching and
//! dramatically simplifies implementing structures like igloos, shipwrecks, villages, etc.
//!
//! # Architecture
//!
//! - [`StructureTemplate`]: Represents a loaded NBT template with size, palette, and blocks
//! - [`TemplatePiece`]: A structure piece that places blocks from a template
//! - [`Rotation`] and [`Mirror`]: Transform positions and block properties
//! - [`TemplateCache`]: Lazy-loading cache for embedded template files
//!
//! # Example Usage
//!
//! ```ignore
//! use pumpkin_world::generation::structure::template::{TemplateCache, TemplatePiece};
//! use pumpkin_data::Rotation;
//!
//! // Load a template from the cache
//! let template = TemplateCache::get("igloo/top").expect("Template not found");
//!
//! // Create a piece to place the template
//! let piece = TemplatePiece::new(template, rotation, mirror, position);
//! ```

mod block_state_resolver;
mod cache;
pub mod processor;
mod structure_template;
mod template_piece;

use pumpkin_data::BlockState;
use pumpkin_data::Mirror;
use pumpkin_data::Rotation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomImpl, hash_block_pos, legacy_rand::LegacyRand};

use crate::ProtoChunk;

pub use block_state_resolver::BlockStateResolver;
pub use cache::{
    TemplateCache, get_pool_elements, get_processor_list_json, get_template,
    get_template_pool_json, global_cache,
};
pub use processor::StructureProcessor;
pub use pumpkin_data::{Mirror as BlockMirror, Rotation as BlockRotation};
pub use structure_template::{PaletteEntry, StructureTemplate, TemplateBlock, TemplateEntity};
pub use template_piece::TemplatePiece;

/// Places a template at a world origin with an un-rotated XZ offset.
///
/// All rotation is handled internally:
/// - The offset is rotated to position the template correctly
/// - Block positions within the template are rotated
/// - Directional block properties (facing, axis, etc.) are rotated
/// - Block entities are created from template NBT data
///
/// `origin` is the base world position (x, y, z).
/// `offset` is the un-rotated XZ offset from origin (`x_offset`, `z_offset`) - rotation is applied automatically.
/// `world_seed` feeds the capped-processor selection random
/// (`CappedProcessor.java:62` forks the world random at the piece origin).
#[allow(clippy::too_many_arguments)]
pub fn place_template(
    chunk: &mut ProtoChunk,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    offset: (i32, i32),
    rotation: Rotation,
    skip_air: bool,
    apply_waterlogging: bool,
    processors: &[StructureProcessor],
    chunk_box: Option<&pumpkin_util::math::block_box::BlockBox>,
    world_seed: i64,
) {
    let (rotated_ox, rotated_oz) = rotation.rotate_offset(offset.0, offset.1);
    let world_x = origin.x + rotated_ox;
    let world_z = origin.z + rotated_oz;

    place_template_blocks(
        chunk,
        template,
        origin,
        world_x,
        world_z,
        rotation,
        skip_air,
        apply_waterlogging,
        processors,
        chunk_box,
        world_seed,
    );
    place_template_entities(
        chunk, template, origin, world_x, world_z, rotation, chunk_box,
    );
}

/// A template block that survived per-block processing and is waiting for the
/// finalize + write passes (vanilla `processedBlockInfoList`).
struct PendingBlock {
    world_pos: Vector3<i32>,
    /// Raw template-local position; vanilla keeps the original block infos and
    /// hands `originalBlockInfo.pos()` to finalize delegates as the
    /// `templateRelativePos` argument (CappedProcessor.java:73).
    template_pos: Vector3<i32>,
    /// Untransformed processed state — vanilla processors run before rotation
    /// is applied (`StructureTemplate.java:381-386`); rotation happens at
    /// write time.
    state: &'static BlockState,
    entry: PaletteEntry,
    nbt: Option<NbtCompound>,
    loot: Option<(std::sync::Arc<str>, i64)>,
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn place_template_blocks(
    chunk: &mut ProtoChunk,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    world_x: i32,
    world_z: i32,
    rotation: Rotation,
    skip_air: bool,
    apply_waterlogging: bool,
    processors: &[StructureProcessor],
    chunk_box: Option<&pumpkin_util::math::block_box::BlockBox>,
    world_seed: i64,
) {
    // Vanilla only clips to the chunk during processing when no processor
    // evaluates the entire piece (StructureTemplate.java:373-382); a capped
    // processor must select over the full piece so every chunk agrees on
    // which blocks were chosen, and clipping happens at write time instead.
    let has_capped = processors
        .iter()
        .any(|processor| matches!(processor, StructureProcessor::Capped { .. }));
    let clip_early = !has_capped;

    let mut pending: Vec<PendingBlock> = Vec::new();

    for block in &template.blocks {
        let palette_entry = &template.palette[block.state as usize];

        // Structure blocks are data markers and structure void preserves the existing block.
        if palette_entry.name == "minecraft:structure_void"
            || palette_entry.name == "minecraft:structure_block"
        {
            continue;
        }

        // Skip air blocks when using IGNORE_AIR processor (e.g. nether fossils)
        if skip_air && palette_entry.name == "minecraft:air" {
            continue;
        }

        let mut block_entity_nbt = block.nbt.clone();
        let mut placed_entry = palette_entry.clone();

        // Jigsaw blocks are replaced during template processing, before block entities are
        // collected. Keeping this in the placement pipeline avoids stale jigsaw entities
        // and matches vanilla's processor order: JigsawReplacementProcessor runs before
        // the data-driven list (SinglePoolElement.java:159-165).
        if palette_entry.name == "minecraft:jigsaw" {
            let final_state = block_entity_nbt
                .as_ref()
                .and_then(|nbt| nbt.get_string("final_state"))
                .unwrap_or("minecraft:air");
            placed_entry = PaletteEntry::from_string(final_state);
            block_entity_nbt = None;
            // JigsawReplacementProcessor.java:58-60: a final_state of
            // structure_void drops the block entirely.
            if placed_entry.name == "minecraft:structure_void"
                || placed_entry.name == "structure_void"
            {
                continue;
            }
        }

        // Vanilla processors run on the saved template state; rotation is only
        // applied to the surviving state at write time
        // (StructureTemplate.java:383, placeInWorld).
        let Some(mut state) = BlockStateResolver::resolve_simple(&placed_entry) else {
            continue;
        };

        // Rotate block position within template bounds
        let local_pos = rotation.transform_pos(block.pos, template.size);

        let wx = world_x + local_pos.x;
        let wy = origin.y + local_pos.y;
        let wz = world_z + local_pos.z;

        if clip_early
            && let Some(bbox) = chunk_box
            && (wx < bbox.min.x
                || wx > bbox.max.x
                || wy < bbox.min.y
                || wy > bbox.max.y
                || wz < bbox.min.z
                || wz > bbox.max.z)
        {
            continue;
        }

        let mut world_pos = Vector3::new(wx, wy, wz);

        // Apply per-block processors. Vanilla threads the (possibly moved)
        // processed info through the chain while always passing the raw
        // template-local position (StructureTemplate.java:384-387).
        let mut loot = None;
        let mut dropped = false;
        for processor in processors {
            let Some(processed) = processor.process(chunk, world_pos, block.pos, state) else {
                dropped = true;
                break;
            };
            world_pos = processed.pos;
            state = processed.state;
            if processed.loot.is_some() {
                loot = processed.loot;
            }
        }
        if dropped {
            continue;
        }

        pending.push(PendingBlock {
            world_pos,
            template_pos: block.pos,
            state,
            entry: placed_entry,
            nbt: block_entity_nbt,
            loot,
        });
    }

    finalize_capped_processors(chunk, processors, origin, world_seed, &mut pending);

    for pending_block in pending {
        let world_pos = pending_block.world_pos;
        if let Some(bbox) = chunk_box
            && !bbox.contains_pos(&world_pos)
        {
            continue;
        }

        let mut state = pumpkin_data::block_state_transform::transform_block_state(
            pending_block.state.id,
            Mirror::default(),
            rotation,
        );

        // Vanilla waterlogs the final rotated state when it stands in water
        // (StructureTemplate.placeInWorld fluid handling).
        if apply_waterlogging
            && chunk.get_block_state(&world_pos).to_block_id() == pumpkin_data::Block::WATER.id
        {
            state = with_waterlogged(state);
        }

        chunk.set_block_state(world_pos.x, world_pos.y, world_pos.z, state);
        place_block_entity(
            chunk,
            &pending_block.entry,
            pending_block.nbt.as_ref(),
            world_pos.x,
            world_pos.y,
            world_pos.z,
            pending_block.loot,
            state,
        );
    }
}

/// Vanilla `CappedProcessor.finalizeProcessing` (CappedProcessor.java:54-80):
/// fork the world random at the piece origin, shuffle the processed block
/// list, and run the delegate over shuffled entries until `limit` of them
/// actually changed.
fn finalize_capped_processors(
    chunk: &ProtoChunk,
    processors: &[StructureProcessor],
    origin: Vector3<i32>,
    world_seed: i64,
    pending: &mut [PendingBlock],
) {
    for processor in processors {
        let StructureProcessor::Capped { limit, delegate } = processor else {
            continue;
        };
        if *limit == 0 || pending.is_empty() {
            continue;
        }

        // RandomSource.createThreadLocalInstance(seed).forkPositional().at(position)
        // (CappedProcessor.java:62, LegacyRandomSource.java:73-77).
        let fork_seed = LegacyRand::from_seed(world_seed as u64).next_i64();
        let mut random = LegacyRand::from_seed(
            (hash_block_pos(origin.x, origin.y, origin.z) ^ fork_seed) as u64,
        );

        let max_to_replace = (*limit).min(pending.len() as i32);
        if max_to_replace < 1 {
            continue;
        }

        // Util.toShuffledList (Util.java:1013-1021).
        let mut indices: Vec<usize> = (0..pending.len()).collect();
        for i in (2..=indices.len()).rev() {
            let swap_to = random.next_bounded_i32(i as i32) as usize;
            indices.swap(i - 1, swap_to);
        }

        let mut replaced = 0;
        for &index in &indices {
            if replaced >= max_to_replace {
                break;
            }
            let entry = &mut pending[index];
            // CappedProcessor.java:73: the delegate gets the original
            // template-local position as `templateRelativePos`.
            let Some(processed) =
                delegate.process(chunk, entry.world_pos, entry.template_pos, entry.state)
            else {
                continue;
            };
            // Vanilla counts an entry only when the delegate changed it
            // (CappedProcessor.java:74-77: `processedBlockInfo.equals(...)`
            // compares position, state and nbt).
            if processed.state.id == entry.state.id
                && processed.pos == entry.world_pos
                && processed.loot.is_none()
            {
                continue;
            }
            entry.world_pos = processed.pos;
            entry.state = processed.state;
            if processed.loot.is_some() {
                entry.loot = processed.loot;
            }
            replaced += 1;
        }
    }
}

/// Returns the state with `waterlogged=true` if the block supports it.
fn with_waterlogged(state: &'static BlockState) -> &'static BlockState {
    let block = pumpkin_data::Block::from_state_id(state.id);
    let Some(properties) = block.properties(state.id) else {
        return state;
    };
    let mut props = properties.to_props();
    let mut found = false;
    for (name, value) in &mut props {
        if *name == "waterlogged" {
            *value = "true";
            found = true;
        }
    }
    if !found {
        return state;
    }
    BlockState::from_id(block.from_properties(&props).to_state_id(block))
}

#[allow(clippy::too_many_arguments)]
fn place_block_entity(
    chunk: &mut ProtoChunk,
    placed_entry: &PaletteEntry,
    block_entity_nbt: Option<&NbtCompound>,
    wx: i32,
    wy: i32,
    wz: i32,
    loot: Option<(std::sync::Arc<str>, i64)>,
    placed_state: &'static BlockState,
) {
    // A processor may have replaced the template block with a block-entity
    // block (append_loot turns gravel into suspicious gravel), so the id
    // lookup must consider the placed state, not just the palette name.
    let block_entity_id = get_block_entity_id(&placed_entry.name).or_else(|| {
        let placed_block = pumpkin_data::Block::from_state_id(placed_state.id);
        get_block_entity_id(&format!("minecraft:{}", placed_block.name))
    });
    if block_entity_nbt.is_none() && block_entity_id.is_none() && loot.is_none() {
        return;
    }
    let block_entity_id = block_entity_id.unwrap_or(&placed_entry.name);
    let mut placed_nbt = NbtCompound::new();

    placed_nbt.put_string("id", block_entity_id.to_string());
    placed_nbt.put_int("x", wx);
    placed_nbt.put_int("y", wy);
    placed_nbt.put_int("z", wz);

    if let Some(template_nbt) = block_entity_nbt {
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

    // Vanilla append_loot stores the table plus a seed drawn from the rule
    // random (AppendLoot.java:34-40).
    if let Some((table, seed)) = loot {
        placed_nbt.put_string("LootTable", table.to_string());
        placed_nbt.put_long("LootTableSeed", seed);
    }

    if placed_nbt.get_string("LootTable").is_some()
        && placed_nbt.get_long("LootTableSeed").is_none()
    {
        let mut random = LegacyRand::from_seed(hash_block_pos(wx, wy, wz) as u64);
        placed_nbt.put_long("LootTableSeed", random.next_i64());
    }

    chunk.add_block_entity(placed_nbt);
}

fn place_template_entities(
    chunk: &mut ProtoChunk,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    world_x: i32,
    world_z: i32,
    rotation: Rotation,
    chunk_box: Option<&pumpkin_util::math::block_box::BlockBox>,
) {
    // Spawn structure-template entities (villagers, iron golems, animals, …).
    // Vanilla places these when the structure piece is applied to the chunk.
    for entity in &template.entities {
        let local_block = rotation.transform_pos(entity.block_pos, template.size);
        let wx = world_x + local_block.x;
        let wy = origin.y + local_block.y;
        let wz = world_z + local_block.z;

        if let Some(bbox) = chunk_box
            && (wx < bbox.min.x
                || wx > bbox.max.x
                || wy < bbox.min.y
                || wy > bbox.max.y
                || wz < bbox.min.z
                || wz > bbox.max.z)
        {
            continue;
        }

        // Transform fractional position the same way as block coords.
        let local_pos = transform_entity_pos(rotation, entity.pos, template.size);
        let world_ex = f64::from(world_x) + local_pos.x;
        let world_ey = f64::from(origin.y) + local_pos.y;
        let world_ez = f64::from(world_z) + local_pos.z;

        let mut placed = entity.nbt.clone();
        // Template NBT already contains Pos/Motion/Rotation, so replace those
        // fields rather than using NbtCompound::put (which preserves old values).
        placed.child_tags.insert(
            "Pos".into(),
            NbtTag::List(vec![
                NbtTag::Double(world_ex),
                NbtTag::Double(world_ey),
                NbtTag::Double(world_ez),
            ]),
        );
        placed.child_tags.insert(
            "Motion".into(),
            NbtTag::List(vec![
                NbtTag::Double(0.0),
                NbtTag::Double(0.0),
                NbtTag::Double(0.0),
            ]),
        );
        let yaw = placed
            .get_list("Rotation")
            .and_then(|rotation| rotation.first())
            .and_then(NbtTag::extract_float)
            .unwrap_or(0.0);
        let pitch = placed
            .get_list("Rotation")
            .and_then(|rotation| rotation.get(1))
            .and_then(NbtTag::extract_float)
            .unwrap_or(0.0);
        let rotation_degrees = match rotation {
            Rotation::None => 0.0,
            Rotation::Clockwise90 => 90.0,
            Rotation::Rotate180 => 180.0,
            Rotation::CounterClockwise90 => 270.0,
        };
        let yaw = (yaw + 180.0).rem_euclid(360.0) - 180.0 + rotation_degrees;
        placed.child_tags.insert(
            "Rotation".into(),
            NbtTag::List(vec![NbtTag::Float(yaw), NbtTag::Float(pitch)]),
        );
        // Vanilla assigns a fresh UUID to every entity placed from a template.
        placed.child_tags.remove("UUID");
        chunk.add_entity(placed);
    }
}

/// Rotate an entity's relative double position within a template of `size`.
fn transform_entity_pos(rotation: Rotation, pos: Vector3<f64>, size: Vector3<i32>) -> Vector3<f64> {
    match rotation {
        Rotation::None => pos,
        Rotation::Clockwise90 => Vector3::new(f64::from(size.z) - pos.z, pos.y, pos.x),
        Rotation::Rotate180 => {
            Vector3::new(f64::from(size.x) - pos.x, pos.y, f64::from(size.z) - pos.z)
        }
        Rotation::CounterClockwise90 => Vector3::new(pos.z, pos.y, f64::from(size.x) - pos.x),
    }
}

/// Returns the block entity ID for blocks that require one, or None if not needed.
pub(crate) fn get_block_entity_id(block_name: &str) -> Option<&'static str> {
    match block_name {
        "minecraft:furnace" => Some("minecraft:furnace"),
        "minecraft:chest" => Some("minecraft:chest"),
        "minecraft:trapped_chest" => Some("minecraft:trapped_chest"),
        "minecraft:barrel" => Some("minecraft:barrel"),
        "minecraft:hopper" => Some("minecraft:hopper"),
        "minecraft:dropper" => Some("minecraft:dropper"),
        "minecraft:dispenser" => Some("minecraft:dispenser"),
        "minecraft:brewing_stand" => Some("minecraft:brewing_stand"),
        "minecraft:blast_furnace" => Some("minecraft:blast_furnace"),
        "minecraft:smoker" => Some("minecraft:smoker"),
        "minecraft:shulker_box" => Some("minecraft:shulker_box"),
        "minecraft:bed" => Some("minecraft:bed"),
        "minecraft:sign"
        | "minecraft:oak_sign"
        | "minecraft:spruce_sign"
        | "minecraft:birch_sign"
        | "minecraft:jungle_sign"
        | "minecraft:acacia_sign"
        | "minecraft:dark_oak_sign"
        | "minecraft:mangrove_sign"
        | "minecraft:cherry_sign"
        | "minecraft:bamboo_sign"
        | "minecraft:crimson_sign"
        | "minecraft:warped_sign" => Some("minecraft:sign"),
        "minecraft:hanging_sign" => Some("minecraft:hanging_sign"),
        "minecraft:suspicious_sand" | "minecraft:suspicious_gravel" => {
            Some("minecraft:brushable_block")
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Rotation, Vector3, transform_entity_pos};

    #[test]
    fn rotated_entity_position_stays_at_the_block_center() {
        let size = Vector3::new(5, 4, 7);
        let pos = Vector3::new(0.5, 1.0, 0.5);

        let clockwise = transform_entity_pos(Rotation::Clockwise90, pos, size);
        assert_eq!(clockwise, Vector3::new(6.5, 1.0, 0.5));

        let reversed = transform_entity_pos(Rotation::Rotate180, pos, size);
        assert_eq!(reversed, Vector3::new(4.5, 1.0, 6.5));

        let counter_clockwise = transform_entity_pos(Rotation::CounterClockwise90, pos, size);
        assert_eq!(counter_clockwise, Vector3::new(0.5, 1.0, 4.5));
    }
}
