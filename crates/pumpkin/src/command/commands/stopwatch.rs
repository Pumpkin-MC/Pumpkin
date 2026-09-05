use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::double::DoubleArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::world::stopwatches::{Stopwatch, Stopwatches};

const DESCRIPTION: &str = "Creates, queries, restarts, and removes global stopwatches.";
const PERMISSION: &str = "minecraft:command.stopwatch";

pub const ERROR_ALREADY_EXISTS: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_STOPWATCH_ALREADY_EXISTS,
    translation::java::COMMANDS_STOPWATCH_ALREADY_EXISTS,
);

pub const ERROR_DOES_NOT_EXIST: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_STOPWATCH_DOES_NOT_EXIST,
    translation::java::COMMANDS_STOPWATCH_DOES_NOT_EXIST,
);

struct StopwatchSuggestionProvider;

impl SuggestionProvider for StopwatchSuggestionProvider {
    fn suggest(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        let stopwatches = context
            .server()
            .stopwatches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for id in stopwatches.ids() {
            builder = builder.suggest(id);
        }
        builder.build()
    }
}

struct CreateExecutor;

impl CommandExecutor for CreateExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let id = context.get_argument::<Identifier>("id")?;
        let id_str = id.to_string();
        let mut stopwatches = context
            .server()
            .stopwatches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Stopwatches::current_time();
        if !stopwatches.add(id_str.clone(), Stopwatch::new(now)) {
            return Err(ERROR_ALREADY_EXISTS.create_without_context(TextComponent::text(id_str)));
        }

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_STOPWATCH_CREATE_SUCCESS,
                translation::java::COMMANDS_STOPWATCH_CREATE_SUCCESS,
                [TextComponent::text(id_str)],
            ),
            true,
        );
        Ok(1)
    }
}

struct QueryExecutor {
    has_scale: bool,
}

impl CommandExecutor for QueryExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let id = context.get_argument::<Identifier>("id")?;
        let id_str = id.to_string();
        let scale = if self.has_scale {
            DoubleArgumentType::get(context, "scale")?
        } else {
            1.0
        };

        let stopwatches = context
            .server()
            .stopwatches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(stopwatch) = stopwatches.get(&id_str) else {
            return Err(ERROR_DOES_NOT_EXIST.create_without_context(TextComponent::text(id_str)));
        };

        let now = Stopwatches::current_time();
        let elapsed_seconds = stopwatch.elapsed_seconds(now);
        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_STOPWATCH_QUERY,
                translation::java::COMMANDS_STOPWATCH_QUERY,
                [
                    TextComponent::text(id_str),
                    TextComponent::text(format!("{elapsed_seconds:.2}")),
                ],
            ),
            true,
        );
        Ok((elapsed_seconds * scale) as i32)
    }
}

struct RestartExecutor;

impl CommandExecutor for RestartExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let id = context.get_argument::<Identifier>("id")?;
        let id_str = id.to_string();
        let mut stopwatches = context
            .server()
            .stopwatches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Stopwatches::current_time();
        if !stopwatches.update(&id_str, |_| Stopwatch::new(now)) {
            return Err(ERROR_DOES_NOT_EXIST.create_without_context(TextComponent::text(id_str)));
        }

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_STOPWATCH_RESTART_SUCCESS,
                translation::java::COMMANDS_STOPWATCH_RESTART_SUCCESS,
                [TextComponent::text(id_str)],
            ),
            true,
        );
        Ok(1)
    }
}

struct RemoveExecutor;

impl CommandExecutor for RemoveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let id = context.get_argument::<Identifier>("id")?;
        let id_str = id.to_string();
        let mut stopwatches = context
            .server()
            .stopwatches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !stopwatches.remove(&id_str) {
            return Err(ERROR_DOES_NOT_EXIST.create_without_context(TextComponent::text(id_str)));
        }

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_STOPWATCH_REMOVE_SUCCESS,
                translation::java::COMMANDS_STOPWATCH_REMOVE_SUCCESS,
                [TextComponent::text(id_str)],
            ),
            true,
        );
        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let stopwatch_cmd = command("stopwatch", DESCRIPTION)
        .requires(PERMISSION)
        .then(
            literal("create").then(argument("id", IdentifierArgumentType).executes(CreateExecutor)),
        )
        .then(
            literal("query").then(
                argument("id", IdentifierArgumentType)
                    .suggests(StopwatchSuggestionProvider)
                    .executes(QueryExecutor { has_scale: false })
                    .then(
                        argument("scale", DoubleArgumentType::any())
                            .executes(QueryExecutor { has_scale: true }),
                    ),
            ),
        )
        .then(
            literal("restart").then(
                argument("id", IdentifierArgumentType)
                    .suggests(StopwatchSuggestionProvider)
                    .executes(RestartExecutor),
            ),
        )
        .then(
            literal("remove").then(
                argument("id", IdentifierArgumentType)
                    .suggests(StopwatchSuggestionProvider)
                    .executes(RemoveExecutor),
            ),
        );

    dispatcher.register(stopwatch_cmd);
}
