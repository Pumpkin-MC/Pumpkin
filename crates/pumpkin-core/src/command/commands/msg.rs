use pumpkin_data::world::{MSG_COMMAND_INCOMING, MSG_COMMAND_OUTGOING};
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;

const NAMES: [&str; 3] = ["msg", "tell", "w"];
const DESCRIPTION: &str = "Sends a private message to one or more players.";
const PERMISSION: &str = "minecraft:command.msg";

struct MsgExecutor;

impl CommandExecutor for MsgExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;
        let msg = StringArgumentType::get(context, "message")?;

        let sender_name = &context.source.display_name;
        let msg_text = TextComponent::text(msg.to_string());

        if let Some(player) = context.source.player_or_none() {
            for target in &targets {
                player.send_message(
                    &msg_text,
                    MSG_COMMAND_OUTGOING,
                    &player.get_display_name(),
                    Some(&target.get_display_name()),
                );
                target.send_message(
                    &msg_text,
                    MSG_COMMAND_INCOMING,
                    &player.get_display_name(),
                    Some(&target.get_display_name()),
                );
            }
        } else {
            for target in &targets {
                target.send_message(
                    &msg_text,
                    MSG_COMMAND_INCOMING,
                    sender_name,
                    Some(&target.get_display_name()),
                );
            }
        }

        Ok(targets.len() as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Zero),
    ));

    for name in NAMES {
        dispatcher.register(
            command(name, DESCRIPTION).requires(PERMISSION).then(
                argument("targets", EntityArgumentType::Players).then(
                    argument("message", StringArgumentType::GreedyPhrase).executes(MsgExecutor),
                ),
            ),
        );
    }
}
