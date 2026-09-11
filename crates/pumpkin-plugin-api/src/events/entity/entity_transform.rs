use crate::wit::pumpkin::plugin::event::{EntityTransformEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity transforms into another entity.
pub struct EntityTransformEvent;
impl FromIntoEvent for EntityTransformEvent {
    const EVENT_TYPE: EventType = EventType::EntityTransformEvent;
    type Data = EntityTransformEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityTransformEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityTransformEvent(data)
    }
}
