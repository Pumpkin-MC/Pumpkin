use crate::wit::pumpkin::plugin::event::{EntityTameEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity is tamed.
pub struct EntityTameEvent;
impl FromIntoEvent for EntityTameEvent {
    const EVENT_TYPE: EventType = EventType::EntityTameEvent;
    type Data = EntityTameEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityTameEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityTameEvent(data)
    }
}
