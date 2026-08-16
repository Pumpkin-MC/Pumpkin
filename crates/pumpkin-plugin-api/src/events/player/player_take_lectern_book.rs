use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerTakeLecternBookEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player takes a book from a lectern.
pub struct PlayerTakeLecternBookEvent;
impl FromIntoEvent for PlayerTakeLecternBookEvent {
    const EVENT_TYPE: EventType = EventType::PlayerTakeLecternBookEvent;
    type Data = PlayerTakeLecternBookEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerTakeLecternBookEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerTakeLecternBookEvent(data)
    }
}
