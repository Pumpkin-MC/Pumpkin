#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_keep_alive(&self, player: &Player, keep_alive: &SKeepAlive) {
        let mut pending = self
            .pending_keep_alives
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(pos) = pending
            .iter()
            .position(|(id, _)| *id == keep_alive.keep_alive_id)
        {
            let (_, send_time) = pending.swap_remove(pos);
            let ping = send_time.elapsed().as_millis() as u32;
            // Vanilla logic
            player.ping.store(
                (player.ping.load(Ordering::Relaxed) * 3 + ping) / 4,
                Ordering::Relaxed,
            );
            self.wait_for_keep_alive.store(false, Ordering::Relaxed);
        } else if keep_alive.keep_alive_id == self.keep_alive_id.load() {
            let ping = self.last_keep_alive_time.load().elapsed().as_millis() as u32;
            player.ping.store(
                (player.ping.load(Ordering::Relaxed) * 3 + ping) / 4,
                Ordering::Relaxed,
            );
            self.wait_for_keep_alive.store(false, Ordering::Relaxed);
        } else {
            debug!(
                "Ignored unexpected or duplicate keep alive id {} from player {}",
                keep_alive.keep_alive_id, player.gameprofile.name
            );
        }
    }
}
