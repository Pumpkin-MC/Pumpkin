use crate::wit::pumpkin::plugin::event::{EntityResurrectEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity is resurrected.
pub struct EntityResurrectEvent;
impl FromIntoEvent for EntityResurrectEvent {
    const EVENT_TYPE: EventType = EventType::EntityResurrectEvent;
    type Data = EntityResurrectEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityResurrectEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityResurrectEvent(data)
    }
}
