#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub fn handle_actor_event(&self, player: &Player, packet: &SActorEvent) {
        let is_valid_feed = packet.event_id == ActorEventID::Feed
            && player
                .living_entity
                .item_in_use
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref()
                .and_then(|item| item.get_data_component::<ConsumableImpl>())
                .is_some_and(|consumable| consumable.animation == ConsumeAnimation::Eat);

        let is_valid_drink = matches!(
            packet.event_id,
            ActorEventID::DrinkPotion | ActorEventID::DrinkMilk
        ) && player
            .living_entity
            .item_in_use
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(|item| item.get_data_component::<ConsumableImpl>())
            .is_some_and(|consumable| consumable.animation == ConsumeAnimation::Drink);

        if !is_valid_feed && !is_valid_drink {
            return;
        }

        let entity = player.get_entity();
        entity.world.load().broadcast_to_chunk_bedrock(
            entity.chunk_pos.load(),
            &SActorEvent {
                target_runtime_id: VarULong(entity.entity_id as u64),
                event_id: packet.event_id,
                data: packet.data,
                fire_at_position: None,
            },
        );
    }
}
