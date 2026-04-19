use crate::command::CommandResult;
use crate::{
    command::{
        CommandError, CommandExecutor, CommandSender,
        args::{
            Arg, ConsumedArgs,
            gameprofile::{GameProfileSuggestionMode, GameProfilesArgumentConsumer},
            message::MsgArgConsumer,
        },
        tree::{CommandTree, builder::argument},
    },
    net::{DisconnectReason, GameProfile},
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
            let Some(Arg::GameProfiles(targets)) = args.get(&ARG_TARGET) else {
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
            let Some(Arg::GameProfiles(targets)) = args.get(&ARG_TARGET) else {
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
    targets: &[GameProfile],
    reason: Option<&String>,
) -> Result<i32, CommandError> {
    let mut count: usize = 0;
    for target in targets {
        if ban_profile(sender, server, target, reason.cloned()).await {
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
async fn ban_profile(
    sender: &CommandSender,
    server: &crate::server::Server,
    profile: &GameProfile,
    reason: Option<String>,
) -> bool {
    let reason = reason.unwrap_or_else(|| "Banned by an operator.".to_string());

    if let Ok(Some(entry)) = server.banned_player_storage.get(profile.id).await {
        if entry.name != profile.name {
            // Name changed; rewrite the ban entry preserving source/expires/reason.
            let _ = server
                .banned_player_storage
                .ban(
                    profile.id,
                    &profile.name,
                    entry.source,
                    entry.expires,
                    entry.reason,
                )
                .await;
        }
        return false;
    }

    if let Err(e) = server
        .banned_player_storage
        .ban(
            profile.id,
            &profile.name,
            sender.to_string(),
            None,
            reason.clone(),
        )
        .await
    {
        tracing::error!("Failed to ban {}: {e}", profile.name);
        return false;
    }

    // Send messages
    sender
        .send_message(TextComponent::translate(
            translation::COMMANDS_BAN_SUCCESS,
            [
                TextComponent::text(profile.name.clone()),
                TextComponent::text(reason),
            ],
        ))
        .await;

    if let Some(player) = server.get_player_by_uuid(profile.id) {
        player
            .kick(
                DisconnectReason::Kicked,
                TextComponent::translate(translation::MULTIPLAYER_DISCONNECT_BANNED, []),
            )
            .await;
    }

    true
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(
            ARG_TARGET,
            GameProfilesArgumentConsumer::new(GameProfileSuggestionMode::OnlinePlayers, true),
        )
        .execute(NoReasonExecutor)
        .then(argument(ARG_REASON, MsgArgConsumer).execute(ReasonExecutor)),
    )
}
