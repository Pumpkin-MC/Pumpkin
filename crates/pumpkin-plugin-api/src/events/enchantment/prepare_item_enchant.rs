use crate::wit::pumpkin::plugin::event::{Event, EventType, PrepareItemEnchantEventData};

use super::super::FromIntoEvent;

/// Event triggered when an item is placed in an enchanting table to prepare options.
pub struct PrepareItemEnchantEvent;
impl FromIntoEvent for PrepareItemEnchantEvent {
    const EVENT_TYPE: EventType = EventType::PrepareItemEnchantEvent;
    type Data = PrepareItemEnchantEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PrepareItemEnchantEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PrepareItemEnchantEvent(data)
    }
}
