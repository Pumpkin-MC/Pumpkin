use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerLevelChangeEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player's level changes.
pub struct PlayerLevelChangeEvent;
impl FromIntoEvent for PlayerLevelChangeEvent {
    const EVENT_TYPE: EventType = EventType::PlayerLevelChangeEvent;
    type Data = PlayerLevelChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerLevelChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerLevelChangeEvent(data)
    }
}
