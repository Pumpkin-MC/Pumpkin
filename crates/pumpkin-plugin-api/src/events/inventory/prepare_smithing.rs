use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PrepareSmithingEventData};

/// Event triggered when items are prepared in a smithing table.
pub struct PrepareSmithingEvent;
impl FromIntoEvent for PrepareSmithingEvent {
    const EVENT_TYPE: EventType = EventType::PrepareSmithingEvent;
    type Data = PrepareSmithingEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PrepareSmithingEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PrepareSmithingEvent(data)
    }
}
