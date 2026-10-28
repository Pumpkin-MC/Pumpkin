use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{
    CommandExecutor, CommandExecutorResult, RedirectModifier, RedirectModifierResult, Redirection,
};
use crate::data::datapack::{return_from_function, return_run_from_function};
use std::sync::Arc;

const DESCRIPTION: &str = "Controls execution flow in functions and sets return values.";
const PERMISSION: &str = "minecraft:command.return";

struct ReturnValueExecutor;

impl CommandExecutor for ReturnValueExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let value = IntegerArgumentType::get(context, "value")?;
        return_from_function(value);
        Ok(value)
    }
}

struct ReturnFailExecutor;

impl CommandExecutor for ReturnFailExecutor {
    fn execute(&self, _context: &CommandContext) -> CommandExecutorResult {
        return_from_function(0);
        Ok(0)
    }
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "redirect modifiers must return RedirectModifierResult"
)]
fn return_run_modifier(context: &CommandContext) -> RedirectModifierResult {
    // The function returns whatever the redirected command produces for this line.
    return_run_from_function();
    Ok(vec![context.source.clone()])
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
            .then(literal("run").redirect_with_modifier(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(return_run_modifier)),
            )),
    );
}
