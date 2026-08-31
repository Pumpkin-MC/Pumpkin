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
    CommandErrorType::new("commands.difficulty.failure", "commands.difficulty.failure");

struct DifficultyQueryExecutor;

impl CommandExecutor for DifficultyQueryExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let difficulty = context.server().get_difficulty();

        context.source.send_feedback(
            TextComponent::translate_cross(
                "commands.difficulty.query",
                "commands.difficulty.query",
                [TextComponent::translate_cross(
                    difficulty.translation_key(),
                    difficulty.translation_key(),
                    [],
                )],
            ),
            false,
        );

        Ok(difficulty as i32)
    }
}

struct DifficultySetExecutor(Difficulty);

impl CommandExecutor for DifficultySetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let difficulty = self.0;
        let server = context.server();

        {
            let level_info = server.level_info.load();

            if level_info.difficulty == difficulty {
                return Err(FAILURE_ERROR_TYPE
                    .create_without_context(TextComponent::text(difficulty.name())));
            }
        }

        server.set_difficulty(difficulty, true);

        context.source.send_feedback(
            TextComponent::translate_cross(
                pumpkin_data::translation::java::COMMANDS_DIFFICULTY_SUCCESS,
                pumpkin_data::translation::bedrock::COMMANDS_DIFFICULTY_SUCCESS,
                [TextComponent::translate_cross(
                    difficulty.translation_key(),
                    difficulty.translation_key(),
                    [],
                )],
            ),
            true,
        );

        Ok(0)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

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
