use crate::wit::pumpkin::plugin::event::{DialogClickActionEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a player clicks a custom dialog button.
pub struct DialogClickActionEvent;

impl FromIntoEvent for DialogClickActionEvent {
    const EVENT_TYPE: EventType = EventType::DialogClickActionEvent;
    type Data = DialogClickActionEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::DialogClickActionEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::DialogClickActionEvent(data)
    }
}
