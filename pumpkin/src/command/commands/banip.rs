use std::{net::IpAddr, str::FromStr};

use crate::{
    command::{
        CommandError, CommandExecutor, CommandResult, CommandSender,
        args::{Arg, ConsumedArgs, message::MsgArgConsumer, simple::SimpleArgConsumer},
        tree::{CommandTree, builder::argument},
    },
    data::{SaveJSONConfiguration, banlist_serializer::BannedIpEntry},
    net::DisconnectReason,
    server::Server,
};
use CommandError::InvalidConsumption;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["ban-ip"];
const DESCRIPTION: &str = "bans a player-ip";

const ARG_TARGET: &str = "ip";
const ARG_REASON: &str = "reason";

async fn parse_ip(target: &str, server: &Server) -> Option<IpAddr> {
    Some(match IpAddr::from_str(target) {
        Ok(ip) => ip,
        Err(_) => server
            .get_player_by_name(target)?
            .client
            .address()
            .await
            .ip(),
    })
}

struct NoReasonExecutor;

impl CommandExecutor for NoReasonExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(target)) = args.get(&ARG_TARGET) else {
                return Err(InvalidConsumption(Some(ARG_TARGET.into())));
            };

            ban_ip(sender, server, target, None).await
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
            let Some(Arg::Simple(target)) = args.get(&ARG_TARGET) else {
                return Err(InvalidConsumption(Some(ARG_TARGET.into())));
            };

            let Some(Arg::Msg(reason)) = args.get(ARG_REASON) else {
                return Err(InvalidConsumption(Some(ARG_REASON.into())));
            };

            ban_ip(sender, server, target, Some(reason.clone())).await
        })
    }
}

async fn ban_ip(
    sender: &CommandSender,
    server: &Server,
    target: &str,
    reason: Option<String>,
) -> Result<i32, CommandError> {
    let reason = reason.unwrap_or_else(|| "Banned by an operator.".to_string());

    let Some(target_ip) = parse_ip(target, server).await else {
        return Err(CommandError::CommandFailed(TextComponent::translate(
            translation::COMMANDS_BANIP_INVALID,
            [],
        )));
    };

    let mut banned_ips = server.data.banned_ip_list.write().await;

    if banned_ips.get_entry(&target_ip).is_some() {
        return Err(CommandError::CommandFailed(TextComponent::translate(
            translation::COMMANDS_BANIP_FAILED,
            [],
        )));
    }

    banned_ips.banned_ips.push(BannedIpEntry::new(
        target_ip,
        sender.to_string(),
        None,
        reason.clone(),
    ));

    banned_ips.save();
    drop(banned_ips);

    // Send messages
    let affected = server.get_players_by_ip(target_ip).await;
    let names = affected
        .iter()
        .map(|p| p.gameprofile.name.clone())
        .collect::<Vec<_>>()
        .join(" ");

    sender
        .send_message(TextComponent::translate(
            translation::COMMANDS_BANIP_SUCCESS,
            [
                TextComponent::text(target_ip.to_string()),
                TextComponent::text(reason),
            ],
        ))
        .await;

    sender
        .send_message(TextComponent::translate(
            translation::COMMANDS_BANIP_INFO,
            [
                TextComponent::text(affected.len().to_string()),
                TextComponent::text(names),
            ],
        ))
        .await;

    let count = affected.len();
    for target in affected {
        target
            .kick(
                DisconnectReason::Kicked,
                TextComponent::translate(translation::MULTIPLAYER_DISCONNECT_IP_BANNED, []),
            )
            .await;
    }

    Ok(count as i32)
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGET, SimpleArgConsumer)
            .execute(NoReasonExecutor)
            .then(argument(ARG_REASON, MsgArgConsumer).execute(ReasonExecutor)),
    )
}
