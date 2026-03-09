use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{ConsumedArgs, FindArg, simple::SimpleArgConsumer},
    tree::{
        CommandTree,
        builder::{argument, literal},
    },
};

const NAMES: [&str; 1] = ["return"];
const DESCRIPTION: &str = "Returns a value from a function.";
const ARG_VALUE: &str = "value";

struct ValueExecutor;

impl CommandExecutor for ValueExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let value = SimpleArgConsumer::find_arg(args, ARG_VALUE)?;

            // TODO: Implement return when function execution context is available
            let parsed: i32 = value.parse().map_err(|_| {
                CommandError::CommandFailed(TextComponent::text(format!(
                    "Invalid return value: {value}"
                )))
            })?;

            sender
                .send_message(TextComponent::text(format!("Returned {parsed}")))
                .await;

            Ok(parsed)
        })
    }
}

struct FailExecutor;

impl CommandExecutor for FailExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            // TODO: Implement return fail when function execution context is available
            sender
                .send_message(TextComponent::text("Returned fail"))
                .await;

            Ok(0)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_VALUE, SimpleArgConsumer).execute(ValueExecutor))
        .then(literal("fail").execute(FailExecutor))
}
