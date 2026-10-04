#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_debug_sample_subscription(
        &self,
        player: &Arc<Player>,
        packet: &pumpkin_protocol::java::server::play::SDebugSampleSubscription,
    ) {
        if player.permission_lvl.load() >= PermissionLvl::Two
            && packet.sample_type.0
                == pumpkin_protocol::java::server::play::SDebugSampleSubscription::TICK_TIME
        {
            player
                .subscribed_debug_sample
                .store(true, Ordering::Relaxed);
        }
    }
}
