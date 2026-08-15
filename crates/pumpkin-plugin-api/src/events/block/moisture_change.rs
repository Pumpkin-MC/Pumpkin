use crate::wit::pumpkin::plugin::event::{Event, EventType, MoistureChangeEventData};

use super::super::FromIntoEvent;

/// An event that occurs when farmland moisture changes.
pub struct MoistureChangeEvent;
impl FromIntoEvent for MoistureChangeEvent {
    const EVENT_TYPE: EventType = EventType::MoistureChangeEvent;
    type Data = MoistureChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::MoistureChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::MoistureChangeEvent(data)
    }
}
