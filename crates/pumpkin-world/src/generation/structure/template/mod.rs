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

use pumpkin_data::BlockStateId;
use pumpkin_data::Mirror;
use pumpkin_data::Rotation;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomImpl, hash_block_pos, legacy_rand::LegacyRand};

use crate::ProtoChunk;

pub use block_state_resolver::BlockStateResolver;
pub use cache::{
    TemplateCache, all_pool_names, all_structure_names, all_template_names, get_pool_elements,
    get_processor_list_json, get_template, get_template_pool_json, global_cache,
};
pub use processor::StructureProcessor;
pub use pumpkin_data::{BlockState, Mirror as BlockMirror, Rotation as BlockRotation};
pub use structure_template::{
    JigsawBlockInfo, Palette, PaletteEntry, SimplePalette, StructureBlockInfo, StructureEntityInfo,
    StructurePlaceSettings, StructureTemplate, TemplateBlock, TemplateEntity,
};
pub use template_piece::TemplatePiece;

/// Abstraction over block placement, implemented by both [`ProtoChunk`] (worldgen) and
/// [`WorldBlockPlacer`] (live `/place template` command).
pub trait BlockPlacer {
    fn get_block_state(&self, pos: &Vector3<i32>) -> BlockStateId;
    fn set_block_state(&mut self, pos: &Vector3<i32>, state: &BlockState);
    fn add_block_entity(&mut self, nbt: NbtCompound);
}

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
#[allow(clippy::too_many_arguments)]
pub fn place_template(
    placer: &mut impl BlockPlacer,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    offset: (i32, i32),
    rotation: Rotation,
    skip_air: bool,
    apply_waterlogging: bool,
    processors: &[StructureProcessor],
    chunk_box: Option<&pumpkin_util::math::block_box::BlockBox>,
) {
    place_template_with_options(
        placer,
        template,
        origin,
        offset,
        rotation,
        skip_air,
        apply_waterlogging,
        processors,
        chunk_box,
        false,
    );
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn place_template_with_options(
    placer: &mut impl BlockPlacer,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    offset: (i32, i32),
    rotation: Rotation,
    skip_air: bool,
    apply_waterlogging: bool,
    processors: &[StructureProcessor],
    chunk_box: Option<&pumpkin_util::math::block_box::BlockBox>,
    keep_jigsaws: bool,
) {
    let (rotated_ox, rotated_oz) = rotation.rotate_offset(offset.0, offset.1);
    let world_x = origin.x + rotated_ox;
    let world_z = origin.z + rotated_oz;

    let mut context_rng = LegacyRand::from_seed(hash_block_pos(world_x, origin.y, world_z) as u64);
    let mut context = processor::ProcessorContext::new(origin, processors, &mut context_rng);

    for block in &template.blocks {
        let palette_entry = &template.palette[block.state as usize];

        // Structure blocks are data markers.
        if palette_entry.name == "minecraft:structure_block" {
            continue;
        }

        let mut block_entity_nbt = block.nbt.clone();
        let mut placed_entry = palette_entry.clone();

        // Jigsaw blocks are replaced during template processing, before block entities are
        // collected. Keeping this in the placement pipeline avoids stale jigsaw entities.
        if !keep_jigsaws && palette_entry.name == "minecraft:jigsaw" {
            let final_state = block_entity_nbt
                .as_ref()
                .and_then(|nbt| nbt.get_string("final_state"))
                .unwrap_or("minecraft:air");
            placed_entry = PaletteEntry::from_string(final_state);
            block_entity_nbt = None;
        }

        // Structure void preserves the existing block, both from the palette itself and from a
        // jigsaw final state.
        if placed_entry.name == "minecraft:structure_void" {
            continue;
        }

        // Resolve block state with rotation applied to directional properties
        let Some(mut state) =
            BlockStateResolver::resolve(&placed_entry, rotation, Mirror::default())
        else {
            continue;
        };

        // Rotate block position within template bounds
        let local_pos = rotation.transform_pos(block.pos, template.size);

        let wx = world_x + local_pos.x;
        let wy = origin.y + local_pos.y;
        let wz = world_z + local_pos.z;

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

        let world_pos = Vector3::new(wx, wy, wz);

        if apply_waterlogging
            && placer.get_block_state(&world_pos).to_block_id() == pumpkin_data::Block::WATER.id
            && let Some((_, waterlogged)) = placed_entry
                .properties
                .iter_mut()
                .find(|(name, _)| name == "waterlogged")
        {
            *waterlogged = "true".to_string();
            if let Some(waterlogged_state) =
                BlockStateResolver::resolve(&placed_entry, rotation, Mirror::default())
            {
                state = waterlogged_state;
            }
        }

        // Apply processors
        let mut should_place = true;
        let mut capped_idx = 0;
        let mut rng =
            LegacyRand::from_seed(hash_block_pos(world_pos.x, world_pos.y, world_pos.z) as u64);
        for processor in processors {
            let Some(processed_state) = processor.process_with_context(
                placer,
                world_pos,
                state,
                &mut block_entity_nbt,
                &mut context,
                &mut capped_idx,
                &mut rng,
            ) else {
                should_place = false;
                break;
            };
            state = processed_state;
        }
        if !should_place {
            continue;
        }
        // Legacy pool elements ignore air after jigsaw replacement and custom processors.
        if skip_air && state.id.to_block_id() == pumpkin_data::Block::AIR.id {
            continue;
        }

        placer.set_block_state(&Vector3::new(wx, wy, wz), state);

        // Create block entities for interactive blocks (furnaces, chests, etc.)
        let final_block = pumpkin_data::Block::from_id(state.id.to_block_id());
        let block_entity_id = get_block_entity_id(final_block.name);
        if block_entity_nbt.is_some() || block_entity_id.is_some() {
            let fallback_id = block_entity_id.unwrap_or(final_block.name);
            let mut placed_nbt = NbtCompound::new();

            placed_nbt.put_string("id", fallback_id.to_string());
            placed_nbt.put_int("x", wx);
            placed_nbt.put_int("y", wy);
            placed_nbt.put_int("z", wz);

            if let Some(template_nbt) = &block_entity_nbt {
                for (key, value) in &template_nbt.child_tags {
                    if key.as_ref() != "x" && key.as_ref() != "y" && key.as_ref() != "z" {
                        placed_nbt.child_tags.insert(key.clone(), value.clone());
                    }
                }
            }

            if placed_nbt.get_string("LootTable").is_some()
                && placed_nbt.get_long("LootTableSeed").is_none()
            {
                let mut random = LegacyRand::from_seed(hash_block_pos(wx, wy, wz) as u64);
                placed_nbt.put_long("LootTableSeed", random.next_i64());
            }

            placer.add_block_entity(placed_nbt);
        }
    }
}

pub(crate) fn place_template_entities(
    chunk: &mut ProtoChunk,
    template: &StructureTemplate,
    origin: Vector3<i32>,
    rotation: Rotation,
    chunk_box: &pumpkin_util::math::block_box::BlockBox,
) {
    for entity in &template.entities {
        let block_pos = rotation.transform_pos(entity.block_pos, template.size);
        let world_block_pos = Vector3::new(
            origin.x + block_pos.x,
            origin.y + block_pos.y,
            origin.z + block_pos.z,
        );
        if !chunk_box.contains_pos(&world_block_pos) {
            continue;
        }

        let pos = match rotation {
            Rotation::None => entity.pos,
            Rotation::Clockwise90 => Vector3::new(
                f64::from(template.size.z) - entity.pos.z,
                entity.pos.y,
                entity.pos.x,
            ),
            Rotation::Rotate180 => Vector3::new(
                f64::from(template.size.x) - entity.pos.x,
                entity.pos.y,
                f64::from(template.size.z) - entity.pos.z,
            ),
            Rotation::CounterClockwise90 => Vector3::new(
                entity.pos.z,
                entity.pos.y,
                f64::from(template.size.x) - entity.pos.x,
            ),
        };
        let mut nbt = entity.nbt.clone();
        nbt.put(
            "Pos",
            NbtTag::List(vec![
                (f64::from(origin.x) + pos.x).into(),
                (f64::from(origin.y) + pos.y).into(),
                (f64::from(origin.z) + pos.z).into(),
            ]),
        );
        nbt.child_tags.remove("UUID");

        if let Some(rotation_nbt) = nbt.get_list("Rotation")
            && rotation_nbt.len() == 2
        {
            let yaw = rotation_nbt[0].extract_float().unwrap_or_default()
                + match rotation {
                    Rotation::None => 0.0,
                    Rotation::Clockwise90 => 90.0,
                    Rotation::Rotate180 => 180.0,
                    Rotation::CounterClockwise90 => 270.0,
                };
            let pitch = rotation_nbt[1].extract_float().unwrap_or_default();
            nbt.put("Rotation", NbtTag::List(vec![yaw.into(), pitch.into()]));
        }

        chunk.add_structure_entity(nbt);
    }
}

/// Returns the block entity ID for blocks that require one, or None if not needed.
pub(crate) fn get_block_entity_id(block_name: &str) -> Option<&'static str> {
    let name = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    match name {
        "furnace" => Some("minecraft:furnace"),
        "chest" => Some("minecraft:chest"),
        "trapped_chest" => Some("minecraft:trapped_chest"),
        "barrel" => Some("minecraft:barrel"),
        "hopper" => Some("minecraft:hopper"),
        "dropper" => Some("minecraft:dropper"),
        "dispenser" => Some("minecraft:dispenser"),
        "brewing_stand" => Some("minecraft:brewing_stand"),
        "blast_furnace" => Some("minecraft:blast_furnace"),
        "smoker" => Some("minecraft:smoker"),
        "shulker_box" => Some("minecraft:shulker_box"),
        "bed" => Some("minecraft:bed"),
        "suspicious_sand" | "suspicious_gravel" => Some("minecraft:brushable_block"),
        "decorated_pot" => Some("minecraft:decorated_pot"),
        "spawner" => Some("minecraft:mob_spawner"),
        "trial_spawner" => Some("minecraft:trial_spawner"),
        "vault" => Some("minecraft:vault"),
        "crafter" => Some("minecraft:crafter"),
        "creaking_heart" => Some("minecraft:creaking_heart"),
        "chiseled_bookshelf" => Some("minecraft:chiseled_bookshelf"),
        "beehive" | "bee_nest" => Some("minecraft:beehive"),
        "campfire" | "soul_campfire" => Some("minecraft:campfire"),
        "sculk_sensor" | "calibrated_sculk_sensor" => Some("minecraft:sculk_sensor"),
        "sculk_catalyst" => Some("minecraft:sculk_catalyst"),
        "sculk_shrieker" => Some("minecraft:sculk_shrieker"),
        "dragon_head"
        | "dragon_wall_head"
        | "skeleton_skull"
        | "skeleton_wall_skull"
        | "wither_skeleton_skull"
        | "wither_skeleton_wall_skull"
        | "zombie_head"
        | "zombie_wall_head"
        | "player_head"
        | "player_wall_head"
        | "creeper_head"
        | "creeper_wall_head"
        | "piglin_head"
        | "piglin_wall_head" => Some("minecraft:skull"),
        "sign" | "oak_sign" | "spruce_sign" | "birch_sign" | "jungle_sign" | "acacia_sign"
        | "dark_oak_sign" | "mangrove_sign" | "cherry_sign" | "bamboo_sign" | "crimson_sign"
        | "warped_sign" | "pale_oak_sign" => Some("minecraft:sign"),
        "hanging_sign"
        | "oak_hanging_sign"
        | "spruce_hanging_sign"
        | "birch_hanging_sign"
        | "jungle_hanging_sign"
        | "acacia_hanging_sign"
        | "dark_oak_hanging_sign"
        | "mangrove_hanging_sign"
        | "cherry_hanging_sign"
        | "bamboo_hanging_sign"
        | "crimson_hanging_sign"
        | "warped_hanging_sign"
        | "pale_oak_hanging_sign" => Some("minecraft:hanging_sign"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::Block;

    struct CollectingPlacer(Vec<BlockStateId>);

    impl BlockPlacer for CollectingPlacer {
        fn get_block_state(&self, _pos: &Vector3<i32>) -> BlockStateId {
            Block::AIR.default_state.id
        }

        fn set_block_state(&mut self, _pos: &Vector3<i32>, state: &BlockState) {
            self.0.push(state.id);
        }

        fn add_block_entity(&mut self, _nbt: NbtCompound) {}
    }

    #[test]
    fn structure_void_is_never_placed() {
        let mut placed_templates = 0;

        for name in all_template_names() {
            let Some(template) = get_template(name) else {
                continue;
            };

            let mut placer = CollectingPlacer(Vec::new());
            place_template(
                &mut placer,
                &template,
                Vector3::new(0, 0, 0),
                (0, 0),
                Rotation::None,
                false,
                false,
                &[],
                None,
            );

            for state_id in &placer.0 {
                assert_ne!(
                    state_id.to_block_id(),
                    Block::STRUCTURE_VOID.id,
                    "{name} placed a structure void"
                );
            }

            placed_templates += 1;
        }

        assert!(placed_templates > 0);
    }
}
