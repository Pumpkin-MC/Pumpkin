use crate::command::args::GetCloned;
use crate::command::args::gamemode::GamemodeArgumentConsumer;

use crate::TextComponent;
use pumpkin_data::translation;

use crate::command::args::players::PlayersArgumentConsumer;

use crate::command::args::{Arg, ConsumedArgs};
use crate::command::dispatcher::CommandError::{InvalidConsumption, InvalidRequirement};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, require};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;

const NAMES: [&str; 1] = ["gamemode"];

const DESCRIPTION: &str = "Change a player's gamemode.";

const ARG_GAMEMODE: &str = "gamemode";
const ARG_TARGET: &str = "target";

struct TargetExecutor {
    is_self: bool,
}

impl CommandExecutor for TargetExecutor {
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

            let targets = if self.is_self {
                let Some(player) = sender.as_player() else {
                    return Err(InvalidRequirement);
                };
                &[player]
            } else {
                let Some(Arg::Players(targets)) = args.get(ARG_TARGET) else {
                    return Err(InvalidConsumption(Some(ARG_TARGET.into())));
                };
                targets.as_slice()
            };

            let mut succeeded: i32 = 0;
            for target in targets {
                if target.set_gamemode(gamemode).await {
                    succeeded += 1;
                    let gamemode_string = format!("{gamemode:?}").to_lowercase();
                    let gamemode_string = format!("gameMode.{gamemode_string}");
                    // Checking if the target was the sender of this command.
                    let gamemode_comp = TextComponent::translate_cross(
                        gamemode_string.clone(),
                        gamemode_string.clone(),
                        [],
                    );
                    if sender.as_player().as_ref() == Some(target) {
                        target
                            .send_system_message(&TextComponent::translate_cross(
                                translation::java::COMMANDS_GAMEMODE_SUCCESS_SELF,
                                translation::bedrock::COMMANDS_GAMEMODE_SUCCESS_SELF,
                                [gamemode_comp],
                            ))
                            .await;
                    } else {
                        if server.level_info.load().game_rules.send_command_feedback {
                            target
                                .send_system_message(&TextComponent::translate_cross(
                                    translation::java::GAMEMODE_CHANGED,
                                    translation::bedrock::GAMEMODE_CHANGED,
                                    [gamemode_comp.clone()],
                                ))
                                .await;
                        }
                        sender
                            .send_message(TextComponent::translate_cross(
                                translation::java::COMMANDS_GAMEMODE_SUCCESS_OTHER,
                                translation::bedrock::COMMANDS_GAMEMODE_SUCCESS_OTHER,
                                [target.get_display_name().await, gamemode_comp],
                            ))
                            .await;
                    }
                }
            }

            Ok(succeeded)
        })
    }
}

#[expect(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_GAMEMODE, GamemodeArgumentConsumer)
            .then(require(|sender| sender.is_player()).execute(TargetExecutor { is_self: true }))
            .then(
                argument(ARG_TARGET, PlayersArgumentConsumer)
                    .execute(TargetExecutor { is_self: false }),
            ),
    )
}
