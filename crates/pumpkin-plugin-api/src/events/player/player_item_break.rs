use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerItemBreakEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player breaks an item.
pub struct PlayerItemBreakEvent;
impl FromIntoEvent for PlayerItemBreakEvent {
    const EVENT_TYPE: EventType = EventType::PlayerItemBreakEvent;
    type Data = PlayerItemBreakEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerItemBreakEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerItemBreakEvent(data)
    }
}
