use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockPhysicsEventData, Event, EventType};

/// An event that occurs when a block physics check is run.
pub struct BlockPhysicsEvent;
impl FromIntoEvent for BlockPhysicsEvent {
    const EVENT_TYPE: EventType = EventType::BlockPhysicsEvent;
    type Data = BlockPhysicsEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockPhysicsEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockPhysicsEvent(data)
    }
}
