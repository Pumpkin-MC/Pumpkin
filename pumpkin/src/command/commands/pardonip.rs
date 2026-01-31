use std::{net::IpAddr, str::FromStr};

use crate::{
    command::{
        CommandError, CommandExecutor, CommandResult, CommandSender,
        args::{Arg, ConsumedArgs, simple::SimpleArgConsumer},
        tree::{CommandTree, builder::argument},
    },
    data::SaveJSONConfiguration,
};
use CommandError::InvalidConsumption;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["pardon-ip"];
const DESCRIPTION: &str = "unbans a ip";

const ARG_TARGET: &str = "ip";

struct Executor;

impl CommandExecutor for Executor {
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

            let Ok(ip) = IpAddr::from_str(target) else {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.pardonip.invalid",
                    [],
                )));
            };

            let mut lock = server.data.banned_ip_list.write().await;

            let result = if let Some(idx) = lock.banned_ips.iter().position(|entry| entry.ip == ip)
            {
                lock.banned_ips.remove(idx);
                sender
                    .send_message(TextComponent::translate(
                        "commands.pardonip.success",
                        [TextComponent::text(ip.to_string())],
                    ))
                    .await;
                Ok(1)
            } else {
                Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.pardonip.failed",
                    [],
                )))
            };

            lock.save();

            result
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, SimpleArgConsumer).execute(Executor))
}
