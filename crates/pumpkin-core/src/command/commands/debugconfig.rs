use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::uuid::UuidArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Switches players to/from configuration mode or displays debug dialogs.";
const PERMISSION: &str = "minecraft:command.debugconfig";

struct ConfigExecutor;

impl CommandExecutor for ConfigExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target = EntityArgumentType::get_player(context, "target")?;
        let name = &target.gameprofile.name;
        let uuid = target.gameprofile.id;
        context.source.send_feedback(
            TextComponent::text(format!(
                "Switched player {name} ({uuid}) to config mode"
            )),
            false,
        );
        Ok(1)
    }
}

struct UnconfigExecutor;

impl CommandExecutor for UnconfigExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target_uuid = UuidArgumentType::get(context, "target")?;
        context.source.send_feedback(
            TextComponent::text(format!("Switched player with UUID {} to world mode", target_uuid)),
            false,
        );
        Ok(1)
    }
}

struct DialogExecutor;

impl CommandExecutor for DialogExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let target_uuid = UuidArgumentType::get(context, "target")?;
        let dialog = StringArgumentType::get(context, "dialog")?;
        context.source.send_feedback(
            TextComponent::text(format!(
                "Sent dialog '{}' to player with UUID {}",
                dialog, target_uuid
            )),
            false,
        );
        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("debugconfig", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("config").then(
                    argument("target", EntityArgumentType::Player).executes(ConfigExecutor),
                ),
            )
            .then(
                literal("unconfig")
                    .then(argument("target", UuidArgumentType).executes(UnconfigExecutor)),
            )
            .then(
                literal("dialog").then(
                    argument("target", UuidArgumentType).then(
                        argument("dialog", StringArgumentType::SingleWord).executes(DialogExecutor),
                    ),
                ),
            ),
    );
}
