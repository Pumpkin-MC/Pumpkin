use crate::wit::pumpkin::plugin::event::{Event, EventType, HangingBreakEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a hanging entity is broken.
pub struct HangingBreakEvent;
impl FromIntoEvent for HangingBreakEvent {
    const EVENT_TYPE: EventType = EventType::HangingBreakEvent;
    type Data = HangingBreakEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::HangingBreakEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::HangingBreakEvent(data)
    }
}
