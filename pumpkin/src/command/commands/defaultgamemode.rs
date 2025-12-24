use crate::command::CommandResult;
use crate::command::args::gamemode::GamemodeArgumentConsumer;
use crate::command::args::{Arg, GetCloned};
use crate::command::dispatcher::CommandError::InvalidConsumption;
use crate::command::tree::builder::argument;
use crate::command::{CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree};
use pumpkin_util::GameMode;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["defaultgamemode"];

const DESCRIPTION: &str = "Change the default gamemode";

pub const ARG_GAMEMODE: &str = "gamemode";

pub struct DefaultGamemode {
    pub gamemode: GameMode,
}

struct DefaultGamemodeExecutor;

impl CommandExecutor for DefaultGamemodeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::GameMode(gamemode)) = args.get_cloned(&ARG_GAMEMODE) else {
                return Err(InvalidConsumption(Some(ARG_GAMEMODE.into())));
            };

            if server.basic_config.force_gamemode {
                for player in server.get_all_players().await {
                    player.set_gamemode(gamemode).await;
                }
            }

            let gamemode_string = format!("{gamemode:?}").to_lowercase();
            let gamemode_string = format!("gameMode.{gamemode_string}");

            sender
                .send_message(TextComponent::translate(
                    "commands.defaultgamemode.success",
                    [TextComponent::translate(gamemode_string, [])],
                ))
                .await;

            //Change the default gamemode (not in configuration.toml)
            server.defaultgamemode.lock().await.gamemode = gamemode;

            Ok(())
        })
    }
}

#[must_use]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_GAMEMODE, GamemodeArgumentConsumer).execute(DefaultGamemodeExecutor))
}
