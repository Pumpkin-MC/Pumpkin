use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerLeashEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player leashes an entity.
pub struct PlayerLeashEntityEvent;
impl FromIntoEvent for PlayerLeashEntityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerLeashEntityEvent;
    type Data = PlayerLeashEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerLeashEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerLeashEntityEvent(data)
    }
}
