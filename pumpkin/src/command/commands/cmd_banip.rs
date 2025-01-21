use std::{net::IpAddr, str::FromStr};

use crate::{
    command::{
        args::{arg_message::MsgArgConsumer, arg_simple::SimpleArgConsumer, Arg, ConsumedArgs},
        tree::CommandTree,
        tree_builder::argument,
        CommandError, CommandExecutor, CommandSender,
    },
    data::{
        banlist_serializer::BannedIpEntry, banned_ip_data::BANNED_IP_LIST, SaveJSONConfiguration,
    },
    server::Server,
};
use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["ban-ip"];
const DESCRIPTION: &str = "bans a player-ip";

const ARG_TARGET: &str = "ip";
const ARG_REASON: &str = "reason";

async fn parse_ip(target: &str, server: &Server) -> Result<IpAddr, CommandError> {
    Ok(match IpAddr::from_str(target) {
        Ok(ip) => ip,
        Err(_) => server
            .get_player_by_name(target)
            .await
            .ok_or(CommandError::GeneralCommandIssue(
                "Invalid IP address or unknown player".to_string(),
            ))?
            .client
            .address
            .lock()
            .await
            .ip(),
    })
}

struct BanIpNoReasonExecutor;

#[async_trait]
impl CommandExecutor for BanIpNoReasonExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(target)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        let ip = parse_ip(target, server).await?;

        ban_ip(sender, server, ip, "Banned by an operator.").await
    }
}

struct BanIpReasonExecutor;

#[async_trait]
impl CommandExecutor for BanIpReasonExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(target)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        let ip = parse_ip(target, server).await?;

        let Some(Arg::Msg(reason)) = args.get(ARG_REASON) else {
            return Err(InvalidConsumption(Some(ARG_REASON.into())));
        };

        ban_ip(sender, server, ip, reason).await
    }
}

async fn ban_ip(
    sender: &CommandSender<'_>,
    server: &Server,
    target_ip: IpAddr,
    reason: &str,
) -> Result<(), CommandError> {
    let mut banned_ips = BANNED_IP_LIST.write().await;

    if banned_ips.get_entry(&target_ip).is_some() {
        return Err(CommandError::GeneralCommandIssue(
            "Nothing changed. That IP is already banned".to_string(),
        ));
    }

    banned_ips.banned_ips.push(BannedIpEntry::new(
        target_ip,
        sender.to_string(),
        None,
        reason.to_string(),
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
        .send_message(TextComponent::text(format!("Banned IP: {reason}")))
        .await;

    sender
        .send_message(TextComponent::text(format!(
            "This ban affects {} player(s): {}",
            affected.len(),
            names
        )))
        .await;

    for target in affected {
        target
            .kick(TextComponent::text("You are IP banned from this server"))
            .await;
    }

    Ok(())
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).with_child(
        argument(ARG_TARGET, SimpleArgConsumer)
            .execute(BanIpNoReasonExecutor)
            .with_child(argument(ARG_REASON, MsgArgConsumer).execute(BanIpReasonExecutor)),
    )
}
