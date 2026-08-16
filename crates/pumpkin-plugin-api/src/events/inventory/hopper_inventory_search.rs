use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, HopperInventorySearchEventData};

/// Event triggered when a hopper searches for a container.
pub struct HopperInventorySearchEvent;
impl FromIntoEvent for HopperInventorySearchEvent {
    const EVENT_TYPE: EventType = EventType::HopperInventorySearchEvent;
    type Data = HopperInventorySearchEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::HopperInventorySearchEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::HopperInventorySearchEvent(data)
    }
}
