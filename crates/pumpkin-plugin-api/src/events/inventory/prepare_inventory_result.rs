use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PrepareInventoryResultEventData};

/// Generalized event triggered when an inventory result slot is prepared.
pub struct PrepareInventoryResultEvent;
impl FromIntoEvent for PrepareInventoryResultEvent {
    const EVENT_TYPE: EventType = EventType::PrepareInventoryResultEvent;
    type Data = PrepareInventoryResultEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PrepareInventoryResultEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PrepareInventoryResultEvent(data)
    }
}
