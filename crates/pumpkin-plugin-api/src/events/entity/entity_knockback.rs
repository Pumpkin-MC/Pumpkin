use crate::wit::pumpkin::plugin::event::{EntityKnockbackEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity receives knockback.
pub struct EntityKnockbackEvent;
impl FromIntoEvent for EntityKnockbackEvent {
    const EVENT_TYPE: EventType = EventType::EntityKnockbackEvent;
    type Data = EntityKnockbackEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityKnockbackEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityKnockbackEvent(data)
    }
}
