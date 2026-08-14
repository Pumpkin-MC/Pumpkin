#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_close_container(
        &self,
        player: &Arc<Player>,
        _server: &Server,
        _packet: SCloseContainer,
    ) {
        player.on_handled_screen_closed().await;
    }
}
