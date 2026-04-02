use crate::command::argument_builder::{ArgumentBuilder, command, literal};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::{Difficulty, PermissionLvl};

const DESCRIPTION: &str = "Query or change the difficulty of the world.";
const PERMISSION: &str = "minecraft:command.difficulty";

const FAILURE_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new("commands.difficulty.failure");

struct DifficultyQueryExecutor;

impl CommandExecutor for DifficultyQueryExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let difficulty = context.server().get_difficulty();

            context
                .source
                .send_feedback(
                    TextComponent::translate(
                        "commands.difficulty.query",
                        [TextComponent::translate(difficulty.translation_key(), [])],
                    ),
                    false,
                )
                .await;

            Ok(difficulty as i32)
        })
    }
}

struct DifficultySetExecutor(Difficulty);

impl CommandExecutor for DifficultySetExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let difficulty = self.0;
            let server = context.server();

            {
                let level_info = server.level_info.load();

                if level_info.difficulty == difficulty {
                    return Err(FAILURE_ERROR_TYPE
                        .create_without_context(TextComponent::text(difficulty.name())));
                }
            }

            server.set_difficulty(difficulty, true).await;

            context
                .source
                .send_feedback(
                    TextComponent::translate(
                        "commands.difficulty.success",
                        [TextComponent::translate(difficulty.translation_key(), [])],
                    ),
                    true,
                )
                .await;

            Ok(0)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry
        .register_permission(Permission::new(
            PERMISSION,
            DESCRIPTION,
            PermissionDefault::Op(PermissionLvl::Two),
        ))
        .expect("Permission should have registered successfully");

    dispatcher.register(
        command("difficulty", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("peaceful").executes(DifficultySetExecutor(Difficulty::Peaceful)))
            .then(literal("easy").executes(DifficultySetExecutor(Difficulty::Easy)))
            .then(literal("normal").executes(DifficultySetExecutor(Difficulty::Normal)))
            .then(literal("hard").executes(DifficultySetExecutor(Difficulty::Hard)))
            .executes(DifficultyQueryExecutor),
    );
}
