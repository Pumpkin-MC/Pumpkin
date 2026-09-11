use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerShowEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when an entity is made visible to a player.
pub struct PlayerShowEntityEvent;
impl FromIntoEvent for PlayerShowEntityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerShowEntityEvent;
    type Data = PlayerShowEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerShowEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerShowEntityEvent(data)
    }
}
