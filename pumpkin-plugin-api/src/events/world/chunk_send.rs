use crate::wit::pumpkin::plugin::event::{ChunkSendEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a chunk is about to be sent to a client.
///
/// The associated [`ChunkSendEventData`] contains the target world, the chunk
/// being sent, and the cancellation state. This event is cancellable.
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
