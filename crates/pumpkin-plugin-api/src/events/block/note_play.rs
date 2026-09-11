use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, NotePlayEventData};

/// An event that occurs when a note block plays.
pub struct NotePlayEvent;
impl FromIntoEvent for NotePlayEvent {
    const EVENT_TYPE: EventType = EventType::NotePlayEvent;
    type Data = NotePlayEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::NotePlayEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::NotePlayEvent(data)
    }
}
