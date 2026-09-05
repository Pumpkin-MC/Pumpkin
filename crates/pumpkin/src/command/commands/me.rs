use pumpkin_data::world::EMOTE_COMMAND;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Broadcasts a narrative message about yourself.";
const PERMISSION: &str = "minecraft:command.me";

struct MeExecutor;

impl CommandExecutor for MeExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let msg = StringArgumentType::get(context, "action")?;
        let sender = &context.source;
        let server = sender.server();

        server.broadcast_message(
            &TextComponent::text(msg.to_string()),
            &context.source.display_name,
            EMOTE_COMMAND,
            None,
        );

        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Zero),
    ));

    dispatcher.register(
        command("me", DESCRIPTION)
            .requires(PERMISSION)
            .then(argument("action", StringArgumentType::GreedyPhrase).executes(MeExecutor)),
    );
}
