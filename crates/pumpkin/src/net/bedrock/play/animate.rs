#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_animate(&self, player: &Arc<Player>, _server: &Server, packet: &SAnimate) {
        if !player.has_client_loaded() {
            return;
        }

        let entity = &player.get_entity();
        let world = entity.world.load();

        let java_animation = match packet.action {
            AnimateAction::NoAction => None,
            AnimateAction::SwingArm => Some(Animation::SwingMainArm),
            AnimateAction::WakeUp => Some(Animation::LeaveBed),
            AnimateAction::CriticalHit => Some(Animation::CriticalEffect),
            AnimateAction::MagicCriticalHit => Some(Animation::MagicCriticaleffect),
        };

        if let Some(animation) = java_animation {
            let je_packet = CEntityAnimation::new(VarInt(entity.entity_id), animation);
            let be_packet = SAnimate {
                action: packet.action,
                runtime_entity_id: VarULong(entity.entity_id as u64),
                data: 0.0,
                swing_source: None,
            };
            world.broadcast_editioned(&je_packet, &be_packet).await;
        }
    }
}
