use crate::wit::pumpkin::plugin::event::{Event, EventType, SlimeSplitEventData};

use super::super::FromIntoEvent;

/// Event triggered when a slime splits into smaller slimes.
pub struct SlimeSplitEvent;
impl FromIntoEvent for SlimeSplitEvent {
    const EVENT_TYPE: EventType = EventType::SlimeSplitEvent;
    type Data = SlimeSplitEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::SlimeSplitEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::SlimeSplitEvent(data)
    }
}
