use crate::wit::pumpkin::plugin::event::{BatToggleSleepEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a bat toggles its sleep state.
pub struct BatToggleSleepEvent;
impl FromIntoEvent for BatToggleSleepEvent {
    const EVENT_TYPE: EventType = EventType::BatToggleSleepEvent;
    type Data = BatToggleSleepEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BatToggleSleepEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BatToggleSleepEvent(data)
    }
}
