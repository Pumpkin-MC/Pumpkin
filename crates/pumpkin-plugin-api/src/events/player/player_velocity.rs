use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerVelocityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player's velocity changes.
pub struct PlayerVelocityEvent;
impl FromIntoEvent for PlayerVelocityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerVelocityEvent;
    type Data = PlayerVelocityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerVelocityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerVelocityEvent(data)
    }
}
