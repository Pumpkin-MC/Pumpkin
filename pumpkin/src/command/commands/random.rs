use pumpkin_data::translation;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::range::IntRangeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Draws a random value.";

const PERMISSION: &str = "minecraft:command.random";

const ARG_RANGE: &str = "range";

/// The largest allowed range span, matching Vanilla (`i32::MAX - 1`).
const MAX_RANGE_SPAN: i64 = (i32::MAX - 1) as i64;

const RANGE_TOO_LARGE_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_RANDOM_ERROR_RANGE_TOO_LARGE,
    translation::java::COMMANDS_RANDOM_ERROR_RANGE_TOO_LARGE,
);

const RANGE_TOO_SMALL_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_RANDOM_ERROR_RANGE_TOO_SMALL,
    translation::java::COMMANDS_RANDOM_ERROR_RANGE_TOO_SMALL,
);

struct RandomExecutor {
    /// Whether the result is announced to every player (`roll`) instead of
    /// only the command source (`value`).
    roll: bool,
}

impl CommandExecutor for RandomExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let bounds = IntRangeArgumentType::get(context, ARG_RANGE)?;
            let min = bounds.min().unwrap_or(i32::MIN);
            let max = bounds.max().unwrap_or(i32::MAX);

            let span = i64::from(max) - i64::from(min);
            if span == 0 {
                return Err(RANGE_TOO_SMALL_ERROR_TYPE.create_without_context());
            }
            if span >= MAX_RANGE_SPAN {
                return Err(RANGE_TOO_LARGE_ERROR_TYPE.create_without_context());
            }

            let result = rand::random_range(min..=max);

            if self.roll {
                let msg = TextComponent::translate_cross(
                    translation::java::COMMANDS_RANDOM_ROLL,
                    translation::java::COMMANDS_RANDOM_ROLL,
                    [
                        context.source.display_name.clone(),
                        TextComponent::text(result.to_string()),
                        TextComponent::text(min.to_string()),
                        TextComponent::text(max.to_string()),
                    ],
                );
                for player in context.server().get_all_players() {
                    player.send_system_message(&msg).await;
                }
                // Also show the roll on the console (or another non-player source).
                if context.source.player_or_none().is_none() {
                    context.source.send_message(msg).await;
                }
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_RANDOM_SAMPLE_SUCCESS,
                            translation::java::COMMANDS_RANDOM_SAMPLE_SUCCESS,
                            [TextComponent::text(result.to_string())],
                        ),
                        false,
                    )
                    .await;
            }

            Ok(result)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        // Vanilla allows every player to use `/random value` and `/random roll`.
        PermissionDefault::Allow,
    ));

    dispatcher.register(
        command("random", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("value").then(
                argument(ARG_RANGE, IntRangeArgumentType).executes(RandomExecutor { roll: false }),
            ))
            .then(literal("roll").then(
                argument(ARG_RANGE, IntRangeArgumentType).executes(RandomExecutor { roll: true }),
            )),
    );
}
