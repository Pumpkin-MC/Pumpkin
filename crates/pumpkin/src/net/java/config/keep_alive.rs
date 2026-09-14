#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_config_keep_alive(&self, keep_alive: &SKeepAlive) {
        let mut pending = self
            .pending_keep_alives
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(pos) = pending
            .iter()
            .position(|(id, _)| *id == keep_alive.keep_alive_id)
        {
            pending.swap_remove(pos);
            self.wait_for_keep_alive.store(false, Ordering::Relaxed);
        } else if keep_alive.keep_alive_id == self.keep_alive_id.load() {
            self.wait_for_keep_alive.store(false, Ordering::Relaxed);
        } else {
            debug!(
                "Ignored unexpected config keep alive id {}",
                keep_alive.keep_alive_id
            );
        }
    }
}
