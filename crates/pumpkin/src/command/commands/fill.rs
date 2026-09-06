use pumpkin_data::translation;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;

use crate::command::argument_builder::{
    ArgumentBuilder, RequiredArgumentBuilder, argument, command, literal,
};
use crate::command::argument_types::block::BlockArgumentType;
use crate::command::argument_types::block_predicate::{BlockPredicate, BlockPredicateArgumentType};
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Fills all or parts of a region with a specific block.";
const PERMISSION: &str = "minecraft:command.fill";

const ERROR_AREA_TOO_LARGE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_FILL_TOOBIG,
    translation::java::COMMANDS_FILL_TOOBIG,
);

const ERROR_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_FILL_FAILED,
    translation::java::COMMANDS_FILL_FAILED,
);

#[derive(Clone, Copy, PartialEq, Eq)]
enum FillMode {
    Replace,
    Outline,
    Hollow,
    Destroy,
    Keep,
}

#[derive(Clone, Copy)]
enum FilterMode {
    None,
    WithFilter,
    KeepAir,
}

#[expect(clippy::too_many_lines)]
fn fill_blocks(
    source: &CommandSource,
    from: BlockPos,
    to: BlockPos,
    target_block: &'static Block,
    mode: FillMode,
    filter: Option<&BlockPredicate>,
    _strict: bool,
) -> Result<i32, CommandSyntaxError> {
    let min_x = from.0.x.min(to.0.x);
    let min_y = from.0.y.min(to.0.y);
    let min_z = from.0.z.min(to.0.z);
    let max_x = from.0.x.max(to.0.x);
    let max_y = from.0.y.max(to.0.y);
    let max_z = from.0.z.max(to.0.z);

    let x_span = i64::from(max_x - min_x + 1);
    let y_span = i64::from(max_y - min_y + 1);
    let z_span = i64::from(max_z - min_z + 1);
    let area = x_span * y_span * z_span;

    let world = source.world().clone();
    let max_block_modifications = {
        let level_info = world.level_info.load();
        level_info.game_rules.max_block_modifications
    };

    if area > max_block_modifications {
        return Err(ERROR_AREA_TOO_LARGE.create_without_context(
            TextComponent::text(max_block_modifications.to_string()),
            TextComponent::text(area.to_string()),
        ));
    }

    let target_state_id = target_block.default_state.id;
    let mut changed_positions = Vec::new();

    let min_chunk_x = min_x >> 4;
    let max_chunk_x = max_x >> 4;
    let min_chunk_z = min_z >> 4;
    let max_chunk_z = max_z >> 4;

    for chunk_x in min_chunk_x..=max_chunk_x {
        for chunk_z in min_chunk_z..=max_chunk_z {
            let chunk_pos = Vector2::new(chunk_x, chunk_z);
            let local_start_x = (min_x - (chunk_x << 4)).clamp(0, 15) as usize;
            let local_end_x = (max_x - (chunk_x << 4)).clamp(0, 15) as usize;
            let local_start_z = (min_z - (chunk_z << 4)).clamp(0, 15) as usize;
            let local_end_z = (max_z - (chunk_z << 4)).clamp(0, 15) as usize;

            let result = world.level.read_chunk_sync(&chunk_pos, |chunk| {
                let mut updates_for_chunk = Vec::new();
                let mut block_entities_to_remove = Vec::new();
                let min_y_chunk = chunk.section.min_y;

                {
                    let block_sections = chunk
                        .section
                        .block_sections
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);

                    for y in min_y..=max_y {
                        let y_rel = y - min_y_chunk;
                        if y_rel < 0 {
                            continue;
                        }
                        let rel_y = y_rel as usize;
                        let section_index = rel_y / pumpkin_world::chunk::CHUNK_WIDTH;
                        let sub_y = rel_y % pumpkin_world::chunk::CHUNK_WIDTH;

                        let Some(section) = block_sections.get(section_index) else {
                            continue;
                        };

                        for rel_z in local_start_z..=local_end_z {
                            for rel_x in local_start_x..=local_end_x {
                                let global_x = (chunk_x << 4) + rel_x as i32;
                                let global_z = (chunk_z << 4) + rel_z as i32;
                                let pos = BlockPos(Vector3::new(global_x, y, global_z));

                                let current_state_id = section.get(rel_x, sub_y, rel_z);
                                let current_block = Block::from_state_id(current_state_id);

                                match mode {
                                    FillMode::Keep => {
                                        if !current_block.is_air() {
                                            continue;
                                        }
                                    }
                                    _ => {
                                        if let Some(f) = filter
                                            && !f.test(current_block)
                                        {
                                            continue;
                                        }
                                    }
                                }

                                let is_edge = global_x == min_x
                                    || global_x == max_x
                                    || y == min_y
                                    || y == max_y
                                    || global_z == min_z
                                    || global_z == max_z;

                                let block_to_place = match mode {
                                    FillMode::Outline => is_edge.then_some(target_state_id),
                                    FillMode::Hollow => {
                                        if is_edge {
                                            Some(target_state_id)
                                        } else {
                                            Some(BlockStateId::AIR)
                                        }
                                    }
                                    FillMode::Destroy => {
                                        if current_state_id != target_state_id {
                                            world.break_block(
                                                &pos,
                                                None,
                                                BlockFlags::SKIP_DROPS
                                                    | BlockFlags::NOTIFY_ALL
                                                    | BlockFlags::FORCE_STATE,
                                            );
                                        }
                                        Some(target_state_id)
                                    }
                                    FillMode::Replace | FillMode::Keep => Some(target_state_id),
                                };

                                if let Some(state_id) = block_to_place {
                                    if current_block.default_state.block_entity_type != u16::MAX {
                                        block_entities_to_remove.push(pos);
                                    }
                                    updates_for_chunk.push((rel_x, y, rel_z, state_id, pos));
                                }
                            }
                        }
                    }
                }

                if updates_for_chunk.is_empty() {
                    return (Vec::new(), block_entities_to_remove);
                }

                let batch_inputs = updates_for_chunk
                    .iter()
                    .map(|&(rx, y, rz, sid, _)| (rx, y, rz, sid));
                chunk.set_blocks_batch(batch_inputs);

                let chunk_changed = updates_for_chunk
                    .into_iter()
                    .map(|(_, _, _, new_id, pos)| (pos, new_id))
                    .collect::<Vec<_>>();

                (chunk_changed, block_entities_to_remove)
            });

            if let Some((chunk_changed, be_to_remove)) = result {
                for pos in be_to_remove {
                    world.remove_block_entity(&pos);
                }
                changed_positions.extend(chunk_changed);
            }
        }
    }

    let count = changed_positions.len() as i32;
    if count == 0 {
        return Err(ERROR_FAILED.create_without_context());
    }

    world.queue_block_updates(&changed_positions);
    world.flush_block_updates();

    source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_FILL_SUCCESS,
            translation::java::COMMANDS_FILL_SUCCESS,
            [TextComponent::text(count.to_string())],
        ),
        true,
    );

    Ok(count)
}

