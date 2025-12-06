use std::sync::Arc;

use pumpkin_util::text::TextComponent;

use crate::{
    command::{
        CommandExecutor, CommandResult, CommandSender, args::ConsumedArgs, tree::CommandTree,
    },
    entity::player::Player,
};

const NAMES: [&str; 1] = ["list"];

const DESCRIPTION: &str = "Print the list of online players.";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let players: Vec<Arc<Player>> = server.get_all_players().await;
            sender
                .send_message(TextComponent::translate(
                    "commands.list.players",
                    [
                        TextComponent::text(players.len().to_string()),
                        TextComponent::text(server.basic_config.max_players.to_string()),
                        TextComponent::text(get_player_names(players)),
                    ],
                ))
                .await;
            Ok(())
        })
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
    CommandTree::new(NAMES, DESCRIPTION).execute(Executor)
}
