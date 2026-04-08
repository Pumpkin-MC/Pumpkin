use crate::wit::pumpkin::plugin::event::{BlockGrowEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a block grows or changes state naturally (e.g. crops, saplings).
///
/// The associated [`BlockGrowEventData`] contains the world, the old and new block
/// identifiers with their state IDs, and the block position. This event is cancellable.
pub struct BlockGrowEvent;
impl FromIntoEvent for BlockGrowEvent {
    const EVENT_TYPE: EventType = EventType::BlockGrowEvent;
    type Data = BlockGrowEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockGrowEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockGrowEvent(data)
    }
}
