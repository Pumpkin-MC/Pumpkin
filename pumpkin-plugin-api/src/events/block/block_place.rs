use crate::wit::pumpkin::plugin::event::{BlockPlaceEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a player places a block.
///
/// The associated [`BlockPlaceEventData`] contains the player, the block being placed,
/// the block it is placed against, the position, and a `can-build` flag. This event
/// is cancellable.
pub struct BlockPlaceEvent;
impl FromIntoEvent for BlockPlaceEvent {
    const EVENT_TYPE: EventType = EventType::BlockPlaceEvent;
    type Data = BlockPlaceEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockPlaceEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockPlaceEvent(data)
    }
}
