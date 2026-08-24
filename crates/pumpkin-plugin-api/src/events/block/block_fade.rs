use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockFadeEventData, Event, EventType};

/// Event triggered when a block fades or melts away.
pub struct BlockFadeEvent;
impl FromIntoEvent for BlockFadeEvent {
    const EVENT_TYPE: EventType = EventType::BlockFadeEvent;
    type Data = BlockFadeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockFadeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockFadeEvent(data)
    }
}
