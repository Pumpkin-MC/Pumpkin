use pumpkin_data::translation;
use pumpkin_util::{math::vector2::Vector2, text::TextComponent};

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{
        ConsumedArgs, FindArgDefaultName, bounded_num::BoundedNumArgumentConsumer,
        position_2d::Position2DArgumentConsumer,
    },
    tree::{
        CommandTree,
        builder::{argument_default_name, literal},
    },
};

const NAMES: [&str; 1] = ["worldborder"];

const DESCRIPTION: &str = "Worldborder command.";

const fn distance_consumer() -> BoundedNumArgumentConsumer<f64> {
    BoundedNumArgumentConsumer::new().min(0.0).name("distance")
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

struct GetExecutor;

impl CommandExecutor for GetExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let border = world.worldborder.lock().await;

            let diameter = border.new_diameter.round() as i32;
            sender
                .send_message(pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WORLDBORDER_GET,
                    translation::bedrock::COMMANDS_WORLDBORDER_GET_SUCCESS,
                    TextComponent::text(diameter.to_string())
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
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let mut border = world.worldborder.lock().await;

            let distance = distance_consumer().find_arg_default_name(args)??;

            if (distance - border.new_diameter).abs() < f64::EPSILON {
                return Err(CommandError::CommandFailed(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_WORLDBORDER_SET_FAILED_NOCHANGE,
                        translation::bedrock::COMMANDS_WORLDBORDER_SET_SUCCESS
                    ),
                ));
            }

            let d = border.new_diameter;
            sender
                .send_message(pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WORLDBORDER_SET_IMMEDIATE,
                    translation::bedrock::COMMANDS_WORLDBORDER_SET_SUCCESS,
                    TextComponent::text(format!("{distance:.1}")),
                    TextComponent::text(format!("{d:.1}"))
                ))
                .await;

            border.set_diameter(world, distance, None);

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
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let mut border = world.worldborder.lock().await;

            let distance = distance_consumer().find_arg_default_name(args)??;
            let time = time_consumer().find_arg_default_name(args)??;

            let old_dist = format!("{:.1}", border.new_diameter);
            match distance.total_cmp(&border.new_diameter) {
                std::cmp::Ordering::Equal => {
                    return Err(CommandError::CommandFailed(
                        pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_WORLDBORDER_SET_FAILED_NOCHANGE,
                            translation::bedrock::COMMANDS_WORLDBORDER_SET_SUCCESS
                        ),
                    ));
                }
                std::cmp::Ordering::Less => {
                    let dist = format!("{distance:.1}");
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_WORLDBORDER_SET_SHRINK,
                            translation::bedrock::COMMANDS_WORLDBORDER_SETSLOWLY_SHRINK_SUCCESS,
                            TextComponent::text(dist),
                            TextComponent::text(old_dist),
                            TextComponent::text(time.to_string())
                        ))
                        .await;
                }
                std::cmp::Ordering::Greater => {
                    let dist = format!("{distance:.1}");
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_WORLDBORDER_SET_GROW,
                            translation::bedrock::COMMANDS_WORLDBORDER_SETSLOWLY_GROW_SUCCESS,
                            TextComponent::text(dist),
                            TextComponent::text(old_dist),
                            TextComponent::text(time.to_string())
                        ))
                        .await;
                }
            }

            let d = border.new_diameter;
            border.set_diameter(world, distance, Some(i64::from(time) * 1000));

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
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let mut border = world.worldborder.lock().await;

            let distance_add = distance_consumer().find_arg_default_name(args)??;

            if distance_add == 0.0 {
                return Err(CommandError::CommandFailed(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_WORLDBORDER_SET_FAILED_NOCHANGE,
                        translation::bedrock::COMMANDS_WORLDBORDER_SET_SUCCESS
                    ),
                ));
            }

            let distance = border.new_diameter + distance_add;

            let dist = format!("{distance:.1}");
            let old_dist = format!("{:.1}", border.new_diameter);
            sender
                .send_message(pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WORLDBORDER_SET_IMMEDIATE,
                    translation::bedrock::COMMANDS_WORLDBORDER_SET_SUCCESS,
                    TextComponent::text(dist),
                    TextComponent::text(old_dist)
                ))
                .await;
            border.set_diameter(world, distance, None);
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
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let mut border = world.worldborder.lock().await;

            let distance_add = distance_consumer().find_arg_default_name(args)??;
            let time = time_consumer().find_arg_default_name(args)??;

            let distance = distance_add + border.new_diameter;

            let old_dist = format!("{:.1}", border.new_diameter);
            match distance.total_cmp(&border.new_diameter) {
                std::cmp::Ordering::Equal => {
                    return Err(CommandError::CommandFailed(
                        pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_WORLDBORDER_SET_FAILED_NOCHANGE,
                            translation::bedrock::COMMANDS_WORLDBORDER_SET_SUCCESS
                        ),
                    ));
                }
                std::cmp::Ordering::Less => {
                    let dist = format!("{distance:.1}");
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_WORLDBORDER_SET_SHRINK,
                            translation::bedrock::COMMANDS_WORLDBORDER_SETSLOWLY_SHRINK_SUCCESS,
                            TextComponent::text(dist),
                            TextComponent::text(old_dist),
                            TextComponent::text(time.to_string())
                        ))
                        .await;
                }
                std::cmp::Ordering::Greater => {
                    let dist = format!("{distance:.1}");
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_WORLDBORDER_SET_GROW,
                            translation::bedrock::COMMANDS_WORLDBORDER_SETSLOWLY_GROW_SUCCESS,
                            TextComponent::text(dist),
                            TextComponent::text(old_dist),
                            TextComponent::text(time.to_string())
                        ))
                        .await;
                }
            }

            border.set_diameter(world, distance, Some(i64::from(time) * 1000));

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
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let mut border = world.worldborder.lock().await;

            let Vector2 { x, y } = Position2DArgumentConsumer.find_arg_default_name(args)?;

            sender
                .send_message(pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WORLDBORDER_CENTER_SUCCESS,
                    translation::bedrock::COMMANDS_WORLDBORDER_CENTER_SUCCESS,
                    TextComponent::text(format!("{x:.2}")),
                    TextComponent::text(format!("{y:.2}"))
                ))
                .await;
            border.set_center(world, x, y);
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
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let mut border = world.worldborder.lock().await;

            let damage_per_block = damage_per_block_consumer().find_arg_default_name(args)??;

            if (damage_per_block - border.damage_per_block).abs() < f32::EPSILON {
                return Err(CommandError::CommandFailed(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_WORLDBORDER_DAMAGE_AMOUNT_FAILED,
                        translation::bedrock::COMMANDS_WORLDBORDER_DAMAGE_AMOUNT_SUCCESS
                    ),
                ));
            }

            let damage = format!("{damage_per_block:.2}");
            let old_damage = format!("{:.2}", border.damage_per_block);
            sender
                .send_message(pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WORLDBORDER_DAMAGE_AMOUNT_SUCCESS,
                    translation::bedrock::COMMANDS_WORLDBORDER_DAMAGE_AMOUNT_SUCCESS,
                    TextComponent::text(damage),
                    TextComponent::text(old_damage)
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
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let mut border = world.worldborder.lock().await;

            let buffer = damage_buffer_consumer().find_arg_default_name(args)??;

            if (buffer - border.buffer).abs() < f32::EPSILON {
                return Err(CommandError::CommandFailed(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_WORLDBORDER_DAMAGE_BUFFER_FAILED,
                        translation::bedrock::COMMANDS_WORLDBORDER_DAMAGE_BUFFER_SUCCESS
                    ),
                ));
            }

            let buf = format!("{buffer:.2}");
            let old_buf = format!("{:.2}", border.buffer);
            sender
                .send_message(pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WORLDBORDER_DAMAGE_BUFFER_SUCCESS,
                    translation::bedrock::COMMANDS_WORLDBORDER_DAMAGE_BUFFER_SUCCESS,
                    TextComponent::text(buf),
                    TextComponent::text(old_buf)
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
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let mut border = world.worldborder.lock().await;

            let distance = warning_distance_consumer().find_arg_default_name(args)??;

            if distance == border.warning_blocks {
                return Err(CommandError::CommandFailed(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_WORLDBORDER_WARNING_DISTANCE_FAILED,
                        translation::bedrock::COMMANDS_WORLDBORDER_WARNING_DISTANCE_SUCCESS
                    ),
                ));
            }

            sender
                .send_message(pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WORLDBORDER_WARNING_DISTANCE_SUCCESS,
                    translation::bedrock::COMMANDS_WORLDBORDER_WARNING_DISTANCE_SUCCESS,
                    TextComponent::text(distance.to_string()),
                    TextComponent::text(border.warning_blocks.to_string())
                ))
                .await;
            border.set_warning_distance(world, distance);
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
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let mut border = world.worldborder.lock().await;

            let time = time_consumer().find_arg_default_name(args)??;

            if time == border.warning_time {
                return Err(CommandError::CommandFailed(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_WORLDBORDER_WARNING_TIME_FAILED,
                        translation::bedrock::COMMANDS_WORLDBORDER_WARNING_TIME_SUCCESS
                    ),
                ));
            }

            sender
                .send_message(pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WORLDBORDER_WARNING_TIME_SUCCESS,
                    translation::bedrock::COMMANDS_WORLDBORDER_WARNING_TIME_SUCCESS,
                    TextComponent::text(time.to_string()),
                    TextComponent::text(border.warning_time.to_string())
                ))
                .await;
            border.set_warning_delay(world, time);
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
