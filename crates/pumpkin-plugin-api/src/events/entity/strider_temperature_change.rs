use crate::wit::pumpkin::plugin::event::{Event, EventType, StriderTemperatureChangeEventData};

use super::super::FromIntoEvent;

/// Event triggered when a strider's shivering status changes.
pub struct StriderTemperatureChangeEvent;
impl FromIntoEvent for StriderTemperatureChangeEvent {
    const EVENT_TYPE: EventType = EventType::StriderTemperatureChangeEvent;
    type Data = StriderTemperatureChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::StriderTemperatureChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::StriderTemperatureChangeEvent(data)
    }
}
