use crate::wit::pumpkin::plugin::event::{Event, EventType, HangingBreakByEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a hanging entity is broken by another entity.
pub struct HangingBreakByEntityEvent;
impl FromIntoEvent for HangingBreakByEntityEvent {
    const EVENT_TYPE: EventType = EventType::HangingBreakByEntityEvent;
    type Data = HangingBreakByEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::HangingBreakByEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::HangingBreakByEntityEvent(data)
    }
}
