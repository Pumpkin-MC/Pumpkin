use crate::wit::pumpkin::plugin::event::{Event, EventType, PiglinBarterEventData};

use super::super::FromIntoEvent;

/// Event triggered when a piglin barters.
pub struct PiglinBarterEvent;
impl FromIntoEvent for PiglinBarterEvent {
    const EVENT_TYPE: EventType = EventType::PiglinBarterEvent;
    type Data = PiglinBarterEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PiglinBarterEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PiglinBarterEvent(data)
    }
}
