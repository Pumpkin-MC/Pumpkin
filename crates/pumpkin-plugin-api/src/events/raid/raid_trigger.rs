use crate::wit::pumpkin::plugin::event::{Event, EventType, RaidTriggerEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a raid is triggered.
pub struct RaidTriggerEvent;
impl FromIntoEvent for RaidTriggerEvent {
    const EVENT_TYPE: EventType = EventType::RaidTriggerEvent;
    type Data = RaidTriggerEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::RaidTriggerEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::RaidTriggerEvent(data)
    }
}
