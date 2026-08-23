#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_actor_event(&self, player: &Player, packet: SActorEvent) {
        if packet.event_type != ActorEventType::Feed
            || !player
                .living_entity
                .item_in_use
                .lock()
                .await
                .as_ref()
                .and_then(|item| item.get_data_component::<ConsumableImpl>())
                .is_some_and(|consumable| consumable.animation == ConsumeAnimation::Eat)
        {
            return;
        }

        let entity = player.get_entity();
        entity.world.load().broadcast_to_chunk_bedrock(
            entity.chunk_pos.load(),
            &SActorEvent {
                entity_runtime_id: VarULong(entity.entity_id as u64),
                event_type: ActorEventType::Feed,
                event_data: packet.event_data,
                fire_at_position: None,
            },
        );
    }
}
