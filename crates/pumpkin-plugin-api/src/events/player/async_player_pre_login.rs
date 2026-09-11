use crate::wit::pumpkin::plugin::event::{AsyncPlayerPreLoginEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An asynchronous event that occurs when a player attempts to pre-login.
pub struct AsyncPlayerPreLoginEvent;
impl FromIntoEvent for AsyncPlayerPreLoginEvent {
    const EVENT_TYPE: EventType = EventType::AsyncPlayerPreLoginEvent;
    type Data = AsyncPlayerPreLoginEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::AsyncPlayerPreLoginEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::AsyncPlayerPreLoginEvent(data)
    }
}
