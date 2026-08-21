use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, FurnaceExtractEventData};

/// Event triggered when a player extracts items from a furnace output slot.
pub struct FurnaceExtractEvent;
impl FromIntoEvent for FurnaceExtractEvent {
    const EVENT_TYPE: EventType = EventType::FurnaceExtractEvent;
    type Data = FurnaceExtractEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::FurnaceExtractEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::FurnaceExtractEvent(data)
    }
}
