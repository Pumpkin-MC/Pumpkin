use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerNameEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player names an entity using a name tag.
pub struct PlayerNameEntityEvent;
impl FromIntoEvent for PlayerNameEntityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerNameEntityEvent;
    type Data = PlayerNameEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerNameEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerNameEntityEvent(data)
    }
}
