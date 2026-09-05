#![allow(
    clippy::too_many_lines,
    clippy::collapsible_if,
    clippy::unnecessary_wraps,
    clippy::significant_drop_in_scrutinee,
    clippy::nonminimal_bool,
    clippy::if_not_else,
    clippy::explicit_auto_deref,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::match_same_arms,
    clippy::struct_excessive_bools
)]

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::block::BlockArgumentType;
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::coordinates::rotation::RotationArgumentType;
use crate::command::argument_types::coordinates::swizzle::SwizzleArgumentType;
use crate::command::argument_types::coordinates::vec3::Vec3ArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::entity_anchor::EntityAnchorArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::argument_types::nbt_path::{NbtPath, NbtPathArgumentType};
use crate::command::argument_types::objective::ObjectiveArgumentType;
use crate::command::argument_types::range::{FloatRangeArgumentType, IntRangeArgumentType};
use crate::command::argument_types::resource::{ENTITY_TYPE_ARGUMENT, ResourceArgument};
use crate::command::argument_types::resource_key::{BIOME_REGISTRY, ResourceKeyArgument};
use crate::command::argument_types::resource_or_tag::{ResourceOrTag, ResourceOrTagArgument};
use crate::command::commands::data::{
    BlockDataAccessor, DataAccessor, EntityDataAccessor, StorageDataAccessor,
};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::attached::{CommandNodeId, NodeId};
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::tree::Tree;
use crate::command::node::{RedirectModifier, Redirection};
use crate::entity::EntityBase;
use crate::entity::r#type::from_type;
use crate::world::stopwatches::Stopwatches;
use pumpkin_data::biome::Biome;
use pumpkin_data::tag::{self, RegistryKey};
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::PermissionLvl;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use rustc_hash::FxHashSet;
use std::sync::Arc;
use uuid::Uuid;

const DESCRIPTION: &str = "Execute a command with a modified context.";
const PERMISSION: &str = "minecraft:command.execute";

static ERROR_INVALID_DIMENSION: CommandErrorType<1> =
    CommandErrorType::new("argument.dimension.invalid", "argument.dimension.invalid");

static DIMENSION_REGISTRY: &Identifier = &Identifier::vanilla_static("dimension");

fn execute_as_modifier(context: &CommandContext) -> crate::command::node::RedirectModifierResult {
    let targets = EntityArgumentType::get_optional_entities(context, "targets")?;
    let mut sources = Vec::new();
    for target in targets {
        let mut source = context.source.as_ref().clone();
        let display_name = target.get_display_name();
        let name = target.get_name().get_text();
        source.entity = Some(target.clone());
        source.name = name;
        source.display_name = display_name;
        sources.push(Arc::new(source));
    }
    Ok(sources)
}

fn execute_at_modifier(context: &CommandContext) -> crate::command::node::RedirectModifierResult {
    let targets = EntityArgumentType::get_optional_entities(context, "targets")?;
    let mut sources = Vec::new();
    for target in targets {
        let entity = target.get_entity();
        let mut source = context.source.as_ref().clone();
        source.position = entity.pos.load();
        source.rotation = Vector2::new(entity.yaw.load(), entity.pitch.load());
        source.world = Some(entity.world.load().clone());
        sources.push(Arc::new(source));
    }
    Ok(sources)
}

fn execute_in_modifier(context: &CommandContext) -> crate::command::node::RedirectModifierResult {
    let dimension_key = ResourceKeyArgument::get_registry_key(
        context,
        "dimension",
        &Identifier::vanilla_static("dimension"),
        &ERROR_INVALID_DIMENSION,
    )?;
    let dimension_name = dimension_key.identifier.to_string();
    let server = context.server();
    let worlds = server.worlds.load();
    let target_world = worlds
        .iter()
        .find(|w| w.dimension.minecraft_name == dimension_name);

    target_world.map_or_else(
        || Err(ERROR_INVALID_DIMENSION.create_without_context(TextComponent::text(dimension_name))),
        |target_world| {
            let mut source = context.source.as_ref().clone();
            source.world = Some(target_world.clone());
            Ok(vec![Arc::new(source)])
        },
    )
}

fn execute_positioned_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
    let mut source = context.source.as_ref().clone();
    source.position = pos;
    Ok(vec![Arc::new(source)])
}

fn execute_positioned_as_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let targets = EntityArgumentType::get_optional_entities(context, "targets")?;
    let mut sources = Vec::new();
    for target in targets {
        let mut source = context.source.as_ref().clone();
        source.position = target.get_entity().pos.load();
        sources.push(Arc::new(source));
    }
    Ok(sources)
}

fn execute_positioned_over_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let mut source = context.source.as_ref().clone();
    let pos = source.position;
    let block_x = pos.x.floor() as i32;
    let block_z = pos.z.floor() as i32;

    if let Some(ref world) = source.world {
        let top_y = world.get_top_block(Vector2::new(block_x, block_z));
        source.position.y = (top_y + 1) as f64;
    }
    Ok(vec![Arc::new(source)])
}

