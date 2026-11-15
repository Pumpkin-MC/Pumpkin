use std::collections::VecDeque;
use std::sync::Arc;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::block_predicate::BlockPredicateArgumentType;
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::resource_key::{DIMENSION_REGISTRY, ResourceKeyArgument};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::world::World;
use pumpkin_data::{Block, BlockState, BlockStateId, translation};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::PermissionLvl;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;

const DESCRIPTION: &str = "Clones blocks from one region to another.";
const PERMISSION: &str = "minecraft:command.clone";

const ARG_SOURCE_DIM: &str = "sourceDimension";
const ARG_TARGET_DIM: &str = "targetDimension";
const ARG_BEGIN: &str = "begin";
const ARG_END: &str = "end";
const ARG_DEST: &str = "destination";
const ARG_FILTER: &str = "filter";

static ERROR_INVALID_DIMENSION: CommandErrorType<1> =
    CommandErrorType::new("argument.dimension.invalid", "argument.dimension.invalid");

const OVERLAP_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_CLONE_OVERLAP,
    translation::bedrock::COMMANDS_CLONE_NOOVERLAP,
);

const TOOBIG_ERROR: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_CLONE_TOOBIG,
    translation::bedrock::COMMANDS_CLONE_TOOMANYBLOCKS,
);

const FAILED_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_CLONE_FAILED,
    translation::bedrock::COMMANDS_CLONE_FAILED,
);

const NOT_LOADED_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_POS_UNLOADED,
    translation::java::ARGUMENT_POS_UNLOADED,
);

const OUT_OF_WORLD_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_POS_OUTOFWORLD,
    translation::java::ARGUMENT_POS_OUTOFWORLD,
);

const STRICT_SKIP_FLAGS: BlockFlags = BlockFlags::from_bits_truncate(
    BlockFlags::UPDATE_KNOWN_SHAPE.bits()
        | BlockFlags::SKIP_DROPS.bits()
        | BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK.bits()
        | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK.bits(),
);

#[derive(Clone, Copy, PartialEq, Eq)]
enum MaskMode {
    Replace,
    Masked,
    Filtered,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CloneMode {
    Normal,
    Force,
    Move,
}

impl CloneMode {
    const fn can_overlap(self) -> bool {
        matches!(self, Self::Force | Self::Move)
    }
}

#[derive(Clone, Copy)]
struct CloneExecutor {
    mask_mode: MaskMode,
    clone_mode: CloneMode,
    has_source_dim: bool,
    has_target_dim: bool,
    strict: bool,
}

struct CloneBlockInfo {
    dest_pos: BlockPos,
    state_id: BlockStateId,
    block_entity_nbt: Option<NbtCompound>,
    previous_dest_state_id: BlockStateId,
}

fn resolve_dimension(
    context: &CommandContext,
    arg_name: &str,
) -> Result<Arc<World>, CommandSyntaxError> {
    let dimension_key = ResourceKeyArgument::get_registry_key(
        context,
        arg_name,
        &Identifier::vanilla_static("dimension"),
        &ERROR_INVALID_DIMENSION,
    )?;
    let dimension_name = dimension_key.identifier.to_string();
    let worlds = context.server().worlds.load();
    worlds
        .iter()
        .find(|w| w.dimension.minecraft_name == dimension_name)
        .cloned()
        .ok_or_else(|| {
            ERROR_INVALID_DIMENSION.create_without_context(TextComponent::text(dimension_name))
        })
}

fn check_region_loaded(
    world: &World,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
) -> Result<(), CommandSyntaxError> {
    if !world.is_in_build_limit(BlockPos::new(min_x, min_y, min_z))
        || !world.is_in_build_limit(BlockPos::new(max_x, max_y, max_z))
    {
        return Err(OUT_OF_WORLD_ERROR.create_without_context());
    }
    for cx in (min_x >> 4)..=(max_x >> 4) {
        for cz in (min_z >> 4)..=(max_z >> 4) {
            let chunk_pos = Vector2::new(cx, cz);
            if world.level.read_chunk_sync(&chunk_pos, |_| ()).is_none() {
                return Err(NOT_LOADED_ERROR.create_without_context());
            }
        }
    }
    Ok(())
}

impl CommandExecutor for CloneExecutor {
    #[expect(clippy::too_many_lines)]
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let from_world = if self.has_source_dim {
            resolve_dimension(context, ARG_SOURCE_DIM)?
        } else {
            context.source.world().clone()
        };
        let to_world = if self.has_target_dim {
            resolve_dimension(context, ARG_TARGET_DIM)?
        } else {
            context.source.world().clone()
        };

        let begin = BlockPosArgumentType::get_block_pos(context, ARG_BEGIN)?;
        let end = BlockPosArgumentType::get_block_pos(context, ARG_END)?;
        let dest = BlockPosArgumentType::get_block_pos(context, ARG_DEST)?;

