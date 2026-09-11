use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, FurnaceStartSmeltEventData};

/// Event triggered when a furnace starts smelting an item.
pub struct FurnaceStartSmeltEvent;
impl FromIntoEvent for FurnaceStartSmeltEvent {
    const EVENT_TYPE: EventType = EventType::FurnaceStartSmeltEvent;
    type Data = FurnaceStartSmeltEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::FurnaceStartSmeltEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::FurnaceStartSmeltEvent(data)
    }
}
