use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult, Redirection};

const DESCRIPTION: &str = "Controls execution flow in functions and sets return values.";
const PERMISSION: &str = "minecraft:command.return";

struct ReturnValueExecutor;

impl CommandExecutor for ReturnValueExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let value = IntegerArgumentType::get(context, "value")?;
        Ok(value)
    }
}

struct ReturnFailExecutor;

impl CommandExecutor for ReturnFailExecutor {
    fn execute(&self, _context: &CommandContext) -> CommandExecutorResult {
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
        command("return", DESCRIPTION)
            .requires(PERMISSION)
            .then(argument("value", IntegerArgumentType::any()).executes(ReturnValueExecutor))
            .then(literal("fail").executes(ReturnFailExecutor))
            .then(literal("run").redirect(Redirection::Root)),
    );
}
