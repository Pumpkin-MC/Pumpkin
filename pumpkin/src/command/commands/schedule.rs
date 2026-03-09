use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{ConsumedArgs, FindArg, simple::SimpleArgConsumer, time::TimeArgumentConsumer},
    tree::{
        CommandTree,
        builder::{argument, literal},
    },
};

const NAMES: [&str; 1] = ["schedule"];
const DESCRIPTION: &str = "Delays the execution of a function.";
const ARG_FUNCTION: &str = "function";
const ARG_TIME: &str = "time";

struct FunctionExecutor;

impl CommandExecutor for FunctionExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let function = SimpleArgConsumer::find_arg(args, ARG_FUNCTION)?;
            let time = TimeArgumentConsumer::find_arg(args, ARG_TIME)?;

            if time == 0 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_SCHEDULE_SAME_TICK,
                        [],
                    ))
                    .await;
                return Err(CommandError::CommandFailed(TextComponent::text(
                    "Can't schedule for current tick",
                )));
            }

            // TODO: Implement schedule when function scheduler is available
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_SCHEDULE_CREATED_FUNCTION,
                    [
                        TextComponent::text(function.to_string()),
                        TextComponent::text(time.to_string()),
                    ],
                ))
                .await;

            Ok(time)
        })
    }
}

struct ClearExecutor;

impl CommandExecutor for ClearExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let function = SimpleArgConsumer::find_arg(args, ARG_FUNCTION)?;

            // TODO: Implement schedule clear when function scheduler is available
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_SCHEDULE_CLEARED_FAILURE,
                    [TextComponent::text(function.to_string())],
                ))
                .await;

            Err(CommandError::CommandFailed(TextComponent::text(format!(
                "No scheduled function found for {function}"
            ))))
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("function").then(
                argument(ARG_FUNCTION, SimpleArgConsumer).then(
                    argument(ARG_TIME, TimeArgumentConsumer)
                        .then(literal("append").execute(FunctionExecutor))
                        .then(literal("replace").execute(FunctionExecutor))
                        .execute(FunctionExecutor),
                ),
            ),
        )
        .then(
            literal("clear").then(argument(ARG_FUNCTION, SimpleArgConsumer).execute(ClearExecutor)),
        )
}
