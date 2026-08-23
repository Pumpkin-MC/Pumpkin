use crate::wit::pumpkin::plugin::event::{EntityUnleashEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when an entity is unleashed.
pub struct EntityUnleashEvent;
impl FromIntoEvent for EntityUnleashEvent {
    const EVENT_TYPE: EventType = EventType::EntityUnleashEvent;
    type Data = EntityUnleashEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityUnleashEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityUnleashEvent(data)
    }
}
