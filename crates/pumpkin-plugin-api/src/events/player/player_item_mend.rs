use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerItemMendEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player's item is mended with experience.
pub struct PlayerItemMendEvent;
impl FromIntoEvent for PlayerItemMendEvent {
    const EVENT_TYPE: EventType = EventType::PlayerItemMendEvent;
    type Data = PlayerItemMendEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerItemMendEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerItemMendEvent(data)
    }
}
