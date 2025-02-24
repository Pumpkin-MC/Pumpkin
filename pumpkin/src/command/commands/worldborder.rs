use async_trait::async_trait;
use pumpkin_util::{
    math::vector2::Vector2,
    text::{
        TextComponent,
        color::{Color, NamedColor},
    },
};

use crate::{
    command::{
        CommandError, CommandExecutor, CommandSender,
        args::{
            ConsumedArgs, DefaultNameArgConsumer, FindArgDefaultName,
            bounded_num::BoundedNumArgumentConsumer, position_2d::Position2DArgumentConsumer,
        },
        tree::CommandTree,
        tree::builder::{argument_default_name, literal},
    },
    server::Server,
};

const NAMES: [&str; 1] = ["worldborder"];

const DESCRIPTION: &str = "Worldborder command.";

const NOTHING_CHANGED_EXCEPTION: &str = "commands.worldborder.set.failed.nochange";

fn distance_consumer() -> BoundedNumArgumentConsumer<f64> {
    BoundedNumArgumentConsumer::new().min(0.0).name("distance")
}

fn time_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new().min(0).name("time")
}

fn damage_per_block_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new()
        .min(0.0)
        .name("damage_per_block")
}

fn damage_buffer_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new().min(0.0).name("buffer")
}

fn warning_distance_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new().min(0).name("distance")
}

struct GetExecutor;

#[async_trait]
impl CommandExecutor for GetExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let border = world.worldborder.lock().await;

        let diameter = border.new_diameter.round() as i32;
        sender
            .send_message(TextComponent::translate(
                "commands.worldborder.get",
                [TextComponent::text(diameter.to_string())],
            ))
            .await;
        Ok(())
    }
}

struct SetExecutor;

