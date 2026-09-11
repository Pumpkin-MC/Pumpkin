use crate::wit::pumpkin::plugin::event::{Event, EventType, HangingPlaceEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a hanging entity is placed.
pub struct HangingPlaceEvent;
impl FromIntoEvent for HangingPlaceEvent {
    const EVENT_TYPE: EventType = EventType::HangingPlaceEvent;
    type Data = HangingPlaceEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::HangingPlaceEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::HangingPlaceEvent(data)
    }
}