struct FillExecutor {
    mode: FillMode,
    filter_mode: FilterMode,
    strict: bool,
}

impl CommandExecutor for FillExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let from = BlockPosArgumentType::get_loaded_block_pos(context, "from")?;
        let to = BlockPosArgumentType::get_loaded_block_pos(context, "to")?;
        let block = BlockArgumentType::get(context, "block")?;

        let filter = if matches!(self.filter_mode, FilterMode::WithFilter) {
            Some(BlockPredicateArgumentType::get(context, "filter")?)
        } else {
            None
        };

        fill_blocks(
            &context.source,
            from,
            to,
            block,
            self.mode,
            filter.as_ref(),
            self.strict,
        )
    }
}

fn wrap_with_mode(builder: RequiredArgumentBuilder, has_filter: bool) -> RequiredArgumentBuilder {
    let filter_mode = if has_filter {
        FilterMode::WithFilter
    } else {
        FilterMode::None
    };

    builder
        .executes(FillExecutor {
            mode: FillMode::Replace,
            filter_mode,
            strict: false,
        })
        .then(literal("outline").executes(FillExecutor {
            mode: FillMode::Outline,
            filter_mode,
            strict: false,
        }))
        .then(literal("hollow").executes(FillExecutor {
            mode: FillMode::Hollow,
            filter_mode,
            strict: false,
        }))
        .then(literal("destroy").executes(FillExecutor {
            mode: FillMode::Destroy,
            filter_mode,
            strict: false,
        }))
        .then(literal("strict").executes(FillExecutor {
            mode: FillMode::Replace,
            filter_mode,
            strict: true,
        }))
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let filter_arg = wrap_with_mode(argument("filter", BlockPredicateArgumentType), true);

    let replace_literal = literal("replace")
        .executes(FillExecutor {
            mode: FillMode::Replace,
            filter_mode: FilterMode::None,
            strict: false,
        })
        .then(filter_arg);

    let keep_literal = literal("keep").executes(FillExecutor {
        mode: FillMode::Keep,
        filter_mode: FilterMode::KeepAir,
        strict: false,
    });

    let block_arg = wrap_with_mode(argument("block", BlockArgumentType), false)
        .then(replace_literal)
        .then(keep_literal);

    dispatcher.register(
        command("fill", DESCRIPTION).requires(PERMISSION).then(
            argument("from", BlockPosArgumentType)
                .then(argument("to", BlockPosArgumentType).then(block_arg)),
        ),
    );
}
