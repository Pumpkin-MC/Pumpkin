use crate::wit::pumpkin::plugin::event::{EntityBlockFormEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a block forms due to an entity action.
pub struct EntityBlockFormEvent;
impl FromIntoEvent for EntityBlockFormEvent {
    const EVENT_TYPE: EventType = EventType::EntityBlockFormEvent;
    type Data = EntityBlockFormEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityBlockFormEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityBlockFormEvent(data)
    }
}
