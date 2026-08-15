#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_command_suggestion(
        &self,
        player: &Arc<Player>,
        packet: SCommandSuggestion<'_>,
        server: &Arc<Server>,
    ) {
        let Some(cmd) = &packet.command.get(1..) else {
            return;
        };

        let Some((last_word_start, _)) = cmd.char_indices().rfind(|(_, c)| c.is_whitespace())
        else {
            return;
        };

        let suggestions = server
            .command_dispatcher
            .read()
            .await
            .suggest(cmd, &player.get_command_source(server).await)
            .await;

        let response = CCommandSuggestions::new(
            packet.id,
            ((last_word_start + 2) as i32).into(),
            ((cmd.len() - last_word_start - 1) as i32).into(),
            suggestions.into(),
        );

        self.enqueue_packet(&response).await;
    }
}
