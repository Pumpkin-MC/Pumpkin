use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PrepareAnvilEventData};

/// Event triggered when an item is prepared in an anvil.
pub struct PrepareAnvilEvent;
impl FromIntoEvent for PrepareAnvilEvent {
    const EVENT_TYPE: EventType = EventType::PrepareAnvilEvent;
    type Data = PrepareAnvilEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PrepareAnvilEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PrepareAnvilEvent(data)
    }
}
