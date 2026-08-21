use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerInputEventData};

use super::super::FromIntoEvent;

/// An event that occurs when player inputs are received.
pub struct PlayerInputEvent;
impl FromIntoEvent for PlayerInputEvent {
    const EVENT_TYPE: EventType = EventType::PlayerInputEvent;
    type Data = PlayerInputEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerInputEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerInputEvent(data)
    }
}
