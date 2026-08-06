use std::sync::Arc;

use pumpkin_util::{math::vector2::Vector2, text::TextComponent};

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{
        ConsumedArgs, DefaultNameArgConsumer, FindArgDefaultName,
        bounded_num::BoundedNumArgumentConsumer, position_2d::Position2DArgumentConsumer,
    },
    tree::{
        CommandTree,
        builder::{argument_default_name, literal},
    },
};
use crate::world::World;

const NAMES: [&str; 1] = ["worldborder"];

const DESCRIPTION: &str = "Worldborder command.";

const NOTHING_CHANGED_EXCEPTION: &str = "commands.worldborder.set.failed.nochange";

/// Vanilla `WorldBorder.MAX_SIZE` (`5.999997E7F`, i.e. the float rounds to this).
const MAX_SIZE: f64 = 5.999_996_8E7;
/// Vanilla `WorldBorder.MAX_CENTER_COORDINATE`.
const MAX_CENTER_COORDINATE: f64 = 2.999_998_4E7;

// `WorldBorderCommand` registers `distance` as `doubleArg(-MAX_SIZE, MAX_SIZE)` for
// both `set` and `add`; the resulting size is range-checked in `setSize` instead.
const fn distance_consumer() -> BoundedNumArgumentConsumer<f64> {
    BoundedNumArgumentConsumer::new()
        .min(-MAX_SIZE)
        .max(MAX_SIZE)
        .name("distance")
}

/// Vanilla `WorldBorderCommand.formatTicksToSeconds`.
fn format_ticks_to_seconds(ticks: i32) -> String {
    format!("{:.2}", f64::from(ticks) / 20.0)
}

/// Vanilla `WorldBorderCommand.setSize`'s `ERROR_TOO_SMALL` / `ERROR_TOO_BIG` checks.
fn check_size(distance: f64) -> Result<(), CommandError> {
    if distance < 1.0 {
        return Err(CommandError::CommandFailed(TextComponent::translate_cross(
            "commands.worldborder.set.failed.small",
            "commands.worldborder.set.failed.small",
            [],
        )));
    }
    if distance > MAX_SIZE {
        return Err(CommandError::CommandFailed(TextComponent::translate_cross(
            "commands.worldborder.set.failed.big",
            "commands.worldborder.set.failed.big",
            [TextComponent::text(format!("{MAX_SIZE:.1}"))],
        )));
    }
    Ok(())
}

const fn time_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new().min(0).name("time")
}

const fn damage_per_block_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new()
        .min(0.0)
        .name("damage_per_block")
}

const fn damage_buffer_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new().min(0.0).name("buffer")
}

const fn warning_distance_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new().min(0).name("distance")
}

fn world_for_sender(
    sender: &CommandSender,
    server: &crate::server::Server,
) -> Result<Arc<World>, CommandError> {
    match sender {
        CommandSender::Player(player) => Ok(player.world()),
        CommandSender::CommandBlock(_, world) => Ok(world.clone()),
        CommandSender::Console | CommandSender::Rcon(_) | CommandSender::Dummy => server
            .worlds
            .load()
            .first()
            .cloned()
            .ok_or(CommandError::InvalidRequirement),
    }
}

struct GetExecutor;

impl CommandExecutor for GetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = world_for_sender(sender, server)?;
            let border = world.worldborder.lock().await;

            let diameter = border.size().round() as i32;
            sender
                .send_message(TextComponent::translate_cross(
                    "commands.worldborder.get",
                    "commands.worldborder.get",
                    [TextComponent::text(diameter.to_string())],
                ))
                .await;

            Ok(diameter)
        })
    }
}

struct SetExecutor;

impl CommandExecutor for SetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = world_for_sender(sender, server)?;
            let mut border = world.worldborder.lock().await;

            let Ok(distance) = distance_consumer().find_arg_default_name(args)? else {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "{} is out of bounds.",
                    distance_consumer().default_name()
                ))));
            };

            if (distance - border.size()).abs() < f64::EPSILON {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    NOTHING_CHANGED_EXCEPTION,
                    NOTHING_CHANGED_EXCEPTION,
                    [],
                )));
            }
            check_size(distance)?;

            sender
                .send_message(TextComponent::translate_cross(
                    "commands.worldborder.set.immediate",
                    "commands.worldborder.set.immediate",
                    [TextComponent::text(format!("{distance:.1}"))],
                ))
                .await;

            let d = border.size();
            border.set_diameter(&world, distance, None);

            Ok((distance - d) as i32)
        })
    }
}

struct SetTimeExecutor;

