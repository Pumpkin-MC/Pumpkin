use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{EntitiesLoadEventData, Event, EventType};

/// Event triggered when entities are loaded in a chunk.
pub struct EntitiesLoadEvent;
impl FromIntoEvent for EntitiesLoadEvent {
    const EVENT_TYPE: EventType = EventType::EntitiesLoadEvent;
    type Data = EntitiesLoadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntitiesLoadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntitiesLoadEvent(data)
    }
}
