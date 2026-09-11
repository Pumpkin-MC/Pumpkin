use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerShearEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player shears an entity.
pub struct PlayerShearEntityEvent;
impl FromIntoEvent for PlayerShearEntityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerShearEntityEvent;
    type Data = PlayerShearEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerShearEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerShearEntityEvent(data)
    }
}
