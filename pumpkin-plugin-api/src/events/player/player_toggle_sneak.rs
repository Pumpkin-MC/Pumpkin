use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerToggleSneakEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player toggles sneak.
pub struct PlayerToggleSneakEvent;
impl FromIntoEvent for PlayerToggleSneakEvent {
    const EVENT_TYPE: EventType = EventType::PlayerToggleSneakEvent;
    type Data = PlayerToggleSneakEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerToggleSneakEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerToggleSneakEvent(data)
    }
}
