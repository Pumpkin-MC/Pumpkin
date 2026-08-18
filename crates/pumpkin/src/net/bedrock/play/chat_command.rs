#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_chat_command(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: SCommandRequest<'_>,
    ) {
        player.update_last_action_time();
        if player.check_chat_spam(server).await {
            return;
        }
        let player_clone = player.clone();
        let server_clone = server.clone();
        let command = packet.command.strip_prefix('/').unwrap_or(&packet.command);

        send_cancellable! {{
            server;
            PlayerCommandSendEvent {
                player: player.clone(),
                command: command.to_string(),
                cancelled: false
            };

            'after: {
                let command = event.command;
                let command_clone = command.clone();

                // Some commands can take a long time to execute. If they do, they block packet processing for the player.
                // That's why we will spawn a task instead.
                server.spawn_task(async move {
                    let dispatcher = server_clone.command_dispatcher.read().await;
                    dispatcher.handle_command(
                        &player_clone.get_command_source(&server_clone).await,
                        &command_clone
                    ).await;
                });

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
