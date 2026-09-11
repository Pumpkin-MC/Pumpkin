use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, InventoryCreativeEventData};

/// Event triggered when a player in creative mode sets an inventory slot.
pub struct InventoryCreativeEvent;
impl FromIntoEvent for InventoryCreativeEvent {
    const EVENT_TYPE: EventType = EventType::InventoryCreativeEvent;
    type Data = InventoryCreativeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::InventoryCreativeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::InventoryCreativeEvent(data)
    }
}
