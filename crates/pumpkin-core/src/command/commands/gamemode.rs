use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::gamemode::GameModeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;

const DESCRIPTION: &str = "Change a player's gamemode.";
const PERMISSION: &str = "minecraft:command.gamemode";

const ERROR_NOT_PLAYER: CommandErrorType<0> = CommandErrorType::new(
    translation::java::PERMISSIONS_REQUIRES_PLAYER,
    translation::java::PERMISSIONS_REQUIRES_PLAYER,
);

struct GamemodeExecutor {
    is_self: bool,
}

impl CommandExecutor for GamemodeExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let gamemode = GameModeArgumentType::get(context, "gamemode")?;

        let targets = if self.is_self {
            let player = context
                .source
                .output
                .as_player()
                .ok_or_else(|| ERROR_NOT_PLAYER.create_without_context())?;
            vec![player]
        } else {
            EntityArgumentType::get_players(context, "target")?
        };

        let mut succeeded: i32 = 0;
        let server = context.source.server();

        for target in &targets {
            if target.gamemode.load() != gamemode {
                target.set_gamemode(gamemode);
                succeeded += 1;
                let gamemode_string = format!("{gamemode:?}").to_lowercase();
                let gamemode_string = format!("gameMode.{gamemode_string}");
                let gamemode_comp =
                    TextComponent::translate_cross(gamemode_string.clone(), gamemode_string, []);
                let is_self = context
                    .source
                    .output
                    .as_player()
                    .is_some_and(|p| Arc::ptr_eq(&p, target));
                if is_self {
                    target.send_system_message(&TextComponent::translate_cross(
                        translation::java::COMMANDS_GAMEMODE_SUCCESS_SELF,
                        translation::bedrock::COMMANDS_GAMEMODE_SUCCESS_SELF,
                        [gamemode_comp],
                    ));
                } else {
                    if server.level_info.load().game_rules.send_command_feedback {
                        target.send_system_message(&TextComponent::translate_cross(
                            translation::java::GAMEMODE_CHANGED,
                            translation::bedrock::GAMEMODE_CHANGED,
                            [gamemode_comp.clone()],
                        ));
                    }
                    context.source.send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_GAMEMODE_SUCCESS_OTHER,
                            translation::bedrock::COMMANDS_GAMEMODE_SUCCESS_OTHER,
                            [target.as_ref().get_display_name(), gamemode_comp],
                        ),
                        true,
                    );
                }
            }
        }

        Ok(succeeded)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("gamemode", DESCRIPTION).requires(PERMISSION).then(
            argument("gamemode", GameModeArgumentType)
                .executes(GamemodeExecutor { is_self: true })
                .then(
                    argument("target", EntityArgumentType::Players)
                        .executes(GamemodeExecutor { is_self: false }),
                ),
        ),
    );
}
