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
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::Simple(target)) = args.get(&ARG_TARGET) else {
            return Err(InvalidConsumption(Some(ARG_TARGET.into())));
        };

        let Ok(ip) = IpAddr::from_str(target) else {
            return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                pumpkin_data::translation::java::COMMANDS_PARDONIP_INVALID,
                pumpkin_data::translation::bedrock::COMMANDS_UNBANIP_INVALID,
                [],
            )));
        };

        let mut lock = server
            .data
            .banned_ip_list
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let result = lock
            .banned_ips
            .iter()
            .position(|entry| entry.ip == ip)
            .map_or_else(
                || {
                    Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        pumpkin_data::translation::java::COMMANDS_PARDONIP_FAILED,
                        pumpkin_data::translation::java::COMMANDS_PARDONIP_FAILED,
                        [],
                    )))
                },
                |idx| {
                    lock.banned_ips.remove(idx);
                    sender.send_message(TextComponent::translate_cross(
                        pumpkin_data::translation::java::COMMANDS_PARDONIP_SUCCESS,
                        pumpkin_data::translation::bedrock::COMMANDS_UNBANIP_SUCCESS,
                        [TextComponent::text(ip.to_string())],
                    ));
                    Ok(1)
                },
            );

        lock.save();

        result
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGET, SimpleArgConsumer).execute(Executor))
}