fn execute_rotated_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let rot_coords = RotationArgumentType::get(context, "rotation")?;
    let rot = rot_coords.rotation(&context.source);
    let mut source = context.source.as_ref().clone();
    source.rotation = rot;
    Ok(vec![Arc::new(source)])
}

fn execute_rotated_as_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let targets = EntityArgumentType::get_optional_entities(context, "targets")?;
    let mut sources = Vec::new();
    for target in targets {
        let entity = target.get_entity();
        let mut source = context.source.as_ref().clone();
        source.rotation = Vector2::new(entity.yaw.load(), entity.pitch.load());
        sources.push(Arc::new(source));
    }
    Ok(sources)
}

fn execute_if_entity_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let targets = EntityArgumentType::get_optional_entities(context, "targets")?;
    if targets.is_empty() {
        Ok(vec![])
    } else {
        Ok(vec![context.source.clone()])
    }
}

fn execute_unless_entity_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let targets = EntityArgumentType::get_optional_entities(context, "targets")?;
    if targets.is_empty() {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn execute_align_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let swizzle = SwizzleArgumentType::get(context, "axes")?;
    let mut source = context.source.as_ref().clone();

    source.position = swizzle.align(source.position);

    Ok(vec![Arc::new(source)])
}

fn execute_anchored_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let anchor = EntityAnchorArgumentType::get(context, "anchor")?;
    let mut source = context.source.as_ref().clone();
    source.entity_anchor = anchor;
    Ok(vec![Arc::new(source)])
}

fn execute_facing_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
    let mut source = context.source.as_ref().clone();

    let dx = pos.x - source.position.x;
    let dy = pos.y - source.position.y;
    let dz = pos.z - source.position.z;

    let xz_dist = dx.hypot(dz);
    let yaw = (dz.atan2(dx).to_degrees() as f32) - 90.0;
    let pitch = -(dy.atan2(xz_dist).to_degrees() as f32);

    source.rotation = Vector2::new(yaw, pitch);
    Ok(vec![Arc::new(source)])
}

fn execute_facing_entity_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let targets = EntityArgumentType::get_optional_entities(context, "targets")?;
    let anchor = EntityAnchorArgumentType::get(context, "anchor")?;
    let mut sources = Vec::new();

    for target in targets {
        let target_pos = anchor.position_at_entity(target.get_entity());
        let mut source = context.source.as_ref().clone();

        let dx = target_pos.x - source.position.x;
        let dy = target_pos.y - source.position.y;
        let dz = target_pos.z - source.position.z;

        let xz_dist = dx.hypot(dz);
        let yaw = (dz.atan2(dx).to_degrees() as f32) - 90.0;
        let pitch = -(dy.atan2(xz_dist).to_degrees() as f32);

        source.rotation = Vector2::new(yaw, pitch);
        sources.push(Arc::new(source));
    }
    Ok(sources)
}

fn execute_if_block_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
    let expected_block = BlockArgumentType::get(context, "block")?;

    if let Some(ref world) = context.source.world {
        let block = world.get_block(&pos);
        if block == expected_block {
            return Ok(vec![context.source.clone()]);
        }
    }
    Ok(vec![])
}

