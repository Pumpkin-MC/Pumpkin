use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerPortalEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player uses a portal.
pub struct PlayerPortalEvent;
impl FromIntoEvent for PlayerPortalEvent {
    const EVENT_TYPE: EventType = EventType::PlayerPortalEvent;
    type Data = PlayerPortalEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerPortalEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerPortalEvent(data)
    }
}
