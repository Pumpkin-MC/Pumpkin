use crate::wit::pumpkin::plugin::event::{Event, EventType, ServerBroadcastEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a message is broadcast to all players on the server.
///
/// The associated [`ServerBroadcastEventData`] contains the message and the sender.
/// This event is cancellable.
pub struct ServerBroadcastEvent;
impl FromIntoEvent for ServerBroadcastEvent {
    const EVENT_TYPE: EventType = EventType::ServerBroadcastEvent;
    type Data = ServerBroadcastEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ServerBroadcastEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ServerBroadcastEvent(data)
    }
}