fn execute_unless_block_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
    let expected_block = BlockArgumentType::get(context, "block")?;

    if let Some(ref world) = context.source.world {
        let block = world.get_block(&pos);
        if block != expected_block {
            return Ok(vec![context.source.clone()]);
        }
    } else {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn execute_if_loaded_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;

    if let Some(ref world) = context.source.world
        && world.is_loaded(&pos)
    {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn execute_unless_loaded_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;

    if let Some(ref world) = context.source.world {
        if !world.is_loaded(&pos) {
            return Ok(vec![context.source.clone()]);
        }
    } else {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn execute_if_dimension_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let dimension_key = ResourceKeyArgument::get_registry_key(
        context,
        "dimension",
        &Identifier::vanilla_static("dimension"),
        &ERROR_INVALID_DIMENSION,
    )?;
    let dimension_name = dimension_key.identifier.to_string();

    if let Some(ref world) = context.source.world
        && world.dimension.minecraft_name == dimension_name
    {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn execute_unless_dimension_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let dimension_key = ResourceKeyArgument::get_registry_key(
        context,
        "dimension",
        &Identifier::vanilla_static("dimension"),
        &ERROR_INVALID_DIMENSION,
    )?;
    let dimension_name = dimension_key.identifier.to_string();

    if let Some(ref world) = context.source.world {
        if world.dimension.minecraft_name != dimension_name {
            return Ok(vec![context.source.clone()]);
        }
    } else {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn execute_if_biome_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
    let biome_arg = context.get_argument::<ResourceOrTag>("biome")?.clone();

    if let Some(ref world) = context.source.world {
        let biome = world.get_biome(&pos);
        let targets: FxHashSet<u8> = match &biome_arg {
            ResourceOrTag::Resource(id) => Biome::from_name(id.path())
                .map(|b| b.id)
                .into_iter()
                .collect(),
            ResourceOrTag::Tag(id) => {
                tag::get_tag_values(RegistryKey::WorldgenBiome, &id.to_string())
                    .into_iter()
                    .flatten()
                    .filter_map(|name| Biome::from_name(*name))
                    .map(|b| b.id)
                    .collect()
            }
        };

        if targets.contains(&biome.id) {
            return Ok(vec![context.source.clone()]);
        }
    }
    Ok(vec![])
}

fn execute_unless_biome_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
    let biome_arg = context.get_argument::<ResourceOrTag>("biome")?.clone();

    if let Some(ref world) = context.source.world {
        let biome = world.get_biome(&pos);
        let targets: FxHashSet<u8> = match &biome_arg {
            ResourceOrTag::Resource(id) => Biome::from_name(id.path())
                .map(|b| b.id)
                .into_iter()
                .collect(),
            ResourceOrTag::Tag(id) => {
                tag::get_tag_values(RegistryKey::WorldgenBiome, &id.to_string())
                    .into_iter()
                    .flatten()
                    .filter_map(|name| Biome::from_name(*name))
                    .map(|b| b.id)
                    .collect()
            }
        };

        if !targets.contains(&biome.id) {
            return Ok(vec![context.source.clone()]);
        }
    } else {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn get_score(context: &CommandContext, target: &str, objective: &str) -> Option<i32> {
    let world = context.source.world.as_ref()?;
    let scoreboard = world
        .scoreboard
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    scoreboard.get_score(target, objective).map(|s| s.value.0)
}

fn execute_if_score_matches_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let range = IntRangeArgumentType::get(context, "range")?;

    if let Some(score) = get_score(context, target, target_obj) {
        if range.matches(score) {
            return Ok(vec![context.source.clone()]);
        }
    }
    Ok(vec![])
}

fn execute_unless_score_matches_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let range = IntRangeArgumentType::get(context, "range")?;

    if let Some(score) = get_score(context, target, target_obj) {
        if !range.matches(score) {
            return Ok(vec![context.source.clone()]);
        }
    } else {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn execute_if_score_eq_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let source = StringArgumentType::get(context, "source")?;
    let source_obj = ObjectiveArgumentType::get(context, "source_obj")?;

    if let (Some(a), Some(b)) = (
        get_score(context, target, target_obj),
        get_score(context, source, source_obj),
    ) {
        if a == b {
            return Ok(vec![context.source.clone()]);
        }
    }
    Ok(vec![])
}

fn execute_if_score_lt_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let source = StringArgumentType::get(context, "source")?;
    let source_obj = ObjectiveArgumentType::get(context, "source_obj")?;

    if let (Some(a), Some(b)) = (
        get_score(context, target, target_obj),
        get_score(context, source, source_obj),
    ) {
        if a < b {
            return Ok(vec![context.source.clone()]);
        }
    }
    Ok(vec![])
}

fn execute_if_score_le_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let source = StringArgumentType::get(context, "source")?;
    let source_obj = ObjectiveArgumentType::get(context, "source_obj")?;

    if let (Some(a), Some(b)) = (
        get_score(context, target, target_obj),
        get_score(context, source, source_obj),
    ) {
        if a <= b {
            return Ok(vec![context.source.clone()]);
        }
    }
    Ok(vec![])
}

fn execute_if_score_gt_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let source = StringArgumentType::get(context, "source")?;
    let source_obj = ObjectiveArgumentType::get(context, "source_obj")?;

    if let (Some(a), Some(b)) = (
        get_score(context, target, target_obj),
        get_score(context, source, source_obj),
    ) {
        if a > b {
            return Ok(vec![context.source.clone()]);
        }
    }
    Ok(vec![])
}

fn execute_if_score_ge_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let source = StringArgumentType::get(context, "source")?;
    let source_obj = ObjectiveArgumentType::get(context, "source_obj")?;

    if let (Some(a), Some(b)) = (
        get_score(context, target, target_obj),
        get_score(context, source, source_obj),
    ) {
        if a >= b {
            return Ok(vec![context.source.clone()]);
        }
    }
    Ok(vec![])
}

