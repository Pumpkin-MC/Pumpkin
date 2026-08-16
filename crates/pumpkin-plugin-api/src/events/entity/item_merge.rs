use crate::wit::pumpkin::plugin::event::{Event, EventType, ItemMergeEventData};

use super::super::FromIntoEvent;

/// Event triggered when two item entities merge.
pub struct ItemMergeEvent;
impl FromIntoEvent for ItemMergeEvent {
    const EVENT_TYPE: EventType = EventType::ItemMergeEvent;
    type Data = ItemMergeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ItemMergeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ItemMergeEvent(data)
    }
}
