use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerUnleashEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player unleashes an entity.
pub struct PlayerUnleashEntityEvent;
impl FromIntoEvent for PlayerUnleashEntityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerUnleashEntityEvent;
    type Data = PlayerUnleashEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerUnleashEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerUnleashEntityEvent(data)
    }
}
