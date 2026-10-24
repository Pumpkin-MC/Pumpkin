use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::vec2::Vec2ArgumentType;
use crate::command::argument_types::core::double::DoubleArgumentType;
use crate::command::argument_types::core::float::FloatArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::time::TimeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Manages the world border.";
const PERMISSION: &str = "minecraft:command.worldborder";

const ERROR_SAME_CENTER: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WORLDBORDER_CENTER_FAILED,
    translation::java::COMMANDS_WORLDBORDER_CENTER_FAILED,
);
const ERROR_SAME_SIZE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WORLDBORDER_SET_FAILED_NOCHANGE,
    translation::java::COMMANDS_WORLDBORDER_SET_FAILED_NOCHANGE,
);
const ERROR_TOO_SMALL: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WORLDBORDER_SET_FAILED_SMALL,
    translation::java::COMMANDS_WORLDBORDER_SET_FAILED_SMALL,
);
const ERROR_TOO_BIG: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WORLDBORDER_SET_FAILED_BIG,
    translation::java::COMMANDS_WORLDBORDER_SET_FAILED_BIG,
);
const ERROR_TOO_FAR_OUT: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WORLDBORDER_SET_FAILED_FAR,
    translation::java::COMMANDS_WORLDBORDER_SET_FAILED_FAR,
);
const ERROR_SAME_WARNING_TIME: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WORLDBORDER_WARNING_TIME_FAILED,
    translation::java::COMMANDS_WORLDBORDER_WARNING_TIME_FAILED,
);
const ERROR_SAME_WARNING_DISTANCE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WORLDBORDER_WARNING_DISTANCE_FAILED,
    translation::java::COMMANDS_WORLDBORDER_WARNING_DISTANCE_FAILED,
);
const ERROR_SAME_DAMAGE_BUFFER: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WORLDBORDER_DAMAGE_BUFFER_FAILED,
    translation::java::COMMANDS_WORLDBORDER_DAMAGE_BUFFER_FAILED,
);
const ERROR_SAME_DAMAGE_AMOUNT: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WORLDBORDER_DAMAGE_AMOUNT_FAILED,
    translation::java::COMMANDS_WORLDBORDER_DAMAGE_AMOUNT_FAILED,
);

fn set_size(
    source: &CommandSource,
    distance: f64,
    time_in_ticks: i64,
) -> Result<i32, CommandSyntaxError> {
    let world = source.world().clone();
    let (current, diff) = {
        let mut border = world
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let current = border.new_diameter;

        if (current - distance).abs() < f64::EPSILON {
            return Err(ERROR_SAME_SIZE.create_without_context());
        }
        if distance < 1.0 {
            return Err(ERROR_TOO_SMALL.create_without_context());
        }
        if distance > 5.999_997e7 {
            return Err(ERROR_TOO_BIG.create_without_context());
        }

        let speed = (time_in_ticks > 0).then(|| time_in_ticks * 50); // ticks to milliseconds

        border.set_diameter(&world, distance, speed);
        (current, (distance - current) as i32)
    };

    let formatted_distance = format!("{distance:.1}");
    if time_in_ticks > 0 {
        let seconds_str = format!("{:.2}", time_in_ticks as f64 / 20.0);
        if distance > current {
            source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_WORLDBORDER_SET_GROW,
                    translation::java::COMMANDS_WORLDBORDER_SET_GROW,
                    [
                        TextComponent::text(formatted_distance),
                        TextComponent::text(seconds_str),
                    ],
                ),
                true,
            );
        } else {
            source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_WORLDBORDER_SET_SHRINK,
                    translation::java::COMMANDS_WORLDBORDER_SET_SHRINK,
                    [
                        TextComponent::text(formatted_distance),
                        TextComponent::text(seconds_str),
                    ],
                ),
                true,
            );
        }
    } else {
        source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_WORLDBORDER_SET_IMMEDIATE,
                translation::java::COMMANDS_WORLDBORDER_SET_IMMEDIATE,
                [TextComponent::text(formatted_distance)],
            ),
            true,
        );
    }

    Ok(diff)
}

