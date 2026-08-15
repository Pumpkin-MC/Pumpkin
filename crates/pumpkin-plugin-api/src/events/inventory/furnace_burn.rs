use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, FurnaceBurnEventData};

/// Event triggered when fuel is burned in a furnace.
pub struct FurnaceBurnEvent;
impl FromIntoEvent for FurnaceBurnEvent {
    const EVENT_TYPE: EventType = EventType::FurnaceBurnEvent;
    type Data = FurnaceBurnEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::FurnaceBurnEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::FurnaceBurnEvent(data)
    }
}
