use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};

const DESCRIPTION: &str = "Stops the execution of a function and sets its return value.";
const PERMISSION: &str = "minecraft:command.return";

const ARG_VALUE: &str = "value";
const ARG_COMMAND: &str = "command";

static COMMAND_FAILED: CommandErrorType<0> =
    CommandErrorType::new("command.failed", "command.failed");

struct ReturnValueExecutor;

impl CommandExecutor for ReturnValueExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let value = IntegerArgumentType::get(context, ARG_VALUE)?;
            Ok(value)
        })
    }
}

struct ReturnFailExecutor;

impl CommandExecutor for ReturnFailExecutor {
    fn execute<'a>(&'a self, _context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move { Err(COMMAND_FAILED.create_without_context_args_slice(&[])) })
    }
}

struct ReturnRunExecutor;

impl CommandExecutor for ReturnRunExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let command_str = StringArgumentType::get(context, ARG_COMMAND)?;
            let dispatcher = context.server().command_dispatcher.read().await;
            let result = dispatcher
                .execute_input(command_str, &context.source)
                .await?;
            Ok(result)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("return", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                argument(ARG_VALUE, IntegerArgumentType::any()).executes(ReturnValueExecutor),
            )
            .then(literal("fail").executes(ReturnFailExecutor))
            .then(
                literal("run").then(
                    argument(ARG_COMMAND, StringArgumentType::GreedyPhrase)
                        .executes(ReturnRunExecutor),
                ),
            ),
    );
}
