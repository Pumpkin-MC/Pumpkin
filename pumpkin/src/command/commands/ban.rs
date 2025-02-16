use crate::{
    command::{
        args::{message::MsgArgConsumer, players::PlayersArgumentConsumer, Arg, ConsumedArgs},
        tree::{builder::argument, CommandTree},
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

struct BanNoReasonExecutor;

#[async_trait]
impl CommandExecutor for BanNoReasonExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Players(targets)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        ban_player(sender, &targets[0], None).await;
        Ok(())
    }
}

struct BanReasonExecutor;

#[async_trait]
impl CommandExecutor for BanReasonExecutor {
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

        ban_player(sender, &targets[0], Some(reason.to_string())).await;
        Ok(())
    }
}

async fn ban_player(sender: &CommandSender<'_>, player: &Player, reason: Option<String>) {
    let mut banned_players = BANNED_PLAYER_LIST.write().await;

    let reason = reason.unwrap_or_else(|| "Banned by an operator.".to_string());
    let profile = &player.gameprofile;

    if banned_players.get_entry(&player.gameprofile).is_some() {
        sender
            .send_message(TextComponent::translate("commands.ban.failed", []))
            .await;
        return;
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
            "commands.ban.success",
            [
                TextComponent::text(player.gameprofile.name.clone()),
                TextComponent::text(reason),
            ],
        ))
        .await;

    player
        .kick(TextComponent::translate(
            "multiplayer.disconnect.banned",
            [],
        ))
        .await;
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGET, PlayersArgumentConsumer)
            .execute(BanNoReasonExecutor)
            .then(argument(ARG_REASON, MsgArgConsumer).execute(BanReasonExecutor)),
    )
}
