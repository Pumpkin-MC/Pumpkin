use std::{net::IpAddr, str::FromStr};

use crate::{
    command::{
        args::{arg_simple::SimpleArgConsumer, Arg, ConsumedArgs},
        tree::CommandTree,
        tree_builder::argument,
        CommandError, CommandExecutor, CommandSender,
    },
    data::{banned_ip_data::BANNED_IP_LIST, SaveJSONConfiguration},
};
use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["pardon-ip"];
const DESCRIPTION: &str = "unbans a ip";

const ARG_TARGET: &str = "ip";

struct PardonIpExecutor;

#[async_trait]
impl CommandExecutor for PardonIpExecutor {
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
            return Err(CommandError::GeneralCommandIssue(
                "Invalid IP address".to_string(),
            ));
        };

        let mut lock = BANNED_IP_LIST.write().await;

        if let Some(idx) = lock.banned_ips.iter().position(|entry| entry.ip == ip) {
            lock.banned_ips.remove(idx);
        } else {
            return Err(CommandError::GeneralCommandIssue(
                "Nothing changed. That IP isn't banned.".to_string(),
            ));
        }

        lock.save();

        sender
            .send_message(TextComponent::text(format!("Unbanned IP {target}")))
            .await;
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(argument(ARG_TARGET, SimpleArgConsumer).execute(PardonIpExecutor))
}
