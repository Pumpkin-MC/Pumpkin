use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PrepareItemCraftEventData};

/// Event triggered when a recipe is prepared in a crafting matrix.
pub struct PrepareItemCraftEvent;
impl FromIntoEvent for PrepareItemCraftEvent {
    const EVENT_TYPE: EventType = EventType::PrepareItemCraftEvent;
    type Data = PrepareItemCraftEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PrepareItemCraftEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PrepareItemCraftEvent(data)
    }
}
