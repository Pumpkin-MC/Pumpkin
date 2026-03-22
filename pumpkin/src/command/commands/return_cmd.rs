use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{ConsumedArgs, FindArg, message::MsgArgConsumer, simple::SimpleArgConsumer},
    tree::{
        CommandTree,
        builder::{argument, literal},
    },
};

const NAMES: [&str; 1] = ["return"];
const DESCRIPTION: &str = "Returns a value from a function.";
const ARG_VALUE: &str = "value";
const ARG_COMMAND: &str = "command";

struct ValueExecutor;

impl CommandExecutor for ValueExecutor {
    fn execute<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let value = SimpleArgConsumer::find_arg(args, ARG_VALUE)?;

            // TODO: Implement return when function execution context is available
            let parsed: i32 = value.parse().map_err(|_| {
                CommandError::CommandFailed(TextComponent::text(format!(
                    "Expected integer, got: {value}"
                )))
            })?;

            Ok(parsed)
        })
    }
}

struct FailExecutor;

impl CommandExecutor for FailExecutor {
    fn execute<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            // TODO: Implement return fail when function execution context is available
            // In vanilla, /return fail causes the enclosing function to fail
            Err(CommandError::CommandFailed(TextComponent::text(
                "/return fail can only be used within a function",
            )))
        })
    }
}

struct RunExecutor;

impl CommandExecutor for RunExecutor {
    fn execute<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let _command = MsgArgConsumer::find_arg(args, ARG_COMMAND)?;

            // TODO: Implement return run when function execution context is available
            Err(CommandError::CommandFailed(TextComponent::text(
                "/return run can only be used within a function",
            )))
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        // Literals must come before argument to avoid parse ambiguity
        .then(literal("fail").execute(FailExecutor))
        .then(literal("run").then(argument(ARG_COMMAND, MsgArgConsumer).execute(RunExecutor)))
        .then(argument(ARG_VALUE, SimpleArgConsumer).execute(ValueExecutor))
}
