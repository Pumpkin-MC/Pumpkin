use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PrepareGrindstoneEventData};

/// Event triggered when items are prepared in a grindstone.
pub struct PrepareGrindstoneEvent;
impl FromIntoEvent for PrepareGrindstoneEvent {
    const EVENT_TYPE: EventType = EventType::PrepareGrindstoneEvent;
    type Data = PrepareGrindstoneEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PrepareGrindstoneEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PrepareGrindstoneEvent(data)
    }
}
