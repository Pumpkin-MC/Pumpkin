use std::sync::atomic::Ordering;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::time::TimeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;

const DESCRIPTION: &str = "Schedules an action or function to execute after a given duration.";
const PERMISSION: &str = "minecraft:command.schedule";

pub const ERROR_SAME_TICK: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SCHEDULE_SAME_TICK,
    translation::java::COMMANDS_SCHEDULE_SAME_TICK,
);

pub const ERROR_CANT_REMOVE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_SCHEDULE_CLEARED_FAILURE,
    translation::java::COMMANDS_SCHEDULE_CLEARED_FAILURE,
);

#[expect(dead_code)]
pub const ERROR_MACRO: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_SCHEDULE_MACRO,
    translation::java::COMMANDS_SCHEDULE_MACRO,
);

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

struct ScheduleSuggestionProvider;

impl SuggestionProvider for ScheduleSuggestionProvider {
    fn suggest(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        let server = context.server();
        for id in server.scheduled_functions.get_event_ids() {
            builder = builder.suggest(id);
        }
        builder.build()
    }
}

fn execute_schedule(context: &CommandContext, replace: bool) -> Result<i32, CommandSyntaxError> {
    let function_name = StringArgumentType::get(context, "function")?.to_string();
    let time = TimeArgumentType::get(context, "time")?;

    if time == 0 {
        return Err(ERROR_SAME_TICK.create_without_context());
    }

    let server = context.server();
    let current_tick = server.tick_count.load(Ordering::Relaxed) as u64;
    let tick_time = current_tick + (time as u64);
    let is_tag = function_name.starts_with('#');

    server.scheduled_functions.schedule(
        function_name.clone(),
        tick_time,
        function_name.clone(),
        is_tag,
        replace,
    );

    if is_tag {
        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_SCHEDULE_CREATED_TAG,
                translation::java::COMMANDS_SCHEDULE_CREATED_TAG,
                [
                    TextComponent::text(function_name),
                    TextComponent::text(time.to_string()),
                    TextComponent::text(tick_time.to_string()),
                ],
            ),
            true,
        );
    } else {
        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_SCHEDULE_CREATED_FUNCTION,
                translation::java::COMMANDS_SCHEDULE_CREATED_FUNCTION,
                [
                    TextComponent::text(function_name),
                    TextComponent::text(time.to_string()),
                    TextComponent::text(tick_time.to_string()),
                ],
            ),
            true,
        );
    }

    Ok((tick_time % (i32::MAX as u64)) as i32)
}

struct ScheduleExecutor {
    replace: bool,
}

impl CommandExecutor for ScheduleExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        execute_schedule(context, self.replace)
    }
}

struct ClearExecutor;

impl CommandExecutor for ClearExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let function_name = StringArgumentType::get(context, "function")?;
        let server = context.server();
        let count = server.scheduled_functions.remove(function_name);

        if count == 0 {
            return Err(ERROR_CANT_REMOVE
                .create_without_context(TextComponent::text(function_name.to_string())));
        }

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_SCHEDULE_CLEARED_SUCCESS,
                translation::java::COMMANDS_SCHEDULE_CLEARED_SUCCESS,
                [
                    TextComponent::text(count.to_string()),
                    TextComponent::text(function_name.to_string()),
                ],
            ),
            true,
        );

        Ok(count as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("schedule", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("function").then(
                    argument("function", StringArgumentType::SingleWord)
                        .suggests(FunctionSuggestionProvider)
                        .then(
                            argument("time", TimeArgumentType::any())
                                .executes(ScheduleExecutor { replace: true })
                                .then(
                                    literal("append").executes(ScheduleExecutor { replace: false }),
                                )
                                .then(
                                    literal("replace").executes(ScheduleExecutor { replace: true }),
                                ),
                        ),
                ),
            )
            .then(
                literal("clear").then(
                    argument("function", StringArgumentType::GreedyPhrase)
                        .suggests(ScheduleSuggestionProvider)
                        .executes(ClearExecutor),
                ),
            ),
    );
}
