use crate::wit::pumpkin::plugin::event::{EnchantItemEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an item is enchanted at an enchanting table.
pub struct EnchantItemEvent;
impl FromIntoEvent for EnchantItemEvent {
    const EVENT_TYPE: EventType = EventType::EnchantItemEvent;
    type Data = EnchantItemEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EnchantItemEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EnchantItemEvent(data)
    }
}
