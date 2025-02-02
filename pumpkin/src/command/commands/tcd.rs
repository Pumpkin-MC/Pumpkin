use async_trait::async_trait;
use pumpkin_util::text::TextComponent;

use crate::command::args::ConsumedArgs;
use crate::command::tree::CommandTree;
use crate::command::CommandError;
use crate::command::{CommandExecutor, CommandSender};

const NAMES: [&str; 1] = ["tcd"];
const DESCRIPTION: &str = "Teleport cross-dimension"; // todo

struct TpSelfToPosExecutor;

#[async_trait]
impl CommandExecutor for TpSelfToPosExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        match sender {
            CommandSender::Player(player) => {
                // DELETE ME
                let cw = player.living_entity.entity.world.read().await.clone();
                let new_world_index = cw.level.level_info.level_name == "world";
                let worlds = _server.worlds.read().await;
                if new_world_index {
                    player
                        .clone()
                        .teleport_world(worlds[1].clone(), None, None, None)
                        .await;
                } else {
                    player
                        .clone()
                        .teleport_world(worlds[0].clone(), None, None, None)
                        .await;
                }
            }
            _ => {
                sender
                    .send_message(TextComponent::translate(
                        "permissions.requires.player",
                        [].into(),
                    ))
                    .await;
            }
        };

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).execute(TpSelfToPosExecutor)
}
