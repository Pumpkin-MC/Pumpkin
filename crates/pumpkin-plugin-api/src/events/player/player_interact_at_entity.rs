use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerInteractAtEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player interacts at a specific location on an entity.
pub struct PlayerInteractAtEntityEvent;
impl FromIntoEvent for PlayerInteractAtEntityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerInteractAtEntityEvent;
    type Data = PlayerInteractAtEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerInteractAtEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerInteractAtEntityEvent(data)
    }
}
