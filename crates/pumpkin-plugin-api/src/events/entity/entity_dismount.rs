use crate::wit::pumpkin::plugin::event::{EntityDismountEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity dismounts another entity.
pub struct EntityDismountEvent;
impl FromIntoEvent for EntityDismountEvent {
    const EVENT_TYPE: EventType = EventType::EntityDismountEvent;
    type Data = EntityDismountEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityDismountEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityDismountEvent(data)
    }
}