fn set_center(source: &CommandSource, center: Vector2<f64>) -> Result<i32, CommandSyntaxError> {
    let world = source.world().clone();
    let mut border = world
        .worldborder
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    if (border.center_x - center.x).abs() < f64::EPSILON
        && (border.center_z - center.y).abs() < f64::EPSILON
    {
        return Err(ERROR_SAME_CENTER.create_without_context());
    }

    if center.x.abs() > 2.999_998_4e7 || center.y.abs() > 2.999_998_4e7 {
        return Err(ERROR_TOO_FAR_OUT.create_without_context());
    }

    border.set_center(&world, center.x, center.y);

    source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_WORLDBORDER_CENTER_SUCCESS,
            translation::java::COMMANDS_WORLDBORDER_CENTER_SUCCESS,
            [
                TextComponent::text(format!("{:.2}", center.x)),
                TextComponent::text(format!("{:.2}", center.y)),
            ],
        ),
        true,
    );

    Ok(0)
}

#[allow(clippy::unnecessary_wraps)]
fn get_size(source: &CommandSource) -> Result<i32, CommandSyntaxError> {
    let world = source.world().clone();
    let size = world
        .worldborder
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .new_diameter;

    source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_WORLDBORDER_GET,
            translation::java::COMMANDS_WORLDBORDER_GET,
            [TextComponent::text(format!("{size:.0}"))],
        ),
        false,
    );

    Ok((size + 0.5).floor() as i32)
}

fn set_damage_amount(
    source: &CommandSource,
    damage_per_block: f32,
) -> Result<i32, CommandSyntaxError> {
    let world = source.world().clone();
    let mut border = world
        .worldborder
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    if (border.damage_per_block - damage_per_block).abs() < f32::EPSILON {
        return Err(ERROR_SAME_DAMAGE_AMOUNT.create_without_context());
    }

    border.set_damage_per_block(damage_per_block);

    source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_WORLDBORDER_DAMAGE_AMOUNT_SUCCESS,
            translation::java::COMMANDS_WORLDBORDER_DAMAGE_AMOUNT_SUCCESS,
            [TextComponent::text(format!("{damage_per_block:.2}"))],
        ),
        true,
    );

    Ok(damage_per_block as i32)
}

fn set_damage_buffer(source: &CommandSource, distance: f32) -> Result<i32, CommandSyntaxError> {
    let world = source.world().clone();
    let mut border = world
        .worldborder
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    if (border.buffer - distance).abs() < f32::EPSILON {
        return Err(ERROR_SAME_DAMAGE_BUFFER.create_without_context());
    }

    border.set_damage_buffer(distance);

    source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_WORLDBORDER_DAMAGE_BUFFER_SUCCESS,
            translation::java::COMMANDS_WORLDBORDER_DAMAGE_BUFFER_SUCCESS,
            [TextComponent::text(format!("{distance:.2}"))],
        ),
        true,
    );

    Ok(distance as i32)
}

fn set_warning_distance(source: &CommandSource, distance: i32) -> Result<i32, CommandSyntaxError> {
    let world = source.world().clone();
    let mut border = world
        .worldborder
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    if border.warning_blocks == distance {
        return Err(ERROR_SAME_WARNING_DISTANCE.create_without_context());
    }

    border.set_warning_distance(&world, distance);

    source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_WORLDBORDER_WARNING_DISTANCE_SUCCESS,
            translation::java::COMMANDS_WORLDBORDER_WARNING_DISTANCE_SUCCESS,
            [TextComponent::text(distance.to_string())],
        ),
        true,
    );

    Ok(distance)
}