fn execute_unless_score_eq_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let source = StringArgumentType::get(context, "source")?;
    let source_obj = ObjectiveArgumentType::get(context, "source_obj")?;

    if let (Some(a), Some(b)) = (
        get_score(context, target, target_obj),
        get_score(context, source, source_obj),
    ) {
        if a != b {
            return Ok(vec![context.source.clone()]);
        }
    } else {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn execute_unless_score_lt_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let source = StringArgumentType::get(context, "source")?;
    let source_obj = ObjectiveArgumentType::get(context, "source_obj")?;

    if let (Some(a), Some(b)) = (
        get_score(context, target, target_obj),
        get_score(context, source, source_obj),
    ) {
        if a >= b {
            return Ok(vec![context.source.clone()]);
        }
    } else {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn execute_unless_score_le_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let source = StringArgumentType::get(context, "source")?;
    let source_obj = ObjectiveArgumentType::get(context, "source_obj")?;

    if let (Some(a), Some(b)) = (
        get_score(context, target, target_obj),
        get_score(context, source, source_obj),
    ) {
        if a > b {
            return Ok(vec![context.source.clone()]);
        }
    } else {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn execute_unless_score_gt_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let source = StringArgumentType::get(context, "source")?;
    let source_obj = ObjectiveArgumentType::get(context, "source_obj")?;

    if let (Some(a), Some(b)) = (
        get_score(context, target, target_obj),
        get_score(context, source, source_obj),
    ) {
        if a <= b {
            return Ok(vec![context.source.clone()]);
        }
    } else {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn execute_unless_score_ge_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let target = StringArgumentType::get(context, "target")?;
    let target_obj = ObjectiveArgumentType::get(context, "target_obj")?;
    let source = StringArgumentType::get(context, "source")?;
    let source_obj = ObjectiveArgumentType::get(context, "source_obj")?;

    if let (Some(a), Some(b)) = (
        get_score(context, target, target_obj),
        get_score(context, source, source_obj),
    ) {
        if a < b {
            return Ok(vec![context.source.clone()]);
        }
    } else {
        return Ok(vec![context.source.clone()]);
    }
    Ok(vec![])
}

fn check_blocks_match(context: &CommandContext, masked: bool) -> Result<bool, CommandSyntaxError> {
    let start = BlockPosArgumentType::get_block_pos(context, "start")?;
    let end = BlockPosArgumentType::get_block_pos(context, "end")?;
    let destination = BlockPosArgumentType::get_block_pos(context, "destination")?;

    if let Some(ref world) = context.source.world {
        let min_x = start.0.x.min(end.0.x);
        let max_x = start.0.x.max(end.0.x);
        let min_y = start.0.y.min(end.0.y);
        let max_y = start.0.y.max(end.0.y);
        let min_z = start.0.z.min(end.0.z);
        let max_z = start.0.z.max(end.0.z);

        let dx = destination.0.x - min_x;
        let dy = destination.0.y - min_y;
        let dz = destination.0.z - min_z;

        for y in min_y..=max_y {
            for z in min_z..=max_z {
                for x in min_x..=max_x {
                    let src_pos = BlockPos::new(x, y, z);
                    let dst_pos = BlockPos::new(x + dx, y + dy, z + dz);
                    let src_block = world.get_block(&src_pos);
                    let dst_block = world.get_block(&dst_pos);

                    if masked && src_block.is_air() {
                        continue;
                    }
                    if src_block != dst_block {
                        return Ok(false);
                    }
                }
            }
        }
        return Ok(true);
    }
    Ok(false)
}

fn execute_if_blocks_all_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if check_blocks_match(context, false)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn execute_if_blocks_masked_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if check_blocks_match(context, true)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn execute_unless_blocks_all_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if !check_blocks_match(context, false)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn execute_unless_blocks_masked_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if !check_blocks_match(context, true)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn check_data_matches(
    context: &CommandContext,
    is_block: bool,
    is_entity: bool,
) -> Result<bool, CommandSyntaxError> {
    let path = context.get_argument::<NbtPath>("path")?;
    let tags = if is_block {
        let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
        let world = context.source.world().clone();
        let accessor = BlockDataAccessor::new(pos, world)?;
        let data = accessor.get_data()?;
        path.get(&NbtTag::Compound(data))?
    } else if is_entity {
        let entity = EntityArgumentType::get_entity(context, "target")?;
        let accessor = EntityDataAccessor::new(entity);
        let data = accessor.get_data()?;
        path.get(&NbtTag::Compound(data))?
    } else {
        let id = context.get_argument::<Identifier>("id")?;
        let server = context.server().clone();
        let accessor = StorageDataAccessor::new(id.to_string(), server);
        let data = accessor.get_data()?;
        path.get(&NbtTag::Compound(data))?
    };
    Ok(!tags.is_empty())
}

fn execute_if_data_block_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if check_data_matches(context, true, false)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn execute_if_data_entity_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if check_data_matches(context, false, true)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn execute_if_data_storage_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if check_data_matches(context, false, false)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn execute_unless_data_block_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if !check_data_matches(context, true, false)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn execute_unless_data_entity_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if !check_data_matches(context, false, true)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn execute_unless_data_storage_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if !check_data_matches(context, false, false)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn check_stopwatch_matches(context: &CommandContext) -> Result<bool, CommandSyntaxError> {
    let id = context.get_argument::<Identifier>("id")?;
    let range = FloatRangeArgumentType::get(context, "range")?;
    let stopwatches = context
        .server()
        .stopwatches
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let id_str = id.to_string();
    let Some(stopwatch) = stopwatches.get(&id_str) else {
        return Err(crate::command::commands::stopwatch::ERROR_DOES_NOT_EXIST
            .create_without_context(TextComponent::text(id_str)));
    };
    let now = Stopwatches::current_time();
    let elapsed = stopwatch.elapsed_seconds(now);
    Ok(range.matches(elapsed))
}

fn execute_if_stopwatch_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if check_stopwatch_matches(context)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn execute_unless_stopwatch_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    if !check_stopwatch_matches(context)? {
        Ok(vec![context.source.clone()])
    } else {
        Ok(vec![])
    }
}

fn execute_summon_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let entity_type = ResourceArgument::get_summonable_entity_type(context, "entity_type")?;
    let entity = from_type(
        entity_type,
        context.source.position,
        context.source.world(),
        Uuid::new_v4(),
    );
    context.source.world().spawn_entity(entity.clone());
    let mut source = context.source.as_ref().clone();
    source.entity = Some(entity);
    Ok(vec![Arc::new(source)])
}

fn execute_on_owner_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let mut sources = Vec::new();
    if let Some(ref entity) = context.source.entity {
        let world = context.source.world();
        if let Some(mob) = entity.get_mob() {
            if let Some(owner_uuid) = mob.get_owner_uuid() {
                if let Some(owner) = world.get_player_by_uuid(owner_uuid) {
                    let mut source = context.source.as_ref().clone();
                    let name = owner.get_name().get_text();
                    let display_name = owner.get_display_name();
                    source.entity = Some(owner);
                    source.name = name;
                    source.display_name = display_name;
                    sources.push(Arc::new(source));
                }
            }
        }
    }
    Ok(sources)
}

