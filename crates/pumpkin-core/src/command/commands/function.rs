use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::function::FunctionArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::{CommandErrorType, LiteralCommandErrorType};
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::data::datapack::FunctionCallError;
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;

const DESCRIPTION: &str = "Runs commands found in the corresponding function files.";
const PERMISSION: &str = "minecraft:command.function";

static ERROR_UNKNOWN_FUNCTION: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_FUNCTION_UNKNOWN,
    translation::java::ARGUMENTS_FUNCTION_UNKNOWN,
);

/// Internal queue/execution failures are not user input errors; they are
/// logged with detail and surfaced as a generic internal error instead of the
/// "unknown function" message.
static ERROR_FUNCTION_INTERNAL: LiteralCommandErrorType =
    LiteralCommandErrorType::new("Internal error while running function");

struct FunctionSuggestionProvider;

impl SuggestionProvider for FunctionSuggestionProvider {
    fn suggest(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        let server = context.server();
        let function_names = server.datapack_manager.get_function_names();
        for name in function_names {
            builder = builder.suggest(name);
        }
        builder.build()
    }
}

struct FunctionExecutor;

impl CommandExecutor for FunctionExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let name_str = FunctionArgumentType::get(context, "name")?;
        let server = context.server();

        let executed_count = match server
            .datapack_manager
            .execute_function(server, &context.source, name_str)
        {
            Ok(executed_count) => executed_count,
            Err(FunctionCallError::Unknown(_)) => {
                return Err(ERROR_UNKNOWN_FUNCTION
                    .create_without_context(TextComponent::text(name_str.to_string())));
            }
            Err(FunctionCallError::Internal(message)) => {
                tracing::error!("Internal error running function {name_str}: {message}");
                return Err(ERROR_FUNCTION_INTERNAL.create_without_context());
            }
        };

        if name_str.starts_with('#') {
            context.source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_FUNCTION_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_FUNCTION_SUCCESS_MULTIPLE,
                    [
                        TextComponent::text(executed_count.to_string()),
                        TextComponent::text(name_str.to_string()),
                    ],
                ),
                true,
            );
        } else {
            context.source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_FUNCTION_SUCCESS_SINGLE,
                    translation::java::COMMANDS_FUNCTION_SUCCESS_SINGLE,
                    [
                        TextComponent::text(executed_count.to_string()),
                        TextComponent::text(name_str.to_string()),
                    ],
                ),
                true,
            );
        }

        // The count is saturated rather than truncated: a saturated feedback
        // beats a wrapped negative one for absurdly large queues.
        Ok(i32::try_from(executed_count).unwrap_or(i32::MAX))
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("function", DESCRIPTION).requires(PERMISSION).then(
            argument("name", FunctionArgumentType)
                .suggests(FunctionSuggestionProvider)
                .executes(FunctionExecutor),
        ),
    );
}
