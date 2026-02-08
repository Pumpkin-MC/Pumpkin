use std::sync::Arc;

use crate::command::CommandResult;
use crate::entity::EntityBase;
use crate::{
    command::{
        CommandError, CommandExecutor, CommandSender,
        args::{Arg, ConsumedArgs, message::MsgArgConsumer, players::PlayersArgumentConsumer},
        tree::{CommandTree, builder::argument},
    },
    data::{SaveJSONConfiguration, banlist_serializer::BannedPlayerEntry},
    entity::player::Player,
    net::DisconnectReason,
};
use CommandError::InvalidConsumption;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["ban"];
const DESCRIPTION: &str = "bans a player";

const ARG_TARGET: &str = "player";
const ARG_REASON: &str = "reason";

struct NoReasonExecutor;

impl CommandExecutor for NoReasonExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
                return Err(InvalidConsumption(Some(ARG_TARGET.into())));
            };

            ban_players(sender, server, targets.as_slice(), None).await
        })
    }
}

struct ReasonExecutor;

impl CommandExecutor for ReasonExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
                return Err(InvalidConsumption(Some(ARG_TARGET.into())));
            };

            let Some(Arg::Msg(reason)) = args.get(ARG_REASON) else {
                return Err(InvalidConsumption(Some(ARG_REASON.into())));
            };

            ban_players(sender, server, targets.as_slice(), Some(reason)).await
        })
    }
}

/// Returns the number of players successfully banned.
async fn ban_players(
    sender: &CommandSender,
    server: &crate::server::Server,
    targets: &[Arc<Player>],
    reason: Option<&String>,
) -> Result<i32, CommandError> {
    let mut count: usize = 0;
    for target in targets {
        if ban_player(sender, server, target, reason.cloned()).await {
            count += 1;
        }
    }

    if count == 0 {
        Err(CommandError::CommandFailed(TextComponent::translate(
            translation::COMMANDS_BAN_FAILED,
            [],
        )))
    } else {
        Ok(count as i32)
    }
}

/// Returns `true` if the player was successfully banned.
async fn ban_player(
    sender: &CommandSender,
    server: &crate::server::Server,
    player: &Player,
    reason: Option<String>,
) -> bool {
    let mut banned_players = server.data.banned_player_list.write().await;

    let reason = reason.unwrap_or_else(|| "Banned by an operator.".to_string());
    let profile = &player.gameprofile;

    if banned_players.get_entry(&player.gameprofile).is_some() {
        return false;
    }

    banned_players.banned_players.push(BannedPlayerEntry::new(
        profile,
        sender.to_string(),
        None,
        reason.clone(),
    ));

    banned_players.save();
    drop(banned_players);

    // Send messages
    sender
        .send_message(TextComponent::translate(
            translation::COMMANDS_BAN_SUCCESS,
            [player.get_display_name().await, TextComponent::text(reason)],
        ))
        .await;

    player
        .kick(
            DisconnectReason::Kicked,
            TextComponent::translate(translation::MULTIPLAYER_DISCONNECT_BANNED, []),
        )
        .await;

    true
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGET, PlayersArgumentConsumer)
            .execute(NoReasonExecutor)
            .then(argument(ARG_REASON, MsgArgConsumer).execute(ReasonExecutor)),
    )
}
