use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::game_profile::GameProfileArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::data::SaveJSONConfiguration;

const DESCRIPTION: &str = "unbans a player";
const PERMISSION: &str = "minecraft:command.pardon";

const ERROR_PARDON_FAILED: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_PARDON_FAILED,
    translation::bedrock::COMMANDS_UNBAN_FAILED,
);

struct PardonExecutor;

impl CommandExecutor for PardonExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = GameProfileArgumentType::get(context, "targets")?;
        let server = context.source.server();
        let mut lock = server.data.banned_player_list.write().unwrap();
        let mut successes = 0;

        for target in &targets {
            let idx = lock
                .banned_players
                .iter()
                .position(|entry| entry.uuid == target.id);

            if let Some(idx) = idx {
                lock.banned_players.remove(idx);
                context.source.send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_PARDON_SUCCESS,
                        translation::bedrock::COMMANDS_UNBAN_SUCCESS,
                        [TextComponent::text(target.name.clone())],
                    ),
                    true,
                );
                successes += 1;
            }
        }

        if successes > 0 {
            lock.save();
            Ok(successes)
        } else {
            let err_target = targets
                .first()
                .map_or_else(String::new, |first_target| first_target.name.clone());
            Err(ERROR_PARDON_FAILED.create_without_context(TextComponent::text(err_target)))
        }
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("pardon", DESCRIPTION)
            .requires(PERMISSION)
            .then(argument("targets", GameProfileArgumentType).executes(PardonExecutor)),
    );
}
