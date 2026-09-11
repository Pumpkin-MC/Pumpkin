use crate::wit::pumpkin::plugin::event::{AsyncPlayerChatEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An asynchronous event that occurs when a player sends a chat message.
pub struct AsyncPlayerChatEvent;
impl FromIntoEvent for AsyncPlayerChatEvent {
    const EVENT_TYPE: EventType = EventType::AsyncPlayerChatEvent;
    type Data = AsyncPlayerChatEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::AsyncPlayerChatEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::AsyncPlayerChatEvent(data)
    }
}
