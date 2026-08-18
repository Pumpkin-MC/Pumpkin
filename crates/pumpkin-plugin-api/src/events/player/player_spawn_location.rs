use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerSpawnLocationEventData};

use super::super::FromIntoEvent;

/// An event that occurs when determining player spawn position.
pub struct PlayerSpawnLocationEvent;
impl FromIntoEvent for PlayerSpawnLocationEvent {
    const EVENT_TYPE: EventType = EventType::PlayerSpawnLocationEvent;
    type Data = PlayerSpawnLocationEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerSpawnLocationEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerSpawnLocationEvent(data)
    }
}