impl CommandExecutor for SetTimeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = world_for_sender(sender, server)?;
            let mut border = world.worldborder.lock().await;

            let Ok(distance) = distance_consumer().find_arg_default_name(args)? else {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "{} is out of bounds.",
                    distance_consumer().default_name()
                ))));
            };
            let Ok(time) = time_consumer().find_arg_default_name(args)? else {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "{} is out of bounds.",
                    distance_consumer().default_name()
                ))));
            };

            check_size(distance)?;

            match distance.total_cmp(&border.size()) {
                std::cmp::Ordering::Equal => {
                    return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        NOTHING_CHANGED_EXCEPTION,
                        NOTHING_CHANGED_EXCEPTION,
                        [],
                    )));
                }
                std::cmp::Ordering::Less => {
                    let dist = format!("{distance:.1}");
                    sender
                        .send_message(TextComponent::translate_cross(
                            "commands.worldborder.set.shrink",
                            "commands.worldborder.set.shrink",
                            [
                                TextComponent::text(dist),
                                TextComponent::text(format_ticks_to_seconds(time)),
                            ],
                        ))
                        .await;
                }
                std::cmp::Ordering::Greater => {
                    let dist = format!("{distance:.1}");
                    sender
                        .send_message(TextComponent::translate_cross(
                            "commands.worldborder.set.grow",
                            "commands.worldborder.set.grow",
                            [
                                TextComponent::text(dist),
                                TextComponent::text(format_ticks_to_seconds(time)),
                            ],
                        ))
                        .await;
                }
            }

            let d = border.size();
            border.set_diameter(&world, distance, Some(i64::from(time)));

            Ok((distance - d) as i32)
        })
    }
}

struct AddExecutor;

impl CommandExecutor for AddExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = world_for_sender(sender, server)?;
            let mut border = world.worldborder.lock().await;

            let Ok(distance_add) = distance_consumer().find_arg_default_name(args)? else {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "{} is out of bounds.",
                    distance_consumer().default_name()
                ))));
            };

            if distance_add == 0.0 {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    NOTHING_CHANGED_EXCEPTION,
                    NOTHING_CHANGED_EXCEPTION,
                    [],
                )));
            }

            let distance = border.size() + distance_add;
            check_size(distance)?;

            let dist = format!("{distance:.1}");
            sender
                .send_message(TextComponent::translate_cross(
                    "commands.worldborder.set.immediate",
                    "commands.worldborder.set.immediate",
                    [TextComponent::text(dist)],
                ))
                .await;
            border.set_diameter(&world, distance, None);
            Ok(distance_add as i32)
        })
    }
}

struct AddTimeExecutor;

impl CommandExecutor for AddTimeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = world_for_sender(sender, server)?;
            let mut border = world.worldborder.lock().await;

            let Ok(distance_add) = distance_consumer().find_arg_default_name(args)? else {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "{} is out of bounds.",
                    distance_consumer().default_name()
                ))));
            };
            let Ok(time) = time_consumer().find_arg_default_name(args)? else {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "{} is out of bounds.",
                    distance_consumer().default_name()
                ))));
            };

            let distance = distance_add + border.size();

            check_size(distance)?;

            match distance.total_cmp(&border.size()) {
                std::cmp::Ordering::Equal => {
                    return Err(CommandError::CommandFailed(TextComponent::text(format!(
                        "{} is out of bounds.",
                        distance_consumer().default_name()
                    ))));
                }
                std::cmp::Ordering::Less => {
                    let dist = format!("{distance:.1}");
                    sender
                        .send_message(TextComponent::translate_cross(
                            "commands.worldborder.set.shrink",
                            "commands.worldborder.set.shrink",
                            [
                                TextComponent::text(dist),
                                TextComponent::text(format_ticks_to_seconds(time)),
                            ],
                        ))
                        .await;
                }
                std::cmp::Ordering::Greater => {
                    let dist = format!("{distance:.1}");
                    sender
                        .send_message(TextComponent::translate_cross(
                            "commands.worldborder.set.grow",
                            "commands.worldborder.set.grow",
                            [
                                TextComponent::text(dist),
                                TextComponent::text(format_ticks_to_seconds(time)),
                            ],
                        ))
                        .await;
                }
            }

            let ticks = border.lerp_time() + i64::from(time);
            border.set_diameter(&world, distance, Some(ticks));

            Ok(distance_add as i32)
        })
    }
}

struct CenterExecutor;

impl CommandExecutor for CenterExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = world_for_sender(sender, server)?;
            let mut border = world.worldborder.lock().await;

            let Vector2 { x, y } = Position2DArgumentConsumer.find_arg_default_name(args)?;

            if (x - border.center_x).abs() < f64::EPSILON
                && (y - border.center_z).abs() < f64::EPSILON
            {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    "commands.worldborder.center.failed",
                    "commands.worldborder.center.failed",
                    [],
                )));
            }
            if x.abs() > MAX_CENTER_COORDINATE || y.abs() > MAX_CENTER_COORDINATE {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    "commands.worldborder.set.failed.far",
                    "commands.worldborder.set.failed.far",
                    [TextComponent::text(format!("{MAX_CENTER_COORDINATE:.1}"))],
                )));
            }

            sender
                .send_message(TextComponent::translate_cross(
                    "commands.worldborder.center.success",
                    "commands.worldborder.center.success",
                    [
                        TextComponent::text(format!("{x:.2}")),
                        TextComponent::text(format!("{y:.2}")),
                    ],
                ))
                .await;
            border.set_center(&world, x, y);
            Ok(0)
        })
    }
}

