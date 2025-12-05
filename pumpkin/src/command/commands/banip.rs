use std::{net::IpAddr, str::FromStr};

use crate::{
    command::{
        CommandError, CommandExecutor, CommandResult, CommandSender,
        args::{Arg, ConsumedArgs, message::MsgArgConsumer, simple::SimpleArgConsumer},
        tree::{CommandTree, builder::argument},
    },
    data::{
        SaveJSONConfiguration, banlist_serializer::BannedIpEntry, banned_ip_data::BANNED_IP_LIST,
    },
    net::DisconnectReason,
    server::Server,
};
use CommandError::InvalidConsumption;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["ban-ip"];
const DESCRIPTION: &str = "bans a player-ip";

const ARG_TARGET: &str = "ip";
const ARG_REASON: &str = "reason";

async fn parse_ip(target: &str, server: &Server) -> Option<IpAddr> {
    Some(match IpAddr::from_str(target) {
        Ok(ip) => ip,
        Err(_) => server
            .get_player_by_name(target)
            .await?
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

            ban_ip(sender, server, target, None).await;
            Ok(())
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

            ban_ip(sender, server, target, Some(reason.clone())).await;
            Ok(())
        })
    }
}

async fn ban_ip(sender: &CommandSender, server: &Server, target: &str, reason: Option<String>) {
    let reason = reason.unwrap_or_else(|| "Banned by an operator.".to_string());

    let Some(target_ip) = parse_ip(target, server).await else {
        sender
            .send_message(TextComponent::translate("commands.banip.invalid", []))
            .await;
        return;
    };

    let mut banned_ips = BANNED_IP_LIST.write().await;

    if banned_ips.get_entry(&target_ip).is_some() {
        sender
            .send_message(TextComponent::translate("commands.banip.failed", []))
            .await;
        return;
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
            "commands.banip.success",
            [
                TextComponent::text(target_ip.to_string()),
                TextComponent::text(reason),
            ],
        ))
        .await;

    sender
        .send_message(TextComponent::translate(
            "commands.banip.info",
            [
                TextComponent::text(affected.len().to_string()),
                TextComponent::text(names),
            ],
        ))
        .await;

    for target in affected {
        target
            .kick(
                DisconnectReason::Kicked,
                TextComponent::translate("multiplayer.disconnect.ip_banned", []),
            )
            .await;
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGET, SimpleArgConsumer)
            .execute(NoReasonExecutor)
            .then(argument(ARG_REASON, MsgArgConsumer).execute(ReasonExecutor)),
    )
}
