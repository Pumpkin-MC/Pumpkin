use crate::wit::pumpkin::plugin::event::{DialogShowEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a dialog is shown to a player.
pub struct DialogShowEvent;

impl FromIntoEvent for DialogShowEvent {
    const EVENT_TYPE: EventType = EventType::DialogShowEvent;
    type Data = DialogShowEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::DialogShowEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::DialogShowEvent(data)
    }
}
