use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerLocaleChangeEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player changes client language/locale.
pub struct PlayerLocaleChangeEvent;
impl FromIntoEvent for PlayerLocaleChangeEvent {
    const EVENT_TYPE: EventType = EventType::PlayerLocaleChangeEvent;
    type Data = PlayerLocaleChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerLocaleChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerLocaleChangeEvent(data)
    }
}
