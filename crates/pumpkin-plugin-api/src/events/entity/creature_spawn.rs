use crate::wit::pumpkin::plugin::event::{CreatureSpawnEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when a creature spawns.
pub struct CreatureSpawnEvent;
impl FromIntoEvent for CreatureSpawnEvent {
    const EVENT_TYPE: EventType = EventType::CreatureSpawnEvent;
    type Data = CreatureSpawnEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::CreatureSpawnEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::CreatureSpawnEvent(data)
    }
}
