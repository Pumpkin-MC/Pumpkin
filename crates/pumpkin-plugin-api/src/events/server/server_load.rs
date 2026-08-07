use crate::wit::pumpkin::plugin::event::{Event, EventType, ServerLoadEventData};

use super::super::FromIntoEvent;

/// An event that fires once the server has finished loading.
///
/// The associated [`ServerLoadEventData`] contains the load reason:
/// startup or a full server reload.
pub struct ServerLoadEvent;
impl FromIntoEvent for ServerLoadEvent {
    const EVENT_TYPE: EventType = EventType::ServerLoadEvent;
    type Data = ServerLoadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ServerLoadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ServerLoadEvent(data)
    }
}
