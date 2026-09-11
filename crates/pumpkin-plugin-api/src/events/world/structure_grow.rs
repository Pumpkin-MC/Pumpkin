use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, StructureGrowEventData};

/// Event triggered when a structure grows.
pub struct StructureGrowEvent;
impl FromIntoEvent for StructureGrowEvent {
    const EVENT_TYPE: EventType = EventType::StructureGrowEvent;
    type Data = StructureGrowEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::StructureGrowEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::StructureGrowEvent(data)
    }
}
