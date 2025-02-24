use std::{net::IpAddr, str::FromStr};

use crate::{
    command::{
        CommandError, CommandExecutor, CommandSender,
        args::{Arg, ConsumedArgs, simple::SimpleArgConsumer},
        tree::CommandTree,
        tree::builder::argument,
    },
    data::{SaveJSONConfiguration, banned_ip_data::BANNED_IP_LIST},
};
use CommandError::InvalidConsumption;
use async_trait::async_trait;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["pardon-ip"];
const DESCRIPTION: &str = "unbans a ip";

const ARG_TARGET: &str = "ip";

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(target)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        let Ok(ip) = IpAddr::from_str(target) else {
            sender
                .send_message(TextComponent::translate("commands.pardonip.invalid", []))
                .await;
            return Ok(());
        };

        let mut lock = BANNED_IP_LIST.write().await;

        if let Some(idx) = lock.banned_ips.iter().position(|entry| entry.ip == ip) {
            lock.banned_ips.remove(idx);
        } else {
            sender
                .send_message(TextComponent::translate("commands.pardonip.failed", []))
                .await;
            return Ok(());
        }

        lock.save();

        sender
            .send_message(TextComponent::translate(
                "commands.pardonip.success",
                [TextComponent::text(ip.to_string())],
            ))
            .await;
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, SimpleArgConsumer).execute(Executor))
}
