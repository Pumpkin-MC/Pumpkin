use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerKickEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player is kicked from the server.
pub struct PlayerKickEvent;
impl FromIntoEvent for PlayerKickEvent {
    const EVENT_TYPE: EventType = EventType::PlayerKickEvent;
    type Data = PlayerKickEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerKickEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerKickEvent(data)
    }
}
