use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{ChunkPopulateEventData, Event, EventType};

/// Event triggered when a chunk is populated.
pub struct ChunkPopulateEvent;
impl FromIntoEvent for ChunkPopulateEvent {
    const EVENT_TYPE: EventType = EventType::ChunkPopulateEvent;
    type Data = ChunkPopulateEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ChunkPopulateEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ChunkPopulateEvent(data)
    }
}
