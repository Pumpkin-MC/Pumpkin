use crate::wit::pumpkin::plugin::event::{EntityPortalEnterEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when an entity enters a portal.
pub struct EntityPortalEnterEvent;
impl FromIntoEvent for EntityPortalEnterEvent {
    const EVENT_TYPE: EventType = EventType::EntityPortalEnterEvent;
    type Data = EntityPortalEnterEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityPortalEnterEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityPortalEnterEvent(data)
    }
}
