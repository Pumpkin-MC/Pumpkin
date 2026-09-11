use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, SmithItemEventData};

/// Event triggered when an item is crafted in a smithing table.
pub struct SmithItemEvent;
impl FromIntoEvent for SmithItemEvent {
    const EVENT_TYPE: EventType = EventType::SmithItemEvent;
    type Data = SmithItemEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::SmithItemEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::SmithItemEvent(data)
    }
}
