#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub fn handle_animate(&self, player: &Arc<Player>, packet: &SAnimate) {
        if !player.has_client_loaded() {
            return;
        }

        let entity = &player.get_entity();
        let world = entity.world.load();

        let java_animation = match packet.action {
            AnimateAction::NoAction | AnimateAction::SwingArm => None,
            AnimateAction::WakeUp => Some(Animation::LeaveBed),
            AnimateAction::CriticalHit => Some(Animation::CriticalEffect),
            AnimateAction::MagicCriticalHit => Some(Animation::MagicCriticaleffect),
        };

        let be_packet = SAnimate {
            action: packet.action,
            target_actor_runtime_id: VarULong(entity.entity_id as u64),
            data: 0.0,
            swing_source: None,
        };

        if let Some(animation) = java_animation {
            let je_packet = CEntityAnimation::new(VarInt(entity.entity_id), animation);
            world.broadcast_editioned(&je_packet, &be_packet);
        } else if matches!(packet.action, AnimateAction::SwingArm) {
            // Swings are their own packet on Java since 26.3
            let je_packet = CSwingArm::new(VarInt(entity.entity_id), false);
            world.broadcast_editioned(&je_packet, &be_packet);
        }
    }
}
