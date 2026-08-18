use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{AsyncStructureGenerateEventData, Event, EventType};

/// Event triggered when a structure is asynchronously generated.
pub struct AsyncStructureGenerateEvent;
impl FromIntoEvent for AsyncStructureGenerateEvent {
    const EVENT_TYPE: EventType = EventType::AsyncStructureGenerateEvent;
    type Data = AsyncStructureGenerateEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::AsyncStructureGenerateEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::AsyncStructureGenerateEvent(data)
    }
}