#[async_trait]
impl CommandExecutor for SetExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let mut border = world.worldborder.lock().await;

        let Ok(distance) = distance_consumer().find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text(format!(
                        "{} is out of bounds.",
                        distance_consumer().default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        if (distance - border.new_diameter).abs() < f64::EPSILON {
            sender
                .send_message(
                    TextComponent::translate(NOTHING_CHANGED_EXCEPTION, [])
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        let dist = format!("{distance:.1}");
        sender
            .send_message(TextComponent::translate(
                "commands.worldborder.set.immediate",
                [TextComponent::text(dist)],
            ))
            .await;
        border.set_diameter(world, distance, None).await;
        Ok(())
    }
}

struct SetTimeExecutor;

#[async_trait]
impl CommandExecutor for SetTimeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let mut border = world.worldborder.lock().await;

        let Ok(distance) = distance_consumer().find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text(format!(
                        "{} is out of bounds.",
                        distance_consumer().default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };
        let Ok(time) = time_consumer().find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text(format!(
                        "{} is out of bounds.",
                        time_consumer().default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        match distance.total_cmp(&border.new_diameter) {
            std::cmp::Ordering::Equal => {
                sender
                    .send_message(
                        TextComponent::translate(NOTHING_CHANGED_EXCEPTION, [])
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
            std::cmp::Ordering::Less => {
                let dist = format!("{distance:.1}");
                sender
                    .send_message(TextComponent::translate(
                        "commands.worldborder.set.shrink",
                        [
                            TextComponent::text(dist),
                            TextComponent::text(time.to_string()),
                        ],
                    ))
                    .await;
            }
            std::cmp::Ordering::Greater => {
                let dist = format!("{distance:.1}");
                sender
                    .send_message(TextComponent::translate(
                        "commands.worldborder.set.grow",
                        [
                            TextComponent::text(dist),
                            TextComponent::text(time.to_string()),
                        ],
                    ))
                    .await;
            }
        }

        border
            .set_diameter(world, distance, Some(i64::from(time) * 1000))
            .await;
        Ok(())
    }
}

struct AddExecutor;

#[async_trait]
impl CommandExecutor for AddExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let mut border = world.worldborder.lock().await;

        let Ok(distance) = distance_consumer().find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text(format!(
                        "{} is out of bounds.",
                        distance_consumer().default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        if distance == 0.0 {
            sender
                .send_message(
                    TextComponent::translate(NOTHING_CHANGED_EXCEPTION, [])
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        let distance = border.new_diameter + distance;

        let dist = format!("{distance:.1}");
        sender
            .send_message(TextComponent::translate(
                "commands.worldborder.set.immediate",
                [TextComponent::text(dist)],
            ))
            .await;
        border.set_diameter(world, distance, None).await;
        Ok(())
    }
}

struct AddTimeExecutor;

#[async_trait]
impl CommandExecutor for AddTimeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let mut border = world.worldborder.lock().await;

        let Ok(distance) = distance_consumer().find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text(format!(
                        "{} is out of bounds.",
                        distance_consumer().default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };
        let Ok(time) = time_consumer().find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text(format!(
                        "{} is out of bounds.",
                        time_consumer().default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        let distance = distance + border.new_diameter;

        match distance.total_cmp(&border.new_diameter) {
            std::cmp::Ordering::Equal => {
                sender
                    .send_message(
                        TextComponent::translate(NOTHING_CHANGED_EXCEPTION, [])
                            .color(Color::Named(NamedColor::Red)),
                    )
                    .await;
                return Ok(());
            }
            std::cmp::Ordering::Less => {
                let dist = format!("{distance:.1}");
                sender
                    .send_message(TextComponent::translate(
                        "commands.worldborder.set.shrink",
                        [
                            TextComponent::text(dist),
                            TextComponent::text(time.to_string()),
                        ],
                    ))
                    .await;
            }
            std::cmp::Ordering::Greater => {
                let dist = format!("{distance:.1}");
                sender
                    .send_message(TextComponent::translate(
                        "commands.worldborder.set.grow",
                        [
                            TextComponent::text(dist),
                            TextComponent::text(time.to_string()),
                        ],
                    ))
                    .await;
            }
        }

        border
            .set_diameter(world, distance, Some(i64::from(time) * 1000))
            .await;
        Ok(())
    }
}

struct CenterExecutor;

#[async_trait]
impl CommandExecutor for CenterExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let mut border = world.worldborder.lock().await;

        let Vector2 { x, z } = Position2DArgumentConsumer.find_arg_default_name(args)?;

        sender
            .send_message(TextComponent::translate(
                "commands.worldborder.center.success",
                [
                    TextComponent::text(format!("{x:.2}")),
                    TextComponent::text(format!("{z:.2}")),
                ],
            ))
            .await;
        border.set_center(world, x, z).await;
        Ok(())
    }
}

struct DamageAmountExecutor;

#[async_trait]
impl CommandExecutor for DamageAmountExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let mut border = world.worldborder.lock().await;

        let Ok(damage_per_block) = damage_per_block_consumer().find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text(format!(
                        "{} is out of bounds.",
                        damage_per_block_consumer().default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        if (damage_per_block - border.damage_per_block).abs() < f32::EPSILON {
            sender
                .send_message(
                    TextComponent::translate("commands.worldborder.damage.amount.failed", [])
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        let damage = format!("{damage_per_block:.2}");
        sender
            .send_message(TextComponent::translate(
                "commands.worldborder.damage.amount.success",
                [TextComponent::text(damage)],
            ))
            .await;
        border.damage_per_block = damage_per_block;
        Ok(())
    }
}

struct DamageBufferExecutor;

#[async_trait]
impl CommandExecutor for DamageBufferExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let mut border = world.worldborder.lock().await;

        let Ok(buffer) = damage_buffer_consumer().find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text(format!(
                        "{} is out of bounds.",
                        damage_buffer_consumer().default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        if (buffer - border.buffer).abs() < f32::EPSILON {
            sender
                .send_message(
                    TextComponent::translate("commands.worldborder.damage.buffer.failed", [])
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        let buf = format!("{buffer:.2}");
        sender
            .send_message(TextComponent::translate(
                "commands.worldborder.damage.buffer.success",
                [TextComponent::text(buf)],
            ))
            .await;
        border.buffer = buffer;
        Ok(())
    }
}

struct WarningDistanceExecutor;

#[async_trait]
impl CommandExecutor for WarningDistanceExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let mut border = world.worldborder.lock().await;

        let Ok(distance) = warning_distance_consumer().find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text(format!(
                        "{} is out of bounds.",
                        warning_distance_consumer().default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        if distance == border.warning_blocks {
            sender
                .send_message(
                    TextComponent::translate("commands.worldborder.warning.distance.failed", [])
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        sender
            .send_message(TextComponent::translate(
                "commands.worldborder.warning.distance.success",
                [TextComponent::text(distance.to_string())],
            ))
            .await;
        border.set_warning_distance(world, distance).await;
        Ok(())
    }
}

struct WarningTimeExecutor;

#[async_trait]
impl CommandExecutor for WarningTimeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let mut border = world.worldborder.lock().await;

        let Ok(time) = time_consumer().find_arg_default_name(args)? else {
            sender
                .send_message(
                    TextComponent::text(format!(
                        "{} is out of bounds.",
                        time_consumer().default_name()
                    ))
                    .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        if time == border.warning_time {
            sender
                .send_message(
                    TextComponent::translate("commands.worldborder.warning.time.failed", [])
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        }

        sender
            .send_message(TextComponent::translate(
                "commands.worldborder.warning.time.success",
                [TextComponent::text(time.to_string())],
            ))
            .await;
        border.set_warning_delay(world, time).await;
        Ok(())
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
