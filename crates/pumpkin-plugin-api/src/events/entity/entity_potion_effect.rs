use crate::wit::pumpkin::plugin::event::{EntityPotionEffectEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when a potion status effect is applied to an entity.
pub struct EntityPotionEffectEvent;
impl FromIntoEvent for EntityPotionEffectEvent {
    const EVENT_TYPE: EventType = EventType::EntityPotionEffectEvent;
    type Data = EntityPotionEffectEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityPotionEffectEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityPotionEffectEvent(data)
    }
}
