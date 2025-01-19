use crate::{
    command::{
        args::{
            arg_message::MsgArgConsumer, arg_players::PlayersArgumentConsumer, Arg, ConsumedArgs,
        },
        tree::CommandTree,
        tree_builder::argument,
        CommandError, CommandExecutor, CommandSender,
    },
    data::{
        banlist_serializer::BannedPlayerEntry, banned_player_data::BANNED_PLAYER_LIST,
        SaveJSONConfiguration,
    },
    entity::player::Player,
};
use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["ban"];
const DESCRIPTION: &str = "bans a player";

const ARG_TARGET: &str = "player";
const ARG_REASON: &str = "reason";

struct BanExecutorNoReason;

#[async_trait]
impl CommandExecutor for BanExecutorNoReason {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        ban_player(sender, &targets[0], "Banned by an operator.").await
    }
}

struct BanExecutorReason;

#[async_trait]
impl CommandExecutor for BanExecutorReason {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        let Some(Arg::Msg(reason)) = args.get(ARG_REASON) else {
            return Err(InvalidConsumption(Some(ARG_REASON.into())));
        };

        ban_player(sender, &targets[0], reason).await
    }
}

async fn ban_player(
    sender: &CommandSender<'_>,
    player: &Player,
    reason: &str,
) -> Result<(), CommandError> {
    let mut banned_players = BANNED_PLAYER_LIST.write().await;
    let profile = &player.gameprofile;

    if banned_players.get_entry(&player.gameprofile).is_some() {
        return Err(CommandError::GeneralCommandIssue(
            "Nothing changed. The player is already banned".to_string(),
        ));
    }

    let source = match sender {
        CommandSender::Player(player) => &player.gameprofile.name,
        _ => "Server",
    };

    banned_players.banned_players.push(BannedPlayerEntry::new(
        profile,
        source.to_string(),
        None,
        reason.to_string(),
    ));

    banned_players.save();
    drop(banned_players);

    // Send messages
    let player_name = &player.gameprofile.name;
    sender
        .send_message(TextComponent::text(format!(
            "Banned {player_name}: {reason}"
        )))
        .await;

    player
        .kick(TextComponent::text("You are banned from this server"))
        .await;

    Ok(())
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        argument(ARG_TARGET, PlayersArgumentConsumer)
            .execute(BanExecutorNoReason)
            .with_child(argument(ARG_REASON, MsgArgConsumer).execute(BanExecutorReason)),
    )
}
