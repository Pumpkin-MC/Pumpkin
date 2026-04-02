use crate::wit::pumpkin::plugin::event::{Event, EventType, ServerCommandEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a command is executed from the server console.
///
/// The associated [`ServerCommandEventData`] contains the command string.
/// This event is cancellable.
pub struct ServerCommandEvent;
impl FromIntoEvent for ServerCommandEvent {
    const EVENT_TYPE: EventType = EventType::ServerCommandEvent;
    type Data = ServerCommandEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ServerCommandEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ServerCommandEvent(data)
    }
}
