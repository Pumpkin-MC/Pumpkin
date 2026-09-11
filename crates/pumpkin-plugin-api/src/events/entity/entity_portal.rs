use crate::wit::pumpkin::plugin::event::{EntityPortalEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity enters a portal.
pub struct EntityPortalEvent;
impl FromIntoEvent for EntityPortalEvent {
    const EVENT_TYPE: EventType = EventType::EntityPortalEvent;
    type Data = EntityPortalEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityPortalEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityPortalEvent(data)
    }
}
