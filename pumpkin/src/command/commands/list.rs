use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_config::BASIC_CONFIG;
use pumpkin_util::text::TextComponent;

use crate::{
    command::{
        args::ConsumedArgs, tree::CommandTree, CommandError, CommandExecutor, CommandSender,
    },
    entity::player::Player,
};

const NAMES: [&str; 1] = ["list"];

const DESCRIPTION: &str = "Print the list of online players.";

struct ListExecutor;

#[async_trait]
impl CommandExecutor for ListExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let players: Vec<Arc<Player>> = server.get_all_players().await;

        sender
            .send_message(TextComponent::translate(
                "commands.list.players",
                [
                    TextComponent::text(players.len().to_string()),
                    TextComponent::text(BASIC_CONFIG.max_players.to_string()),
                    TextComponent::text(get_player_names(players)),
                ],
            ))
            .await;

        Ok(())
    }
}

fn get_player_names(players: Vec<Arc<Player>>) -> String {
    let mut names = String::new();
    for player in players {
        if !names.is_empty() {
            names.push_str(", ");
        }
        names.push_str(&player.gameprofile.name);
    }
    names
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(ListExecutor)
}
