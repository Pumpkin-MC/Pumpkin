use crate::wit::pumpkin::plugin::event::{ChunkLoadEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a chunk is loaded in a world.
pub struct ChunkLoadEvent;

impl FromIntoEvent for ChunkLoadEvent {
    const EVENT_TYPE: EventType = EventType::ChunkLoadEvent;
    type Data = ChunkLoadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ChunkLoadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ChunkLoadEvent(data)
    }
}
