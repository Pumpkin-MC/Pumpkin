use std::{net::IpAddr, str::FromStr};

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{Arg, ConsumedArgs, simple::SimpleArgConsumer},
    tree::{CommandTree, builder::argument},
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

            if !server
                .banned_ip_storage
                .is_banned(ip)
                .await
                .unwrap_or(false)
            {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.pardonip.failed",
                    [],
                )));
            }
            if let Err(e) = server.banned_ip_storage.unban(ip).await {
                tracing::error!("Failed to pardon {ip}: {e}");
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.pardonip.failed",
                    [],
                )));
            }
            sender
                .send_message(TextComponent::translate(
                    "commands.pardonip.success",
                    [TextComponent::text(ip.to_string())],
                ))
                .await;
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, SimpleArgConsumer).execute(Executor))
}