fn execute_on_leasher_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let mut sources = Vec::new();
    if let Some(ref entity) = context.source.entity {
        if let Some(leasher) = entity
            .get_entity()
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        {
            let mut source = context.source.as_ref().clone();
            let name = leasher.get_name().get_text();
            let display_name = leasher.get_display_name();
            source.entity = Some(leasher);
            source.name = name;
            source.display_name = display_name;
            sources.push(Arc::new(source));
        }
    }
    Ok(sources)
}

fn execute_on_target_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let mut sources = Vec::new();
    if let Some(ref entity) = context.source.entity {
        if let Some(mob) = entity.get_mob() {
            if let Some(target) = mob.get_mob_entity().get_target() {
                let mut source = context.source.as_ref().clone();
                let name = target.get_name().get_text();
                let display_name = target.get_display_name();
                source.entity = Some(target);
                source.name = name;
                source.display_name = display_name;
                sources.push(Arc::new(source));
            }
        }
    }
    Ok(sources)
}

fn execute_on_attacker_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let mut sources = Vec::new();
    if let Some(ref entity) = context.source.entity {
        if let Some(living) = entity.get_living_entity() {
            let attacker_id = living
                .last_attacker_id
                .load(std::sync::atomic::Ordering::Relaxed);
            if attacker_id != 0 {
                let world = context.source.world();
                if let Some(attacker) = world.get_entity_by_id(attacker_id) {
                    let mut source = context.source.as_ref().clone();
                    let name = attacker.get_name().get_text();
                    let display_name = attacker.get_display_name();
                    source.entity = Some(attacker);
                    source.name = name;
                    source.display_name = display_name;
                    sources.push(Arc::new(source));
                }
            }
        }
    }
    Ok(sources)
}

fn execute_on_vehicle_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let mut sources = Vec::new();
    if let Some(ref entity) = context.source.entity {
        if let Some(vehicle) = entity
            .get_entity()
            .vehicle
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        {
            let mut source = context.source.as_ref().clone();
            let name = vehicle.get_name().get_text();
            let display_name = vehicle.get_display_name();
            source.entity = Some(vehicle);
            source.name = name;
            source.display_name = display_name;
            sources.push(Arc::new(source));
        }
    }
    Ok(sources)
}

fn execute_on_controller_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let mut sources = Vec::new();
    if let Some(ref entity) = context.source.entity {
        if let Some(controller) = entity
            .get_entity()
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first()
            .cloned()
        {
            let mut source = context.source.as_ref().clone();
            let name = controller.get_name().get_text();
            let display_name = controller.get_display_name();
            source.entity = Some(controller);
            source.name = name;
            source.display_name = display_name;
            sources.push(Arc::new(source));
        }
    }
    Ok(sources)
}

fn execute_on_origin_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    execute_on_owner_modifier(context)
}

fn execute_on_passengers_modifier(
    context: &CommandContext,
) -> crate::command::node::RedirectModifierResult {
    let mut sources = Vec::new();
    if let Some(ref entity) = context.source.entity {
        let passengers = entity
            .get_entity()
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for passenger in passengers {
            let mut source = context.source.as_ref().clone();
            let name = passenger.get_name().get_text();
            let display_name = passenger.get_display_name();
            source.entity = Some(passenger);
            source.name = name;
            source.display_name = display_name;
            sources.push(Arc::new(source));
        }
    }
    Ok(sources)
}

