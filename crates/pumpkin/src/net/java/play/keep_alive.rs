#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_keep_alive(&self, player: &Player, keep_alive: SKeepAlive) {
        if self.wait_for_keep_alive.load(Ordering::Relaxed)
            && keep_alive.keep_alive_id == self.keep_alive_id.load()
        {
            let ping = self.last_keep_alive_time.load().elapsed();
            // Vanilla logic
            player.ping.store(
                (player.ping.load(Ordering::Relaxed) * 3 + ping.as_millis() as u32) / 4,
                Ordering::Relaxed,
            );
            self.wait_for_keep_alive.store(false, Ordering::Relaxed);
        } else {
            self.kick(pumpkin_macros::translate_cross!(
                translation::java::DISCONNECT_TIMEOUT,
                translation::bedrock::DISCONNECT_TIMEOUT
            ))
            .await;
        }
    }
}
