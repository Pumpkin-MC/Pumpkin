use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerOpenSignEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player opens a sign editor.
pub struct PlayerOpenSignEvent;
impl FromIntoEvent for PlayerOpenSignEvent {
    const EVENT_TYPE: EventType = EventType::PlayerOpenSignEvent;
    type Data = PlayerOpenSignEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerOpenSignEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerOpenSignEvent(data)
    }
}
