#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_chat_command(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        command: &SChatCommand<'_>,
    ) {
        player.update_last_action_time();
        if player.check_chat_spam(server) {
            return;
        }
        let command_str = command.command.strip_prefix('/').unwrap_or(command.command);
        send_cancellable! {{
            server;
            PlayerCommandSendEvent {
                player: player.clone(),
                command: command_str.to_string(),
                cancelled: false
            };

            'after: {
                let command = event.command;
                let dispatcher = server.command_dispatcher.load();
                dispatcher.handle_command(
                    &player.get_command_source(server),
                    &command,
                );

                if server.advanced_config.commands.log_console {
                    info!(
                        "Player ({}): executed command /{}",
                        player.gameprofile.name,
                        command
                    );
                }
            }
        }}
    }
}
