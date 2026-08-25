use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;

const DESCRIPTION: &str = "Runs commands found in the corresponding function files.";
const PERMISSION: &str = "minecraft:command.function";

static ERROR_UNKNOWN_FUNCTION: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_FUNCTION_UNKNOWN,
    translation::java::ARGUMENTS_FUNCTION_UNKNOWN,
);

struct FunctionSuggestionProvider;

impl SuggestionProvider for FunctionSuggestionProvider {
    fn suggest<'a>(
        &'a self,
        context: &'a CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult<'a> {
        Box::pin(async move {
            let server = context.server();
            let function_names = server.datapack_manager.get_function_names().await;
            for name in function_names {
                builder = builder.suggest(name);
            }
            builder.build()
        })
    }
}

struct FunctionExecutor;

impl CommandExecutor for FunctionExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let name_str = StringArgumentType::get(context, "name")?;
            let server = context.server();

            let Ok(executed_count) = server
                .datapack_manager
                .execute_function(server, &context.source, name_str)
                .await
            else {
                return Err(ERROR_UNKNOWN_FUNCTION
                    .create_without_context(TextComponent::text(name_str.to_string())));
            };

            if name_str.starts_with('#') {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_FUNCTION_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_FUNCTION_SUCCESS_MULTIPLE,
                            [
                                TextComponent::text(executed_count.to_string()),
                                TextComponent::text(name_str.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            } else {
                context
                    .source
                    .send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_FUNCTION_SUCCESS_SINGLE,
                            translation::java::COMMANDS_FUNCTION_SUCCESS_SINGLE,
                            [
                                TextComponent::text(executed_count.to_string()),
                                TextComponent::text(name_str.to_string()),
                            ],
                        ),
                        true,
                    )
                    .await;
            }

            Ok(executed_count as i32)
        })
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
            argument("name", StringArgumentType::SingleWord)
                .suggests(FunctionSuggestionProvider)
                .executes(FunctionExecutor),
        ),
    );
}
