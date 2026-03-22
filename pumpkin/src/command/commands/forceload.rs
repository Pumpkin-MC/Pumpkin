use pumpkin_data::translation;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::text::TextComponent;

use crate::command::args::position_2d::Position2DArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["forceload"];

const DESCRIPTION: &str = "Forces chunks to constantly be loaded or not.";

const ARG_FROM: &str = "from";
const ARG_TO: &str = "to";

const fn block_to_chunk(x: f64, z: f64) -> Vector2<i32> {
    Vector2::new((x.floor() as i32) >> 4, (z.floor() as i32) >> 4)
}

fn add_force_chunks(world: &crate::world::World, from: Vector2<i32>, to: Vector2<i32>) -> i32 {
    let min_x = from.x.min(to.x);
    let max_x = from.x.max(to.x);
    let min_z = from.y.min(to.y);
    let max_z = from.y.max(to.y);

    let mut count = 0i32;
    let mut lock = world.level.chunk_loading.lock().unwrap();
    for cx in min_x..=max_x {
        for cz in min_z..=max_z {
            let pos = Vector2::new(cx, cz);
            if !lock.high_priority.contains(&pos) {
                lock.add_force_ticket(pos);
                count += 1;
            }
        }
    }
    if count > 0 {
        lock.send_change();
    }
    count
}

fn remove_force_chunks(world: &crate::world::World, from: Vector2<i32>, to: Vector2<i32>) -> i32 {
    let min_x = from.x.min(to.x);
    let max_x = from.x.max(to.x);
    let min_z = from.y.min(to.y);
    let max_z = from.y.max(to.y);

    let mut count = 0i32;
    let mut lock = world.level.chunk_loading.lock().unwrap();
    for cx in min_x..=max_x {
        for cz in min_z..=max_z {
            let pos = Vector2::new(cx, cz);
            if lock.high_priority.contains(&pos) {
                lock.remove_force_ticket(pos);
                count += 1;
            }
        }
    }
    if count > 0 {
        lock.send_change();
    }
    count
}

struct AddSingleExecutor;

impl CommandExecutor for AddSingleExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let from = Position2DArgumentConsumer::find_arg(args, ARG_FROM)?;
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let chunk = block_to_chunk(from.x, from.y);

            let count = add_force_chunks(&world, chunk, chunk);
            if count == 0 {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_FORCELOAD_ADDED_FAILURE,
                    [TextComponent::text(format!("[{}, {}]", chunk.x, chunk.y))],
                )));
            }

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_FORCELOAD_ADDED_SINGLE,
                    [
                        TextComponent::text(format!("[{}, {}]", chunk.x, chunk.y)),
                        TextComponent::text(world.dimension.minecraft_name.to_owned()),
                    ],
                ))
                .await;
            Ok(count)
        })
    }
}

struct AddRangeExecutor;

impl CommandExecutor for AddRangeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let from = Position2DArgumentConsumer::find_arg(args, ARG_FROM)?;
            let to = Position2DArgumentConsumer::find_arg(args, ARG_TO)?;
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let chunk_from = block_to_chunk(from.x, from.y);
            let chunk_to = block_to_chunk(to.x, to.y);

            let dimension = world.dimension.minecraft_name.to_owned();
            let count = add_force_chunks(&world, chunk_from, chunk_to);
            if count == 0 {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_FORCELOAD_ADDED_NONE,
                    [TextComponent::text(dimension)],
                )));
            }

            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_FORCELOAD_ADDED_SINGLE,
                        [
                            TextComponent::text(format!("[{}, {}]", chunk_from.x, chunk_from.y)),
                            TextComponent::text(dimension),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_FORCELOAD_ADDED_MULTIPLE,
                        [
                            TextComponent::text(count.to_string()),
                            TextComponent::text(format!("[{}, {}]", chunk_from.x, chunk_from.y)),
                            TextComponent::text(format!("[{}, {}]", chunk_to.x, chunk_to.y)),
                            TextComponent::text(dimension),
                        ],
                    ))
                    .await;
            }
            Ok(count)
        })
    }
}

struct RemoveSingleExecutor;

impl CommandExecutor for RemoveSingleExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let from = Position2DArgumentConsumer::find_arg(args, ARG_FROM)?;
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let chunk = block_to_chunk(from.x, from.y);

            let count = remove_force_chunks(&world, chunk, chunk);
            if count == 0 {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_FORCELOAD_REMOVED_FAILURE,
                    [TextComponent::text(format!("[{}, {}]", chunk.x, chunk.y))],
                )));
            }

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_FORCELOAD_REMOVED_SINGLE,
                    [
                        TextComponent::text(format!("[{}, {}]", chunk.x, chunk.y)),
                        TextComponent::text(world.dimension.minecraft_name.to_owned()),
                    ],
                ))
                .await;
            Ok(count)
        })
    }
}

