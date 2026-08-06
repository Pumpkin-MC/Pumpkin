use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::Coordinates;
use crate::command::argument_types::coordinates::column_pos::ColumnPosArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::{Color, NamedColor};

const DESCRIPTION: &str = "Constantly load chunks in the world.";
const PERMISSION: &str = "minecraft:command.forceload";

static ERROR_FAILED_ADD: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_FORCELOAD_ADDED_FAILURE,
    "No world in source",
);
static ERROR_FAILED_REMOVE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_FORCELOAD_REMOVED_FAILURE,
    "No world in source",
);
static ERROR_FAILED_QUERY: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_FORCELOAD_QUERY_FAILURE,
    "No world in source",
);
static ERROR_TOO_MANY: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_FORCELOAD_TOOBIG,
    "Too many chunks",
);

struct ForceloadAddExecutor;

impl CommandExecutor for ForceloadAddExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let from_pos = ColumnPosArgumentType::get_column_pos(context, "from")?;

            let to_pos = if context.get_argument::<Coordinates>("to").is_ok() {
                ColumnPosArgumentType::get_column_pos(context, "to")?
            } else {
                from_pos
            };

            let world = context
                .source
                .world
                .as_ref()
                .ok_or_else(|| ERROR_FAILED_ADD.create_without_context())?;

            let chunk_x_start = from_pos.0.x >> 4;
            let chunk_z_start = from_pos.0.y >> 4;

            let chunk_x_end = to_pos.0.x >> 4;
            let chunk_z_end = to_pos.0.y >> 4;

            let min_x = chunk_x_start.min(chunk_x_end);
            let max_x = chunk_x_start.max(chunk_x_end);
            let min_z = chunk_z_start.min(chunk_z_end);
            let max_z = chunk_z_start.max(chunk_z_end);

            let count_x = max_x - min_x + 1;
            let count_z = max_z - min_z + 1;
            let total_chunks = count_x * count_z;

            if total_chunks > 256 {
                return Err(ERROR_TOO_MANY.create_without_context(
                    TextComponent::text("256"),
                    TextComponent::text(total_chunks.to_string()),
                ));
            }

            {
                let mut forced = world.forced_chunks.lock().unwrap();
                for x in min_x..=max_x {
                    for z in min_z..=max_z {
                        forced.insert(Vector2::new(x, z));
                    }
                }
            }

            world.update_active_chunks();

            let text = if total_chunks == 1 {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_FORCELOAD_ADDED_SINGLE,
                    translation::java::COMMANDS_FORCELOAD_ADDED_SINGLE,
                    [
                        TextComponent::text(min_x.to_string()),
                        TextComponent::text(min_z.to_string()),
                    ],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_FORCELOAD_ADDED_MULTIPLE,
                    translation::java::COMMANDS_FORCELOAD_ADDED_MULTIPLE,
                    [
                        TextComponent::text(total_chunks.to_string()),
                        TextComponent::text(min_x.to_string()),
                        TextComponent::text(min_z.to_string()),
                        TextComponent::text(max_x.to_string()),
                        TextComponent::text(max_z.to_string()),
                    ],
                )
            };
            context
                .source
                .send_feedback(text.color(Color::Named(NamedColor::Green)), false)
                .await;

            Ok(total_chunks)
        })
    }
}

struct ForceloadRemoveExecutor;

impl CommandExecutor for ForceloadRemoveExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let from_pos = ColumnPosArgumentType::get_column_pos(context, "from")?;

            let to_pos = if context.get_argument::<Coordinates>("to").is_ok() {
                ColumnPosArgumentType::get_column_pos(context, "to")?
            } else {
                from_pos
            };

            let world = context
                .source
                .world
                .as_ref()
                .ok_or_else(|| ERROR_FAILED_REMOVE.create_without_context())?;

            let chunk_x_start = from_pos.0.x >> 4;
            let chunk_z_start = from_pos.0.y >> 4;

            let chunk_x_end = to_pos.0.x >> 4;
            let chunk_z_end = to_pos.0.y >> 4;

            let min_x = chunk_x_start.min(chunk_x_end);
            let max_x = chunk_x_start.max(chunk_x_end);
            let min_z = chunk_z_start.min(chunk_z_end);
            let max_z = chunk_z_start.max(chunk_z_end);

            let count_x = max_x - min_x + 1;
            let count_z = max_z - min_z + 1;
            let total_chunks = count_x * count_z;

            if total_chunks > 256 {
                return Err(ERROR_TOO_MANY.create_without_context(
                    TextComponent::text("256"),
                    TextComponent::text(total_chunks.to_string()),
                ));
            }

            {
                let mut forced = world.forced_chunks.lock().unwrap();
                for x in min_x..=max_x {
                    for z in min_z..=max_z {
                        forced.remove(&Vector2::new(x, z));
                    }
                }
            }

            world.update_active_chunks();

            let text = if total_chunks == 1 {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_FORCELOAD_REMOVED_SINGLE,
                    translation::java::COMMANDS_FORCELOAD_REMOVED_SINGLE,
                    [
                        TextComponent::text(min_x.to_string()),
                        TextComponent::text(min_z.to_string()),
                    ],
                )
            } else {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_FORCELOAD_REMOVED_MULTIPLE,
                    translation::java::COMMANDS_FORCELOAD_REMOVED_MULTIPLE,
                    [
                        TextComponent::text(total_chunks.to_string()),
                        TextComponent::text(min_x.to_string()),
                        TextComponent::text(min_z.to_string()),
                        TextComponent::text(max_x.to_string()),
                        TextComponent::text(max_z.to_string()),
                    ],
                )
            };
            context
                .source
                .send_feedback(text.color(Color::Named(NamedColor::Green)), false)
                .await;

            Ok(total_chunks)
        })
    }
}

