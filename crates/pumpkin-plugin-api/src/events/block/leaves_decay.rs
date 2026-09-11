use crate::wit::pumpkin::plugin::event::{Event, EventType, LeavesDecayEventData};

use super::super::FromIntoEvent;

/// An event that occurs when leaves decay naturally.
pub struct LeavesDecayEvent;
impl FromIntoEvent for LeavesDecayEvent {
    const EVENT_TYPE: EventType = EventType::LeavesDecayEvent;
    type Data = LeavesDecayEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::LeavesDecayEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::LeavesDecayEvent(data)
    }
}
