use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerEditBookEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player edits or signs a book.
pub struct PlayerEditBookEvent;
impl FromIntoEvent for PlayerEditBookEvent {
    const EVENT_TYPE: EventType = EventType::PlayerEditBookEvent;
    type Data = PlayerEditBookEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerEditBookEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerEditBookEvent(data)
    }
}