#[allow(clippy::too_many_lines)]
pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let builder = command("execute", DESCRIPTION)
        .requires(PERMISSION)
        .then(literal("run").redirect(Redirection::Root))
        .then(
            literal("as").then(argument("targets", EntityArgumentType::Entities).fork(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_as_modifier)),
            )),
        )
        .then(
            literal("at").then(argument("targets", EntityArgumentType::Entities).fork(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_at_modifier)),
            )),
        )
        .then(literal("in").then(
            argument("dimension", ResourceKeyArgument(DIMENSION_REGISTRY)).redirect_with_modifier(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_in_modifier)),
            ),
        ))
        .then(
            literal("positioned")
                .then(
                    literal("as").then(argument("targets", EntityArgumentType::Entities).fork(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_positioned_as_modifier)),
                    )),
                )
                .then(literal("over").then(
                    argument("heightmap", StringArgumentType::SingleWord).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_positioned_over_modifier)),
                    ),
                ))
                .then(
                    argument("pos", Vec3ArgumentType::Default).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_positioned_modifier)),
                    ),
                ),
        )
        .then(
            literal("rotated")
                .then(
                    literal("as").then(argument("targets", EntityArgumentType::Entities).fork(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_rotated_as_modifier)),
                    )),
                )
                .then(
                    argument("rotation", RotationArgumentType).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_rotated_modifier)),
                    ),
                ),
        )
        .then(literal("align").then(
            argument("axes", SwizzleArgumentType).redirect_with_modifier(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_align_modifier)),
            ),
        ))
        .then(literal("anchored").then(
            argument("anchor", EntityAnchorArgumentType).redirect_with_modifier(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_anchored_modifier)),
            ),
        ))
        .then(
            literal("facing")
                .then(literal("entity").then(
                    argument("targets", EntityArgumentType::Entities).then(
                        argument("anchor", EntityAnchorArgumentType).fork(
                            Redirection::Root,
                            RedirectModifier::Custom(Arc::new(execute_facing_entity_modifier)),
                        ),
                    ),
                ))
                .then(
                    argument("pos", Vec3ArgumentType::Default).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_facing_modifier)),
                    ),
                ),
        )
        .then(literal("summon").then(
            argument("entity_type", ENTITY_TYPE_ARGUMENT.clone()).redirect_with_modifier(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_summon_modifier)),
            ),
        ))
        .then(
            literal("on")
                .then(literal("owner").fork(
                    Redirection::Root,
                    RedirectModifier::Custom(Arc::new(execute_on_owner_modifier)),
                ))
                .then(literal("leasher").fork(
                    Redirection::Root,
                    RedirectModifier::Custom(Arc::new(execute_on_leasher_modifier)),
                ))
                .then(literal("target").fork(
                    Redirection::Root,
                    RedirectModifier::Custom(Arc::new(execute_on_target_modifier)),
                ))
                .then(literal("attacker").fork(
                    Redirection::Root,
                    RedirectModifier::Custom(Arc::new(execute_on_attacker_modifier)),
                ))
                .then(literal("vehicle").fork(
                    Redirection::Root,
                    RedirectModifier::Custom(Arc::new(execute_on_vehicle_modifier)),
                ))
                .then(literal("controller").fork(
                    Redirection::Root,
                    RedirectModifier::Custom(Arc::new(execute_on_controller_modifier)),
                ))
                .then(literal("origin").fork(
                    Redirection::Root,
                    RedirectModifier::Custom(Arc::new(execute_on_origin_modifier)),
                ))
                .then(literal("passengers").fork(
                    Redirection::Root,
                    RedirectModifier::Custom(Arc::new(execute_on_passengers_modifier)),
                )),
        )
        .then(
            literal("if")
                .then(literal("entity").then(
                    argument("targets", EntityArgumentType::Entities).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_if_entity_modifier)),
                    ),
                ))
                .then(
                    literal("block").then(argument("pos", BlockPosArgumentType).then(
                        argument("block", BlockArgumentType).redirect_with_modifier(
                            Redirection::Root,
                            RedirectModifier::Custom(Arc::new(execute_if_block_modifier)),
                        ),
                    )),
                )
                .then(literal("loaded").then(
                    argument("pos", BlockPosArgumentType).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_if_loaded_modifier)),
                    ),
                ))
                .then(
                    literal("dimension").then(
                        argument("dimension", ResourceKeyArgument(DIMENSION_REGISTRY))
                            .redirect_with_modifier(
                                Redirection::Root,
                                RedirectModifier::Custom(Arc::new(execute_if_dimension_modifier)),
                            ),
                    ),
                )
                .then(
                    literal("biome").then(
                        argument("pos", BlockPosArgumentType).then(
                            argument("biome", ResourceOrTagArgument(BIOME_REGISTRY.clone()))
                                .redirect_with_modifier(
                                    Redirection::Root,
                                    RedirectModifier::Custom(Arc::new(execute_if_biome_modifier)),
                                ),
                        ),
                    ),
                )
                .then(
                    literal("blocks").then(
                        argument("start", BlockPosArgumentType).then(
                            argument("end", BlockPosArgumentType).then(
                                argument("destination", BlockPosArgumentType)
                                    .then(literal("all").redirect_with_modifier(
                                        Redirection::Root,
                                        RedirectModifier::Custom(Arc::new(
                                            execute_if_blocks_all_modifier,
                                        )),
                                    ))
                                    .then(literal("masked").redirect_with_modifier(
                                        Redirection::Root,
                                        RedirectModifier::Custom(Arc::new(
                                            execute_if_blocks_masked_modifier,
                                        )),
                                    )),
                            ),
                        ),
                    ),
                )
                .then(
                    literal("score").then(
                        argument("target", StringArgumentType::SingleWord).then(
                            argument("target_obj", ObjectiveArgumentType)
                                .then(literal("matches").then(
                                    argument("range", IntRangeArgumentType).redirect_with_modifier(
                                        Redirection::Root,
                                        RedirectModifier::Custom(Arc::new(
                                            execute_if_score_matches_modifier,
                                        )),
                                    ),
                                ))
                                .then(
                                    literal("=").then(
                                        argument("source", StringArgumentType::SingleWord).then(
                                            argument("source_obj", ObjectiveArgumentType)
                                                .redirect_with_modifier(
                                                    Redirection::Root,
                                                    RedirectModifier::Custom(Arc::new(
                                                        execute_if_score_eq_modifier,
                                                    )),
                                                ),
                                        ),
                                    ),
                                )
                                .then(
                                    literal("<").then(
                                        argument("source", StringArgumentType::SingleWord).then(
                                            argument("source_obj", ObjectiveArgumentType)
                                                .redirect_with_modifier(
                                                    Redirection::Root,
                                                    RedirectModifier::Custom(Arc::new(
                                                        execute_if_score_lt_modifier,
                                                    )),
                                                ),
                                        ),
                                    ),
                                )
                                .then(
                                    literal("<=").then(
                                        argument("source", StringArgumentType::SingleWord).then(
                                            argument("source_obj", ObjectiveArgumentType)
                                                .redirect_with_modifier(
                                                    Redirection::Root,
                                                    RedirectModifier::Custom(Arc::new(
                                                        execute_if_score_le_modifier,
                                                    )),
                                                ),
                                        ),
                                    ),
                                )
                                .then(
                                    literal(">").then(
                                        argument("source", StringArgumentType::SingleWord).then(
                                            argument("source_obj", ObjectiveArgumentType)
                                                .redirect_with_modifier(
                                                    Redirection::Root,
                                                    RedirectModifier::Custom(Arc::new(
                                                        execute_if_score_gt_modifier,
                                                    )),
                                                ),
                                        ),
                                    ),
                                )
                                .then(
                                    literal(">=").then(
                                        argument("source", StringArgumentType::SingleWord).then(
                                            argument("source_obj", ObjectiveArgumentType)
                                                .redirect_with_modifier(
                                                    Redirection::Root,
                                                    RedirectModifier::Custom(Arc::new(
                                                        execute_if_score_ge_modifier,
                                                    )),
                                                ),
                                        ),
                                    ),
                                ),
                        ),
                    ),
                )
                .then(
                    literal("data")
                        .then(
                            literal("block").then(argument("pos", BlockPosArgumentType).then(
                                argument("path", NbtPathArgumentType).redirect_with_modifier(
                                    Redirection::Root,
                                    RedirectModifier::Custom(Arc::new(
                                        execute_if_data_block_modifier,
                                    )),
                                ),
                            )),
                        )
                        .then(literal("entity").then(
                            argument("target", EntityArgumentType::Entity).then(
                                argument("path", NbtPathArgumentType).redirect_with_modifier(
                                    Redirection::Root,
                                    RedirectModifier::Custom(Arc::new(
                                        execute_if_data_entity_modifier,
                                    )),
                                ),
                            ),
                        ))
                        .then(literal("storage").then(
                            argument("id", IdentifierArgumentType).then(
                                argument("path", NbtPathArgumentType).redirect_with_modifier(
                                    Redirection::Root,
                                    RedirectModifier::Custom(Arc::new(
                                        execute_if_data_storage_modifier,
                                    )),
                                ),
                            ),
                        )),
                )
                .then(
                    literal("stopwatch").then(argument("id", IdentifierArgumentType).then(
                        argument("range", FloatRangeArgumentType).redirect_with_modifier(
                            Redirection::Root,
                            RedirectModifier::Custom(Arc::new(execute_if_stopwatch_modifier)),
                        ),
                    )),
                ),
        )
        .then(
            literal("unless")
                .then(literal("entity").then(
                    argument("targets", EntityArgumentType::Entities).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_unless_entity_modifier)),
                    ),
                ))
                .then(
                    literal("block").then(argument("pos", BlockPosArgumentType).then(
                        argument("block", BlockArgumentType).redirect_with_modifier(
                            Redirection::Root,
                            RedirectModifier::Custom(Arc::new(execute_unless_block_modifier)),
                        ),
                    )),
                )
                .then(literal("loaded").then(
                    argument("pos", BlockPosArgumentType).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_unless_loaded_modifier)),
                    ),
                ))
                .then(
                    literal("dimension").then(
                        argument("dimension", ResourceKeyArgument(DIMENSION_REGISTRY))
                            .redirect_with_modifier(
                                Redirection::Root,
                                RedirectModifier::Custom(Arc::new(
                                    execute_unless_dimension_modifier,
                                )),
                            ),
                    ),
                )
                .then(
                    literal("biome").then(
                        argument("pos", BlockPosArgumentType).then(
                            argument("biome", ResourceOrTagArgument(BIOME_REGISTRY.clone()))
                                .redirect_with_modifier(
                                    Redirection::Root,
                                    RedirectModifier::Custom(Arc::new(
                                        execute_unless_biome_modifier,
                                    )),
                                ),
                        ),
                    ),
                )
                .then(
                    literal("blocks").then(
                        argument("start", BlockPosArgumentType).then(
                            argument("end", BlockPosArgumentType).then(
                                argument("destination", BlockPosArgumentType)
                                    .then(literal("all").redirect_with_modifier(
                                        Redirection::Root,
                                        RedirectModifier::Custom(Arc::new(
                                            execute_unless_blocks_all_modifier,
                                        )),
                                    ))
                                    .then(literal("masked").redirect_with_modifier(
                                        Redirection::Root,
                                        RedirectModifier::Custom(Arc::new(
                                            execute_unless_blocks_masked_modifier,
                                        )),
                                    )),
                            ),
                        ),
                    ),
                )
                .then(
                    literal("score").then(
                        argument("target", StringArgumentType::SingleWord).then(
                            argument("target_obj", ObjectiveArgumentType)
                                .then(literal("matches").then(
                                    argument("range", IntRangeArgumentType).redirect_with_modifier(
                                        Redirection::Root,
                                        RedirectModifier::Custom(Arc::new(
                                            execute_unless_score_matches_modifier,
                                        )),
                                    ),
                                ))
                                .then(
                                    literal("=").then(
                                        argument("source", StringArgumentType::SingleWord).then(
                                            argument("source_obj", ObjectiveArgumentType)
                                                .redirect_with_modifier(
                                                    Redirection::Root,
                                                    RedirectModifier::Custom(Arc::new(
                                                        execute_unless_score_eq_modifier,
                                                    )),
                                                ),
                                        ),
                                    ),
                                )
                                .then(
                                    literal("<").then(
                                        argument("source", StringArgumentType::SingleWord).then(
                                            argument("source_obj", ObjectiveArgumentType)
                                                .redirect_with_modifier(
                                                    Redirection::Root,
                                                    RedirectModifier::Custom(Arc::new(
                                                        execute_unless_score_lt_modifier,
                                                    )),
                                                ),
                                        ),
                                    ),
                                )
                                .then(
                                    literal("<=").then(
                                        argument("source", StringArgumentType::SingleWord).then(
                                            argument("source_obj", ObjectiveArgumentType)
                                                .redirect_with_modifier(
                                                    Redirection::Root,
                                                    RedirectModifier::Custom(Arc::new(
                                                        execute_unless_score_le_modifier,
                                                    )),
                                                ),
                                        ),
                                    ),
                                )
                                .then(
                                    literal(">").then(
                                        argument("source", StringArgumentType::SingleWord).then(
                                            argument("source_obj", ObjectiveArgumentType)
                                                .redirect_with_modifier(
                                                    Redirection::Root,
                                                    RedirectModifier::Custom(Arc::new(
                                                        execute_unless_score_gt_modifier,
                                                    )),
                                                ),
                                        ),
                                    ),
                                )
                                .then(
                                    literal(">=").then(
                                        argument("source", StringArgumentType::SingleWord).then(
                                            argument("source_obj", ObjectiveArgumentType)
                                                .redirect_with_modifier(
                                                    Redirection::Root,
                                                    RedirectModifier::Custom(Arc::new(
                                                        execute_unless_score_ge_modifier,
                                                    )),
                                                ),
                                        ),
                                    ),
                                ),
                        ),
                    ),
                )
                .then(
                    literal("data")
                        .then(
                            literal("block").then(argument("pos", BlockPosArgumentType).then(
                                argument("path", NbtPathArgumentType).redirect_with_modifier(
                                    Redirection::Root,
                                    RedirectModifier::Custom(Arc::new(
                                        execute_unless_data_block_modifier,
                                    )),
                                ),
                            )),
                        )
                        .then(literal("entity").then(
                            argument("target", EntityArgumentType::Entity).then(
                                argument("path", NbtPathArgumentType).redirect_with_modifier(
                                    Redirection::Root,
                                    RedirectModifier::Custom(Arc::new(
                                        execute_unless_data_entity_modifier,
                                    )),
                                ),
                            ),
                        ))
                        .then(literal("storage").then(
                            argument("id", IdentifierArgumentType).then(
                                argument("path", NbtPathArgumentType).redirect_with_modifier(
                                    Redirection::Root,
                                    RedirectModifier::Custom(Arc::new(
                                        execute_unless_data_storage_modifier,
                                    )),
                                ),
                            ),
                        )),
                )
                .then(
                    literal("stopwatch").then(argument("id", IdentifierArgumentType).then(
                        argument("range", FloatRangeArgumentType).redirect_with_modifier(
                            Redirection::Root,
                            RedirectModifier::Custom(Arc::new(execute_unless_stopwatch_modifier)),
                        ),
                    )),
                ),
        );

    let execute_node_id = dispatcher.register(builder);

    set_redirects_to_execute(
        &mut dispatcher.tree,
        NodeId::from(execute_node_id),
        execute_node_id,
    );
}

fn set_redirects_to_execute(tree: &mut Tree, parent: NodeId, execute_id: CommandNodeId) {
    for child_id in tree.get_children(parent) {
        if let Some(redirect) = tree[child_id].redirect()
            && matches!(redirect, Redirection::Root)
            && tree[child_id].name() != "run"
        {
            tree[child_id].set_redirect(Some(Redirection::Local(NodeId::from(execute_id))));
        }
        set_redirects_to_execute(tree, child_id, execute_id);
    }
}
