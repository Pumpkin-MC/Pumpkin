use crate::wit::pumpkin::plugin::event::{Event, EventType, ItemSpawnEventData};

use super::super::FromIntoEvent;

/// Event triggered when an item entity spawns in the world.
pub struct ItemSpawnEvent;
impl FromIntoEvent for ItemSpawnEvent {
    const EVENT_TYPE: EventType = EventType::ItemSpawnEvent;
    type Data = ItemSpawnEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ItemSpawnEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ItemSpawnEvent(data)
    }
}
