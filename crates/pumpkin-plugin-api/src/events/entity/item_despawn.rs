use crate::wit::pumpkin::plugin::event::{Event, EventType, ItemDespawnEventData};

use super::super::FromIntoEvent;

/// Event triggered when an item entity despawns after aging.
pub struct ItemDespawnEvent;
impl FromIntoEvent for ItemDespawnEvent {
    const EVENT_TYPE: EventType = EventType::ItemDespawnEvent;
    type Data = ItemDespawnEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ItemDespawnEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ItemDespawnEvent(data)
    }
}