struct RemoveRangeExecutor;

impl CommandExecutor for RemoveRangeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let from = Position2DArgumentConsumer::find_arg(args, ARG_FROM)?;
            let to = Position2DArgumentConsumer::find_arg(args, ARG_TO)?;
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let chunk_from = block_to_chunk(from.x, from.y);
            let chunk_to = block_to_chunk(to.x, to.y);

            let dimension = world.dimension.minecraft_name.to_owned();
            let count = remove_force_chunks(&world, chunk_from, chunk_to);
            if count == 0 {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_FORCELOAD_REMOVED_FAILURE,
                    [TextComponent::text(format!(
                        "[{}, {}]",
                        chunk_from.x, chunk_from.y
                    ))],
                )));
            }

            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_FORCELOAD_REMOVED_SINGLE,
                        [
                            TextComponent::text(format!("[{}, {}]", chunk_from.x, chunk_from.y)),
                            TextComponent::text(dimension),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_FORCELOAD_REMOVED_MULTIPLE,
                        [
                            TextComponent::text(count.to_string()),
                            TextComponent::text(format!("[{}, {}]", chunk_from.x, chunk_from.y)),
                            TextComponent::text(format!("[{}, {}]", chunk_to.x, chunk_to.y)),
                            TextComponent::text(dimension),
                        ],
                    ))
                    .await;
            }
            Ok(count)
        })
    }
}

struct RemoveAllExecutor;

impl CommandExecutor for RemoveAllExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;

            let count;
            {
                let mut lock = world.level.chunk_loading.lock().unwrap();
                let forced: Vec<_> = lock.high_priority.clone();
                count = forced.len() as i32;
                for pos in forced {
                    lock.remove_force_ticket(pos);
                }
                if count > 0 {
                    lock.send_change();
                }
            }

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_FORCELOAD_REMOVED_ALL,
                    [
                        TextComponent::text(count.to_string()),
                        TextComponent::text(world.dimension.minecraft_name.to_owned()),
                    ],
                ))
                .await;
            Ok(count)
        })
    }
}

struct QueryPosExecutor;

impl CommandExecutor for QueryPosExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let from = Position2DArgumentConsumer::find_arg(args, ARG_FROM)?;
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let chunk = block_to_chunk(from.x, from.y);

            let is_forced = {
                let lock = world.level.chunk_loading.lock().unwrap();
                lock.high_priority.contains(&chunk)
            };

            if is_forced {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_FORCELOAD_QUERY_SUCCESS,
                        [TextComponent::text(format!("[{}, {}]", chunk.x, chunk.y))],
                    ))
                    .await;
                Ok(1)
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_FORCELOAD_QUERY_FAILURE,
                        [TextComponent::text(format!("[{}, {}]", chunk.x, chunk.y))],
                    ))
                    .await;
                Ok(0)
            }
        })
    }
}

struct QueryListExecutor;

impl CommandExecutor for QueryListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;

            let forced = {
                let lock = world.level.chunk_loading.lock().unwrap();
                lock.high_priority.clone()
            };

            let dimension = world.dimension.minecraft_name.to_owned();

            if forced.is_empty() {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_FORCELOAD_ADDED_NONE,
                        [TextComponent::text(dimension)],
                    ))
                    .await;
                Ok(0)
            } else {
                let list = forced
                    .iter()
                    .map(|p| format!("[{}, {}]", p.x, p.y))
                    .collect::<Vec<_>>()
                    .join(", ");

                if forced.len() == 1 {
                    sender
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_FORCELOAD_LIST_SINGLE,
                            [TextComponent::text(dimension), TextComponent::text(list)],
                        ))
                        .await;
                } else {
                    sender
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_FORCELOAD_LIST_MULTIPLE,
                            [
                                TextComponent::text(forced.len().to_string()),
                                TextComponent::text(dimension),
                                TextComponent::text(list),
                            ],
                        ))
                        .await;
                }
                Ok(forced.len() as i32)
            }
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("add").then(
                argument(ARG_FROM, Position2DArgumentConsumer)
                    .then(argument(ARG_TO, Position2DArgumentConsumer).execute(AddRangeExecutor))
                    .execute(AddSingleExecutor),
            ),
        )
        .then(
            literal("remove")
                .then(literal("all").execute(RemoveAllExecutor))
                .then(
                    argument(ARG_FROM, Position2DArgumentConsumer)
                        .then(
                            argument(ARG_TO, Position2DArgumentConsumer)
                                .execute(RemoveRangeExecutor),
                        )
                        .execute(RemoveSingleExecutor),
                ),
        )
        .then(
            literal("query")
                .then(argument(ARG_FROM, Position2DArgumentConsumer).execute(QueryPosExecutor))
                .execute(QueryListExecutor),
        )
}