        let filter_block = if self.mask_mode == MaskMode::Filtered {
            Some(BlockPredicateArgumentType::get(context, ARG_FILTER)?)
        } else {
            None
        };

        let min_x = begin.0.x.min(end.0.x);
        let max_x = begin.0.x.max(end.0.x);
        let min_y = begin.0.y.min(end.0.y);
        let max_y = begin.0.y.max(end.0.y);
        let min_z = begin.0.z.min(end.0.z);
        let max_z = begin.0.z.max(end.0.z);

        let size_x = max_x - min_x + 1;
        let size_y = max_y - min_y + 1;
        let size_z = max_z - min_z + 1;

        let dest_min_x = dest.0.x;
        let dest_min_y = dest.0.y;
        let dest_min_z = dest.0.z;
        let dest_max_x = dest_min_x + size_x - 1;
        let dest_max_y = dest_min_y + size_y - 1;
        let dest_max_z = dest_min_z + size_z - 1;

        let overlap = !(dest_max_x < min_x
            || dest_min_x > max_x
            || dest_max_y < min_y
            || dest_min_y > max_y
            || dest_max_z < min_z
            || dest_min_z > max_z);

        if !self.clone_mode.can_overlap() && Arc::ptr_eq(&from_world, &to_world) && overlap {
            return Err(OVERLAP_ERROR.create_without_context());
        }

        let area = i64::from(size_x) * i64::from(size_y) * i64::from(size_z);
        let limit = context
            .source
            .world()
            .level_info
            .load()
            .game_rules
            .max_block_modifications;

        if area > limit {
            return Err(TOOBIG_ERROR.create_without_context(
                TextComponent::text(limit.to_string()),
                TextComponent::text(area.to_string()),
            ));
        }

        check_region_loaded(&from_world, min_x, min_y, min_z, max_x, max_y, max_z)?;
        check_region_loaded(
            &to_world, dest_min_x, dest_min_y, dest_min_z, dest_max_x, dest_max_y, dest_max_z,
        )?;

        let offset_x = dest_min_x - min_x;
        let offset_y = dest_min_y - min_y;
        let offset_z = dest_min_z - min_z;

        let mut solid_list = Vec::new();
        let mut block_entities_list = Vec::new();
        let mut other_blocks_list = Vec::new();
        let mut clear_blocks_list = VecDeque::new();

        for z in min_z..=max_z {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let src_pos = BlockPos::new(x, y, z);
                    let dest_pos = BlockPos::new(x + offset_x, y + offset_y, z + offset_z);
                    let state_id = from_world.get_block_state_id(&src_pos);

                    let should_clone = match self.mask_mode {
                        MaskMode::Replace => true,
                        MaskMode::Masked => !pumpkin_data::block_properties::is_air(state_id),
                        MaskMode::Filtered => filter_block.as_ref().is_some_and(|f| {
                            f.test(state_id, || from_world.get_block_entity_nbt(&src_pos))
                        }),
                    };
                    if !should_clone {
                        continue;
                    }

                    let previous_dest_state_id = to_world.get_block_state_id(&dest_pos);
                    if let Some(be) = from_world.get_block_entity(&src_pos) {
                        let mut nbt = NbtCompound::new();
                        be.write_nbt(&mut nbt);
                        block_entities_list.push(CloneBlockInfo {
                            dest_pos,
                            state_id,
                            block_entity_nbt: Some(nbt),
                            previous_dest_state_id,
                        });
                        clear_blocks_list.push_back(src_pos);
                    } else {
                        let state = BlockState::from_id(state_id);
                        let info = CloneBlockInfo {
                            dest_pos,
                            state_id,
                            block_entity_nbt: None,
                            previous_dest_state_id,
                        };
                        if !state.is_solid() && !state.is_full_cube() {
                            other_blocks_list.push(info);
                            clear_blocks_list.push_front(src_pos);
                        } else {
                            solid_list.push(info);
                            clear_blocks_list.push_back(src_pos);
                        }
                    }
                }
            }
        }

        let default_update_flags = BlockFlags::NOTIFY_LISTENERS
            | if self.strict {
                STRICT_SKIP_FLAGS
            } else {
                BlockFlags::empty()
            };

        if self.clone_mode == CloneMode::Move {
            for pos in &clear_blocks_list {
                from_world.set_block_state(
                    pos,
                    Block::BARRIER.default_state.id,
                    default_update_flags | STRICT_SKIP_FLAGS,
                );
            }
            let standard_flags = if self.strict {
                default_update_flags
            } else {
                BlockFlags::NOTIFY_LISTENERS | BlockFlags::NOTIFY_NEIGHBORS
            };
            for pos in &clear_blocks_list {
                from_world.set_block_state(pos, BlockStateId::AIR, standard_flags);
            }
        }

        let mut block_info_list = Vec::with_capacity(
            solid_list.len() + block_entities_list.len() + other_blocks_list.len(),
        );
        block_info_list.extend(solid_list);
        let be_start = block_info_list.len();
        block_info_list.extend(block_entities_list);
        let be_end = block_info_list.len();
        block_info_list.extend(other_blocks_list);

        for info in block_info_list.iter().rev() {
            to_world.set_block_state(
                &info.dest_pos,
                Block::BARRIER.default_state.id,
                default_update_flags | STRICT_SKIP_FLAGS,
            );
        }

        let mut count = 0;
        for info in &block_info_list {
            let replaced =
                to_world.set_block_state(&info.dest_pos, info.state_id, default_update_flags);
            if replaced != info.state_id {
                count += 1;
            }
        }

        for info in &block_info_list[be_start..be_end] {
            if let Some(tag) = &info.block_entity_nbt {
                let _ = to_world.load_block_entity_nbt(info.dest_pos, tag);
            }
            to_world.set_block_state(&info.dest_pos, info.state_id, default_update_flags);
        }

        if !self.strict {
            for info in block_info_list.iter().rev() {
                to_world
                    .update_neighbours_on_block_set(&info.dest_pos, info.previous_dest_state_id);
            }
        }

        from_world.flush_block_updates();
        if !Arc::ptr_eq(&from_world, &to_world) {
            to_world.flush_block_updates();
        }

        if count == 0 {
            return Err(FAILED_ERROR.create_without_context());
        }

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_CLONE_SUCCESS,
                translation::bedrock::COMMANDS_CLONE_SUCCESS,
                [TextComponent::text(count.to_string())],
            ),
            true,
        );

        Ok(count)
    }
}

