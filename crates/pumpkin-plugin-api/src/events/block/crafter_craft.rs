use crate::wit::pumpkin::plugin::event::{CrafterCraftEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a crafter crafts an item.
pub struct CrafterCraftEvent;
impl FromIntoEvent for CrafterCraftEvent {
    const EVENT_TYPE: EventType = EventType::CrafterCraftEvent;
    type Data = CrafterCraftEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::CrafterCraftEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::CrafterCraftEvent(data)
    }
}
