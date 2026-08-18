use crate::wit::pumpkin::plugin::event::{Event, EventType, RaidFinishEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a raid finishes.
pub struct RaidFinishEvent;
impl FromIntoEvent for RaidFinishEvent {
    const EVENT_TYPE: EventType = EventType::RaidFinishEvent;
    type Data = RaidFinishEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::RaidFinishEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::RaidFinishEvent(data)
    }
}
