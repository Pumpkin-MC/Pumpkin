use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, FurnaceSmeltEventData};

pub struct FurnaceSmeltEvent;
impl FromIntoEvent for FurnaceSmeltEvent {
    const EVENT_TYPE: EventType = EventType::FurnaceSmeltEvent;
    type Data = FurnaceSmeltEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::FurnaceSmeltEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::FurnaceSmeltEvent(data)
    }
}
