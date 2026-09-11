use crate::wit::pumpkin::plugin::event::{EntityBreedEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when two entities breed.
pub struct EntityBreedEvent;
impl FromIntoEvent for EntityBreedEvent {
    const EVENT_TYPE: EventType = EventType::EntityBreedEvent;
    type Data = EntityBreedEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityBreedEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityBreedEvent(data)
    }
}