fn wrap_with_clone_mode<B: ArgumentBuilder>(
    builder: B,
    mask_mode: MaskMode,
    has_source_dim: bool,
    has_target_dim: bool,
    strict: bool,
) -> B {
    builder
        .executes(CloneExecutor {
            mask_mode,
            clone_mode: CloneMode::Normal,
            has_source_dim,
            has_target_dim,
            strict,
        })
        .then(literal("force").executes(CloneExecutor {
            mask_mode,
            clone_mode: CloneMode::Force,
            has_source_dim,
            has_target_dim,
            strict,
        }))
        .then(literal("move").executes(CloneExecutor {
            mask_mode,
            clone_mode: CloneMode::Move,
            has_source_dim,
            has_target_dim,
            strict,
        }))
        .then(literal("normal").executes(CloneExecutor {
            mask_mode,
            clone_mode: CloneMode::Normal,
            has_source_dim,
            has_target_dim,
            strict,
        }))
}

fn mode_suffix<B: ArgumentBuilder>(
    builder: B,
    has_source_dim: bool,
    has_target_dim: bool,
    strict: bool,
) -> B {
    builder
        .executes(CloneExecutor {
            mask_mode: MaskMode::Replace,
            clone_mode: CloneMode::Normal,
            has_source_dim,
            has_target_dim,
            strict,
        })
        .then(wrap_with_clone_mode(
            literal("replace"),
            MaskMode::Replace,
            has_source_dim,
            has_target_dim,
            strict,
        ))
        .then(wrap_with_clone_mode(
            literal("masked"),
            MaskMode::Masked,
            has_source_dim,
            has_target_dim,
            strict,
        ))
        .then(literal("filtered").then(wrap_with_clone_mode(
            argument(ARG_FILTER, BlockPredicateArgumentType),
            MaskMode::Filtered,
            has_source_dim,
            has_target_dim,
            strict,
        )))
}

fn destination_and_strict_suffix(
    has_source_dim: bool,
    has_target_dim: bool,
) -> crate::command::argument_builder::RequiredArgumentBuilder {
    mode_suffix(
        argument(ARG_DEST, BlockPosArgumentType),
        has_source_dim,
        has_target_dim,
        false,
    )
    .then(mode_suffix(
        literal("strict"),
        has_source_dim,
        has_target_dim,
        true,
    ))
}

fn begin_end_destination_suffix(
    has_source_dim: bool,
) -> crate::command::argument_builder::RequiredArgumentBuilder {
    argument(ARG_BEGIN, BlockPosArgumentType).then(
        argument(ARG_END, BlockPosArgumentType)
            .then(destination_and_strict_suffix(has_source_dim, false))
            .then(
                literal("to").then(
                    argument(ARG_TARGET_DIM, ResourceKeyArgument(DIMENSION_REGISTRY))
                        .then(destination_and_strict_suffix(has_source_dim, true)),
                ),
            ),
    )
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("clone", DESCRIPTION)
            .requires(PERMISSION)
            .then(begin_end_destination_suffix(false))
            .then(
                literal("from").then(
                    argument(ARG_SOURCE_DIM, ResourceKeyArgument(DIMENSION_REGISTRY))
                        .then(begin_end_destination_suffix(true)),
                ),
            ),
    );
}
