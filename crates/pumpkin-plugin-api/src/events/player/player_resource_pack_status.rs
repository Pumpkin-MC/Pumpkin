use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerResourcePackStatusEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player responds to a resource pack request.
pub struct PlayerResourcePackStatusEvent;
impl FromIntoEvent for PlayerResourcePackStatusEvent {
    const EVENT_TYPE: EventType = EventType::PlayerResourcePackStatusEvent;
    type Data = PlayerResourcePackStatusEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerResourcePackStatusEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerResourcePackStatusEvent(data)
    }
}