fn set_warning_time(source: &CommandSource, ticks: i32) -> Result<i32, CommandSyntaxError> {
    let world = source.world().clone();
    let mut border = world
        .worldborder
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    if border.warning_time == ticks {
        return Err(ERROR_SAME_WARNING_TIME.create_without_context());
    }

    border.set_warning_delay(&world, ticks);

    source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_WORLDBORDER_WARNING_TIME_SUCCESS,
            translation::java::COMMANDS_WORLDBORDER_WARNING_TIME_SUCCESS,
            [TextComponent::text(format!("{:.2}", ticks as f64 / 20.0))],
        ),
        true,
    );

    Ok(ticks)
}

struct SetSizeExecutor {
    is_add: bool,
    has_time: bool,
}

impl CommandExecutor for SetSizeExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let distance = DoubleArgumentType::get(context, "distance")?;
        let time = if self.has_time {
            i64::from(TimeArgumentType::get(context, "time")?)
        } else {
            0
        };

        let target_distance = if self.is_add {
            let current = context
                .source
                .world()
                .worldborder
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .new_diameter;
            current + distance
        } else {
            distance
        };

        set_size(&context.source, target_distance, time)
    }
}

struct CenterExecutor;

impl CommandExecutor for CenterExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let center = Vec2ArgumentType::get_vector2(context, "pos")?;
        set_center(&context.source, center)
    }
}

struct GetSizeExecutor;

impl CommandExecutor for GetSizeExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        get_size(&context.source)
    }
}

struct DamageAmountExecutor;

impl CommandExecutor for DamageAmountExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let damage = FloatArgumentType::get(context, "damagePerBlock")?;
        set_damage_amount(&context.source, damage)
    }
}

struct DamageBufferExecutor;

impl CommandExecutor for DamageBufferExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let distance = FloatArgumentType::get(context, "distance")?;
        set_damage_buffer(&context.source, distance)
    }
}

struct WarningDistanceExecutor;

impl CommandExecutor for WarningDistanceExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let distance = IntegerArgumentType::get(context, "distance")?;
        set_warning_distance(&context.source, distance)
    }
}

struct WarningTimeExecutor;

impl CommandExecutor for WarningTimeExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let time = TimeArgumentType::get(context, "time")?;
        set_warning_time(&context.source, time)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("worldborder", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("add").then(
                    argument(
                        "distance",
                        DoubleArgumentType::new(-5.999_997e7, 5.999_997e7),
                    )
                    .executes(SetSizeExecutor {
                        is_add: true,
                        has_time: false,
                    })
                    .then(
                        argument("time", TimeArgumentType::new(0)).executes(SetSizeExecutor {
                            is_add: true,
                            has_time: true,
                        }),
                    ),
                ),
            )
            .then(
                literal("set").then(
                    argument(
                        "distance",
                        DoubleArgumentType::new(-5.999_997e7, 5.999_997e7),
                    )
                    .executes(SetSizeExecutor {
                        is_add: false,
                        has_time: false,
                    })
                    .then(
                        argument("time", TimeArgumentType::new(0)).executes(SetSizeExecutor {
                            is_add: false,
                            has_time: true,
                        }),
                    ),
                ),
            )
            .then(
                literal("center")
                    .then(argument("pos", Vec2ArgumentType::Default).executes(CenterExecutor)),
            )
            .then(
                literal("damage")
                    .then(
                        literal("amount").then(
                            argument("damagePerBlock", FloatArgumentType::with_min(0.0))
                                .executes(DamageAmountExecutor),
                        ),
                    )
                    .then(
                        literal("buffer").then(
                            argument("distance", FloatArgumentType::with_min(0.0))
                                .executes(DamageBufferExecutor),
                        ),
                    ),
            )
            .then(literal("get").executes(GetSizeExecutor))
            .then(
                literal("warning")
                    .then(
                        literal("distance").then(
                            argument("distance", IntegerArgumentType::with_min(0))
                                .executes(WarningDistanceExecutor),
                        ),
                    )
                    .then(literal("time").then(
                        argument("time", TimeArgumentType::new(0)).executes(WarningTimeExecutor),
                    )),
            ),
    );
}