struct DamageAmountExecutor;

impl CommandExecutor for DamageAmountExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = world_for_sender(sender, server)?;
            let mut border = world.worldborder.lock().await;

            let Ok(damage_per_block) = damage_per_block_consumer().find_arg_default_name(args)?
            else {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "{} is out of bounds.",
                    distance_consumer().default_name()
                ))));
            };

            if (damage_per_block - border.damage_per_block).abs() < f32::EPSILON {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    "commands.worldborder.damage.amount.failed",
                    "commands.worldborder.damage.amount.failed",
                    [],
                )));
            }

            let damage = format!("{damage_per_block:.2}");
            sender
                .send_message(TextComponent::translate_cross(
                    "commands.worldborder.damage.amount.success",
                    "commands.worldborder.damage.amount.success",
                    [TextComponent::text(damage)],
                ))
                .await;
            border.damage_per_block = damage_per_block;
            Ok(damage_per_block as i32)
        })
    }
}

struct DamageBufferExecutor;

impl CommandExecutor for DamageBufferExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = world_for_sender(sender, server)?;
            let mut border = world.worldborder.lock().await;

            let Ok(buffer) = damage_buffer_consumer().find_arg_default_name(args)? else {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "{} is out of bounds.",
                    distance_consumer().default_name()
                ))));
            };

            if (buffer - border.buffer).abs() < f32::EPSILON {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    "commands.worldborder.damage.buffer.failed",
                    "commands.worldborder.damage.buffer.failed",
                    [],
                )));
            }

            let buf = format!("{buffer:.2}");
            sender
                .send_message(TextComponent::translate_cross(
                    "commands.worldborder.damage.buffer.success",
                    "commands.worldborder.damage.buffer.success",
                    [TextComponent::text(buf)],
                ))
                .await;
            border.buffer = buffer;
            Ok(buffer as i32)
        })
    }
}

struct WarningDistanceExecutor;

impl CommandExecutor for WarningDistanceExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = world_for_sender(sender, server)?;
            let mut border = world.worldborder.lock().await;

            let Ok(distance) = warning_distance_consumer().find_arg_default_name(args)? else {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "{} is out of bounds.",
                    distance_consumer().default_name()
                ))));
            };

            if distance == border.warning_blocks {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    "commands.worldborder.warning.distance.failed",
                    "commands.worldborder.warning.distance.failed",
                    [],
                )));
            }

            sender
                .send_message(TextComponent::translate_cross(
                    "commands.worldborder.warning.distance.success",
                    "commands.worldborder.warning.distance.success",
                    [TextComponent::text(distance.to_string())],
                ))
                .await;
            border.set_warning_distance(&world, distance);
            Ok(distance)
        })
    }
}

struct WarningTimeExecutor;

impl CommandExecutor for WarningTimeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let world = world_for_sender(sender, server)?;
            let mut border = world.worldborder.lock().await;

            let Ok(time) = time_consumer().find_arg_default_name(args)? else {
                return Err(CommandError::CommandFailed(TextComponent::text(format!(
                    "{} is out of bounds.",
                    distance_consumer().default_name()
                ))));
            };

            if time == border.warning_time {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    "commands.worldborder.warning.time.failed",
                    "commands.worldborder.warning.time.failed",
                    [],
                )));
            }

            sender
                .send_message(TextComponent::translate_cross(
                    "commands.worldborder.warning.time.success",
                    "commands.worldborder.warning.time.success",
                    [TextComponent::text(format_ticks_to_seconds(time))],
                ))
                .await;
            border.set_warning_delay(&world, time);
            Ok(time)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("add").then(
                argument_default_name(distance_consumer())
                    .execute(AddExecutor)
                    .then(argument_default_name(time_consumer()).execute(AddTimeExecutor)),
            ),
        )
        .then(
            literal("center")
                .then(argument_default_name(Position2DArgumentConsumer).execute(CenterExecutor)),
        )
        .then(
            literal("damage")
                .then(
                    literal("amount").then(
                        argument_default_name(damage_per_block_consumer())
                            .execute(DamageAmountExecutor),
                    ),
                )
                .then(literal("buffer").then(
                    argument_default_name(damage_buffer_consumer()).execute(DamageBufferExecutor),
                )),
        )
        .then(literal("get").execute(GetExecutor))
        .then(
            literal("set").then(
                argument_default_name(distance_consumer())
                    .execute(SetExecutor)
                    .then(argument_default_name(time_consumer()).execute(SetTimeExecutor)),
            ),
        )
        .then(
            literal("warning")
                .then(
                    literal("distance").then(
                        argument_default_name(warning_distance_consumer())
                            .execute(WarningDistanceExecutor),
                    ),
                )
                .then(
                    literal("time")
                        .then(argument_default_name(time_consumer()).execute(WarningTimeExecutor)),
                ),
        )
}
