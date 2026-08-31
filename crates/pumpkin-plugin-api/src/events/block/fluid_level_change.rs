use crate::wit::pumpkin::plugin::event::{Event, EventType, FluidLevelChangeEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a fluid level changes.
pub struct FluidLevelChangeEvent;
impl FromIntoEvent for FluidLevelChangeEvent {
    const EVENT_TYPE: EventType = EventType::FluidLevelChangeEvent;
    type Data = FluidLevelChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::FluidLevelChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::FluidLevelChangeEvent(data)
    }
}
