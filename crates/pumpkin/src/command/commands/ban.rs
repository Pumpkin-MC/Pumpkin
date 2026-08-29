use crate::{
    command::{
        CommandError, CommandExecutor, CommandResult, CommandSender,
        args::{
            Arg, ConsumedArgs,
            gameprofile::{GameProfileSuggestionMode, GameProfilesArgumentConsumer},
            message::MsgArgConsumer,
        },
        tree::{CommandTree, builder::argument},
    },
    data::{SaveJSONConfiguration, banlist_serializer::BannedPlayerEntry},
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
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::GameProfiles(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        ban_players(sender, server, targets.as_slice(), None)
    }
}

struct ReasonExecutor;

impl CommandExecutor for ReasonExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::GameProfiles(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        let Some(Arg::Msg(reason)) = args.get(ARG_REASON) else {
            return Err(InvalidConsumption(Some(ARG_REASON.into())));
        };

        ban_players(sender, server, targets.as_slice(), Some(reason))
    }
}

/// Returns the number of players successfully banned.
fn ban_players(
    sender: &CommandSender,
    server: &crate::server::Server,
    targets: &[GameProfile],
    reason: Option<&String>,
) -> Result<i32, CommandError> {
    let mut count: usize = 0;
    for target in targets {
        if ban_profile(sender, server, target, reason.cloned()) {
            count += 1;
        }
    }

    if count == 0 {
        Err(CommandError::CommandFailed(TextComponent::translate_cross(
            translation::java::COMMANDS_BAN_FAILED,
            translation::bedrock::COMMANDS_BAN_FAILED,
            [],
        )))
    } else {
        Ok(count as i32)
    }
}

/// Returns `true` if the player was successfully banned.
fn ban_profile(
    sender: &CommandSender,
    server: &crate::server::Server,
    profile: &GameProfile,
    reason: Option<String>,
) -> bool {
    let mut banned_players = server.data.banned_player_list.write().unwrap();

    let reason = reason.unwrap_or_else(|| "Banned by an operator.".to_string());

    if let Some(entry) = banned_players
        .banned_players
        .iter_mut()
        .find(|entry| entry.uuid == profile.id)
    {
        if entry.name != profile.name {
            entry.name.clone_from(&profile.name);
            banned_players.save();
        }
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
    sender.send_message(TextComponent::translate_cross(
        translation::java::COMMANDS_BAN_SUCCESS,
        translation::bedrock::COMMANDS_BAN_SUCCESS,
        [
            TextComponent::text(profile.name.clone()),
            TextComponent::text(reason),
        ],
    ));

    if let Some(player) = server.get_player_by_uuid(profile.id) {
        let kick_msg = TextComponent::translate_cross(
            translation::java::MULTIPLAYER_DISCONNECT_BANNED,
            translation::bedrock::DISCONNECTIONSCREEN_TITLE_BANNEDBYHOST,
            [],
        );
        player.kick(DisconnectReason::Kicked, &kick_msg);
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
