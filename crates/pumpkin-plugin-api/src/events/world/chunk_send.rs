use crate::wit::pumpkin::plugin::event::{ChunkSendEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a chunk is sent to a client.
pub struct ChunkSendEvent;

impl FromIntoEvent for ChunkSendEvent {
    const EVENT_TYPE: EventType = EventType::ChunkSendEvent;
    type Data = ChunkSendEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ChunkSendEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ChunkSendEvent(data)
    }
}
