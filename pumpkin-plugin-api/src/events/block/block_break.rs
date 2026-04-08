use crate::wit::pumpkin::plugin::event::{BlockBreakEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a block is broken.
///
/// The associated [`BlockBreakEventData`] contains the player (if any), the block
/// identifier, its position, the experience to drop, and whether the block should
/// drop items. This event is cancellable.
pub struct BlockBreakEvent;
impl FromIntoEvent for BlockBreakEvent {
    const EVENT_TYPE: EventType = EventType::BlockBreakEvent;
    type Data = BlockBreakEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockBreakEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockBreakEvent(data)
    }
}
