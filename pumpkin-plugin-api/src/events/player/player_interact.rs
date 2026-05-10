use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerInteractEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player interacts.
pub struct PlayerInteractEvent;
impl FromIntoEvent for PlayerInteractEvent {
    const EVENT_TYPE: EventType = EventType::PlayerInteractEvent;
    type Data = PlayerInteractEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerInteractEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerInteractEvent(data)
    }
}
