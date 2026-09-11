use crate::wit::pumpkin::plugin::event::{Event, EventType, PigZapEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a pig transforms into a zombie pigman due to lightning.
pub struct PigZapEvent;
impl FromIntoEvent for PigZapEvent {
    const EVENT_TYPE: EventType = EventType::PigZapEvent;
    type Data = PigZapEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PigZapEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PigZapEvent(data)
    }
}
