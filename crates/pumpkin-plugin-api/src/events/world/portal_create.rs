use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PortalCreateEventData};

/// Event triggered when a portal is created.
pub struct PortalCreateEvent;
impl FromIntoEvent for PortalCreateEvent {
    const EVENT_TYPE: EventType = EventType::PortalCreateEvent;
    type Data = PortalCreateEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PortalCreateEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PortalCreateEvent(data)
    }
}
