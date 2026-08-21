use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerCommandPreprocessEventData};

use super::super::FromIntoEvent;

/// An event that occurs before a player command is executed.
pub struct PlayerCommandPreprocessEvent;
impl FromIntoEvent for PlayerCommandPreprocessEvent {
    const EVENT_TYPE: EventType = EventType::PlayerCommandPreprocessEvent;
    type Data = PlayerCommandPreprocessEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerCommandPreprocessEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerCommandPreprocessEvent(data)
    }
}
