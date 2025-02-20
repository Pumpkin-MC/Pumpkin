use async_trait::async_trait;

use crate::command::args::gamemode::GamemodeArgumentConsumer;
use crate::command::args::GetCloned;

use crate::TextComponent;

use crate::command::args::players::PlayersArgumentConsumer;

use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError;
use crate::command::dispatcher::CommandError::{InvalidConsumption, InvalidRequirement};
use crate::command::tree::builder::{argument, require};
use crate::command::tree::CommandTree;
use crate::command::CommandSender::Player;
use crate::command::{CommandExecutor, CommandSender};
use crate::server::Server;

const NAMES: [&str; 1] = ["gamemode"];

const DESCRIPTION: &str = "Change a player's gamemode.";

const ARG_GAMEMODE: &str = "gamemode";
const ARG_TARGET: &str = "target";

struct GamemodeTargetSelf;

#[async_trait]
impl CommandExecutor for GamemodeTargetSelf {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::GameMode(gamemode)) = args.get_cloned(&ARG_GAMEMODE) else {
            return Err(InvalidConsumption(Some(ARG_GAMEMODE.into())));
        };

        if let Player(target) = sender {
            if target.gamemode.load() != gamemode {
                target.set_gamemode(gamemode).await;
                let gamemode_string = format!("{gamemode:?}").to_lowercase();
                let gamemode_string = format!("gameMode.{gamemode_string}");
                target
                    .send_system_message(&TextComponent::translate(
                        "commands.gamemode.success.self",
                        [TextComponent::translate(gamemode_string, [])],
                    ))
                    .await;
            }
            Ok(())
        } else {
            Err(InvalidRequirement)
        }
    }
}

struct GamemodeTargetPlayer;

#[async_trait]
impl CommandExecutor for GamemodeTargetPlayer {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::GameMode(gamemode)) = args.get_cloned(&ARG_GAMEMODE) else {
            return Err(InvalidConsumption(Some(ARG_GAMEMODE.into())));
        };
        let Some(Arg::Players(targets)) = args.get(ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        let target_count = targets.len();

        for target in targets {
            if target.gamemode.load() != gamemode {
                target.set_gamemode(gamemode).await;
                let gamemode_string = format!("{gamemode:?}").to_lowercase();
                let gamemode_string = format!("gameMode.{gamemode_string}");
                target
                    .send_system_message(&TextComponent::translate(
                        "gameMode.changed",
                        [TextComponent::translate(gamemode_string.clone(), [])],
                    ))
                    .await;
                if target_count == 1 {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.gamemode.success.other",
                            [
                                TextComponent::text(target.gameprofile.name.clone()),
                                TextComponent::translate(gamemode_string, []),
                            ],
                        ))
                        .await;
                }
            }
        }

        Ok(())
    }
}

#[allow(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_GAMEMODE, GamemodeArgumentConsumer)
            .then(require(|sender| sender.is_player()).execute(GamemodeTargetSelf))
            .then(argument(ARG_TARGET, PlayersArgumentConsumer).execute(GamemodeTargetPlayer)),
    )
}
