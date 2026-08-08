use crate::wit::pumpkin::plugin::event::{Event, EventType, ServerListPingEventData};

use super::super::FromIntoEvent;

/// Fires when the server prepares a Java status/list ping response.
///
/// Register this as a blocking event handler to customize the MOTD, favicon,
/// and reported player counts for WASM plugins.
pub struct ServerListPingEvent;

impl FromIntoEvent for ServerListPingEvent {
    const EVENT_TYPE: EventType = EventType::ServerListPingEvent;
    type Data = ServerListPingEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ServerListPingEvent(data) => data,
            _ => panic!("expected ServerListPingEvent"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ServerListPingEvent(data)
    }
}
