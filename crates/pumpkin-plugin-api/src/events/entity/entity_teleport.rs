use crate::wit::pumpkin::plugin::event::{EntityTeleportEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity teleports.
pub struct EntityTeleportEvent;
impl FromIntoEvent for EntityTeleportEvent {
    const EVENT_TYPE: EventType = EventType::EntityTeleportEvent;
    type Data = EntityTeleportEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityTeleportEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityTeleportEvent(data)
    }
}
