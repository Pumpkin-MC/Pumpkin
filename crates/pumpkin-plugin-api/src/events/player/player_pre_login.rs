use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerPreLoginEventData};

use super::super::FromIntoEvent;

/// An event that occurs synchronously when a player pre-logins.
pub struct PlayerPreLoginEvent;
impl FromIntoEvent for PlayerPreLoginEvent {
    const EVENT_TYPE: EventType = EventType::PlayerPreLoginEvent;
    type Data = PlayerPreLoginEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerPreLoginEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerPreLoginEvent(data)
    }
}
