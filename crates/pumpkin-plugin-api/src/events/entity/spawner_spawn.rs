use crate::wit::pumpkin::plugin::event::{Event, EventType, SpawnerSpawnEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a spawner spawns an entity.
pub struct SpawnerSpawnEvent;
impl FromIntoEvent for SpawnerSpawnEvent {
    const EVENT_TYPE: EventType = EventType::SpawnerSpawnEvent;
    type Data = SpawnerSpawnEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::SpawnerSpawnEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::SpawnerSpawnEvent(data)
    }
}
