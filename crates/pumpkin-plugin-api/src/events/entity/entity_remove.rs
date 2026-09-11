use crate::wit::pumpkin::plugin::event::{EntityRemoveEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when an entity is removed from the world.
pub struct EntityRemoveEvent;
impl FromIntoEvent for EntityRemoveEvent {
    const EVENT_TYPE: EventType = EventType::EntityRemoveEvent;
    type Data = EntityRemoveEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityRemoveEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityRemoveEvent(data)
    }
}
