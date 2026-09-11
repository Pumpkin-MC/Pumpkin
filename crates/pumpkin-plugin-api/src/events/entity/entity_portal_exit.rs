use crate::wit::pumpkin::plugin::event::{EntityPortalExitEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when an entity exits a portal.
pub struct EntityPortalExitEvent;
impl FromIntoEvent for EntityPortalExitEvent {
    const EVENT_TYPE: EventType = EventType::EntityPortalExitEvent;
    type Data = EntityPortalExitEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityPortalExitEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityPortalExitEvent(data)
    }
}
