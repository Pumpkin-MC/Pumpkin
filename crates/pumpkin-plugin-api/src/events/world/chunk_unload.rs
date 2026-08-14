use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{ChunkUnloadEventData, Event, EventType};

/// Event triggered when a chunk is unloaded.
pub struct ChunkUnloadEvent;
impl FromIntoEvent for ChunkUnloadEvent {
    const EVENT_TYPE: EventType = EventType::ChunkUnloadEvent;
    type Data = ChunkUnloadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ChunkUnloadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ChunkUnloadEvent(data)
    }
}
