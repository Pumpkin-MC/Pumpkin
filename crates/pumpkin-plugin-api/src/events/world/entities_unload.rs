use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{EntitiesUnloadEventData, Event, EventType};

/// Event triggered when entities are unloaded in a chunk.
pub struct EntitiesUnloadEvent;
impl FromIntoEvent for EntitiesUnloadEvent {
    const EVENT_TYPE: EventType = EventType::EntitiesUnloadEvent;
    type Data = EntitiesUnloadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntitiesUnloadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntitiesUnloadEvent(data)
    }
}