struct ForceloadRemoveAllExecutor;

impl CommandExecutor for ForceloadRemoveAllExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let world = context
                .source
                .world
                .as_ref()
                .ok_or_else(|| ERROR_FAILED_REMOVE.create_without_context())?;

            let removed_count = {
                let mut forced = world.forced_chunks.lock().unwrap();
                let count = forced.len();
                forced.clear();
                count
            };

            world.update_active_chunks();

            let text = TextComponent::translate_cross(
                translation::java::COMMANDS_FORCELOAD_REMOVED_ALL,
                translation::java::COMMANDS_FORCELOAD_REMOVED_ALL,
                [],
            );
            context
                .source
                .send_feedback(text.color(Color::Named(NamedColor::Green)), false)
                .await;

            Ok(removed_count as i32)
        })
    }
}

struct ForceloadQueryExecutor;

impl CommandExecutor for ForceloadQueryExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let world = context
                .source
                .world
                .as_ref()
                .ok_or_else(|| ERROR_FAILED_QUERY.create_without_context())?;

            let chunk_pos = if context.get_argument::<Coordinates>("pos").is_ok() {
                let pos = ColumnPosArgumentType::get_column_pos(context, "pos")?;
                Vector2::new(pos.0.x >> 4, pos.0.y >> 4)
            } else {
                let block_x = context.source.position.x.floor() as i32;
                let block_z = context.source.position.z.floor() as i32;
                Vector2::new(block_x >> 4, block_z >> 4)
            };

            let is_forced = {
                let forced = world.forced_chunks.lock().unwrap();
                forced.contains(&chunk_pos)
            };

            if is_forced {
                let text = TextComponent::translate_cross(
                    translation::java::COMMANDS_FORCELOAD_QUERY_SUCCESS,
                    translation::java::COMMANDS_FORCELOAD_QUERY_SUCCESS,
                    [
                        TextComponent::text(chunk_pos.x.to_string()),
                        TextComponent::text(chunk_pos.y.to_string()),
                    ],
                );
                context
                    .source
                    .send_feedback(text.color(Color::Named(NamedColor::Green)), false)
                    .await;
            } else {
                let text = TextComponent::translate_cross(
                    translation::java::COMMANDS_FORCELOAD_QUERY_FAILURE,
                    translation::java::COMMANDS_FORCELOAD_QUERY_FAILURE,
                    [
                        TextComponent::text(chunk_pos.x.to_string()),
                        TextComponent::text(chunk_pos.y.to_string()),
                    ],
                );
                context
                    .source
                    .send_feedback(text.color(Color::Named(NamedColor::Red)), false)
                    .await;
            }

            let all_forced = {
                let forced = world.forced_chunks.lock().unwrap();
                forced
                    .iter()
                    .map(|pos| format!("[{}, {}]", pos.x, pos.y))
                    .collect::<Vec<_>>()
            };

            if all_forced.is_empty() {
                let text = TextComponent::translate_cross(
                    translation::java::COMMANDS_FORCELOAD_ADDED_NONE,
                    translation::java::COMMANDS_FORCELOAD_ADDED_NONE,
                    [],
                );
                context
                    .source
                    .send_feedback(text.color(Color::Named(NamedColor::Gray)), false)
                    .await;
            } else {
                let list_str = all_forced.join(", ");
                let text = if all_forced.len() == 1 {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_FORCELOAD_LIST_SINGLE,
                        translation::java::COMMANDS_FORCELOAD_LIST_SINGLE,
                        [TextComponent::text(list_str)],
                    )
                } else {
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_FORCELOAD_LIST_MULTIPLE,
                        translation::java::COMMANDS_FORCELOAD_LIST_MULTIPLE,
                        [TextComponent::text(list_str)],
                    )
                };
                context
                    .source
                    .send_feedback(text.color(Color::Named(NamedColor::Gray)), false)
                    .await;
            }

            Ok(i32::from(is_forced))
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let builder = command("forceload", DESCRIPTION)
        .requires(PERMISSION)
        .then(
            literal("add").then(
                argument("from", ColumnPosArgumentType)
                    .executes(ForceloadAddExecutor)
                    .then(argument("to", ColumnPosArgumentType).executes(ForceloadAddExecutor)),
            ),
        )
        .then(
            literal("remove")
                .then(literal("all").executes(ForceloadRemoveAllExecutor))
                .then(
                    argument("from", ColumnPosArgumentType)
                        .executes(ForceloadRemoveExecutor)
                        .then(
                            argument("to", ColumnPosArgumentType).executes(ForceloadRemoveExecutor),
                        ),
                ),
        )
        .then(
            literal("query")
                .executes(ForceloadQueryExecutor)
                .then(argument("pos", ColumnPosArgumentType).executes(ForceloadQueryExecutor)),
        );

    dispatcher.register(builder);
}
