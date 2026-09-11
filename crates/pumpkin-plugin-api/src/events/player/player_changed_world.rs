use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerChangedWorldEventData};

use super::super::FromIntoEvent;

/// An event that occurs after a player has changed worlds.
pub struct PlayerChangedWorldEvent;
impl FromIntoEvent for PlayerChangedWorldEvent {
    const EVENT_TYPE: EventType = EventType::PlayerChangedWorldEvent;
    type Data = PlayerChangedWorldEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerChangedWorldEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerChangedWorldEvent(data)
    }
}
