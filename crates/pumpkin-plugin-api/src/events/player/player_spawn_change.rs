use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerSpawnChangeEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player's spawn location changes.
pub struct PlayerSpawnChangeEvent;
impl FromIntoEvent for PlayerSpawnChangeEvent {
    const EVENT_TYPE: EventType = EventType::PlayerSpawnChangeEvent;
    type Data = PlayerSpawnChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerSpawnChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerSpawnChangeEvent(data)
    }
}
