use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{AsyncStructureSpawnEventData, Event, EventType};

/// Event triggered when a structure is asynchronously spawned.
pub struct AsyncStructureSpawnEvent;
impl FromIntoEvent for AsyncStructureSpawnEvent {
    const EVENT_TYPE: EventType = EventType::AsyncStructureSpawnEvent;
    type Data = AsyncStructureSpawnEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::AsyncStructureSpawnEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::AsyncStructureSpawnEvent(data)
    }
}
