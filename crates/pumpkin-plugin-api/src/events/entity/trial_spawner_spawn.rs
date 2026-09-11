use crate::wit::pumpkin::plugin::event::{Event, EventType, TrialSpawnerSpawnEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a trial spawner spawns an entity.
pub struct TrialSpawnerSpawnEvent;
impl FromIntoEvent for TrialSpawnerSpawnEvent {
    const EVENT_TYPE: EventType = EventType::TrialSpawnerSpawnEvent;
    type Data = TrialSpawnerSpawnEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::TrialSpawnerSpawnEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::TrialSpawnerSpawnEvent(data)
    }
}
