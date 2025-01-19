use crate::{
    command::{
        args::{arg_simple::SimpleArgConsumer, Arg, ConsumedArgs},
        tree::CommandTree,
        tree_builder::argument,
        CommandError, CommandExecutor, CommandSender,
    },
    data::{banned_ip_data::BANNED_IP_LIST, banned_player_data::BANNED_PLAYER_LIST},
};
use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["banlist"];
const DESCRIPTION: &str = "shows the banlist";

const ARG_LIST_TYPE: &str = "ips|players";

struct BanListExecutor;

#[async_trait]
impl CommandExecutor for BanListExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Simple(list_type)) = args.get(&ARG_LIST_TYPE) else {
            return Err(InvalidConsumption(Some(ARG_LIST_TYPE.into())));
        };

        match *list_type {
            "ips" => {
                let lock = &BANNED_IP_LIST.read().await;
                sender
                    .send_message(TextComponent::text(format!(
                        "There are {} ban(s):",
                        lock.banned_ips.len()
                    )))
                    .await;
                for ip in &lock.banned_ips {
                    sender
                        .send_message(TextComponent::text(format!(
                            "{} was banned by {}: {}",
                            ip.ip, ip.source, ip.reason
                        )))
                        .await;
                }
            }
            "players" => {
                let lock = &BANNED_PLAYER_LIST.read().await;
                sender
                    .send_message(TextComponent::text(format!(
                        "There are {} ban(s):",
                        lock.banned_players.len()
                    )))
                    .await;
                for player in &lock.banned_players {
                    sender
                        .send_message(TextComponent::text(format!(
                            "{} was banned by {}: {}",
                            player.name, player.source, player.reason
                        )))
                        .await;
                }
            }
            _ => {
                return Err(CommandError::GeneralCommandIssue(
                    "Incorrect argument for command".to_string(),
                ))
            }
        }

        Ok(())
    }
}

struct BanListAllExecutor;

#[async_trait]
impl CommandExecutor for BanListAllExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let mut entries = Vec::new();
        for player in &BANNED_PLAYER_LIST.read().await.banned_players {
            entries.push(format!(
                "{} was banned by {}: {}",
                player.name, player.source, player.reason
            ));
        }

        for ip in &BANNED_IP_LIST.read().await.banned_ips {
            entries.push(format!(
                "{} was banned by {}: {}",
                ip.ip, ip.source, ip.reason
            ));
        }

        sender
            .send_message(TextComponent::text(format!(
                "There are {} ban(s):",
                entries.len()
            )))
            .await;
        for entry in entries {
            sender.send_message(TextComponent::text(entry)).await;
        }

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .execute(BanListAllExecutor)
        .with_child(argument(ARG_LIST_TYPE, SimpleArgConsumer).execute(BanListExecutor))
}
