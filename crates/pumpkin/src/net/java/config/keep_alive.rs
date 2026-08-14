#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_config_keep_alive(&self, keep_alive: SKeepAlive) {
        if self.wait_for_keep_alive.load(Ordering::Relaxed)
            && keep_alive.keep_alive_id == self.keep_alive_id.load()
        {
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
