use crate::wit::pumpkin::plugin::event::{CustomClickActionEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a player clicks a custom dialog button.
pub struct CustomClickActionEvent;

impl FromIntoEvent for CustomClickActionEvent {
    const EVENT_TYPE: EventType = EventType::CustomClickActionEvent;
    type Data = CustomClickActionEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::CustomClickActionEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::CustomClickActionEvent(data)
    }
}
