use crate::wit::pumpkin::plugin::event::{DialogClearEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a player's dialog is cleared.
pub struct DialogClearEvent;

impl FromIntoEvent for DialogClearEvent {
    const EVENT_TYPE: EventType = EventType::DialogClearEvent;
    type Data = DialogClearEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::DialogClearEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::DialogClearEvent(data)
    }
}
