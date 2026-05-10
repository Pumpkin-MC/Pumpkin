use crate::wit::pumpkin::plugin::event::{BlockRedstoneEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a redstone component changes its power level.
///
/// The associated [`BlockRedstoneEventData`] contains the world, the block state ID,
/// the block position, and the old and new current values. This event is cancellable.
pub struct BlockRedstoneEvent;
impl FromIntoEvent for BlockRedstoneEvent {
    const EVENT_TYPE: EventType = EventType::BlockRedstoneEvent;
    type Data = BlockRedstoneEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockRedstoneEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockRedstoneEvent(data)
    }
}
