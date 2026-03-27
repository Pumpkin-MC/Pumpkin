use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerLoginEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player attempts to log in to the server.
///
/// The associated [`PlayerLoginEventData`] contains the player and a kick message
/// used if the login is cancelled. This event is cancellable.
pub struct PlayerLoginEvent;
impl FromIntoEvent for PlayerLoginEvent {
    const EVENT_TYPE: EventType = EventType::PlayerLoginEvent;
    type Data = PlayerLoginEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerLoginEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerLoginEvent(data)
    }
}
