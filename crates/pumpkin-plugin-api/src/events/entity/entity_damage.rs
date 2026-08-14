use crate::wit::pumpkin::plugin::event::{EntityDamageEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity receives damage.
pub struct EntityDamageEvent;
impl FromIntoEvent for EntityDamageEvent {
    const EVENT_TYPE: EventType = EventType::EntityDamageEvent;
    type Data = EntityDamageEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityDamageEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityDamageEvent(data)
    }
}
