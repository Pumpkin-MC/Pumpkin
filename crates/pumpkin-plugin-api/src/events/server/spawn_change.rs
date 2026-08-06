use crate::wit::pumpkin::plugin::event::{Event, EventType, SpawnChangeEventData};

use super::super::FromIntoEvent;

/// An event that occurs when the world spawn point changes.
///
/// The associated [`SpawnChangeEventData`] contains the world, the previous spawn
/// position, yaw and pitch, and the new spawn position, yaw and pitch.
pub struct SpawnChangeEvent;
impl FromIntoEvent for SpawnChangeEvent {
    const EVENT_TYPE: EventType = EventType::SpawnChangeEvent;
    type Data = SpawnChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::SpawnChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::SpawnChangeEvent(data)
    }
}
