use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::{GameMode, PermissionLvl};

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::gamemode::GameModeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Change the default gamemode.";
const PERMISSION: &str = "minecraft:command.defaultgamemode";

pub struct DefaultGamemode {
    pub gamemode: GameMode,
}

struct DefaultGamemodeExecutor;

impl CommandExecutor for DefaultGamemodeExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let gamemode = GameModeArgumentType::get(context, "gamemode")?;
        let server = context.source.server();

        let mut successful_changes: i32 = 0;
        if server.basic_config.force_gamemode {
            for player in server.get_all_players() {
                if player.gamemode.load() != gamemode {
                    player.set_gamemode(gamemode);
                    successful_changes += 1;
                }
            }
        }

        let gamemode_string = format!("{gamemode:?}").to_lowercase();
        let gamemode_string = format!("gameMode.{gamemode_string}");

        context.source.send_feedback(
            TextComponent::translate_cross(
                pumpkin_data::translation::java::COMMANDS_DEFAULTGAMEMODE_SUCCESS,
                pumpkin_data::translation::bedrock::COMMANDS_DEFAULTGAMEMODE_SUCCESS,
                [TextComponent::translate_cross(
                    gamemode_string.clone(),
                    gamemode_string,
                    [],
                )],
            ),
            true,
        );

        server.defaultgamemode.lock().unwrap().gamemode = gamemode;

        Ok(successful_changes)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("defaultgamemode", DESCRIPTION)
            .requires(PERMISSION)
            .then(argument("gamemode", GameModeArgumentType).executes(DefaultGamemodeExecutor)),
    );
}
