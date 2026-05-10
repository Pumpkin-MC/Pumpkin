use crate::wit::pumpkin::plugin::event::{BlockCanBuildEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a player attempts to place a block, checking if it can be built.
///
/// The associated [`BlockCanBuildEventData`] contains the player, the block being
/// placed, the block it is being placed against, and a `buildable` flag that can be
/// overridden. This event is cancellable.
pub struct BlockCanBuildEvent;
impl FromIntoEvent for BlockCanBuildEvent {
    const EVENT_TYPE: EventType = EventType::BlockCanBuildEvent;
    type Data = BlockCanBuildEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockCanBuildEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockCanBuildEvent(data)
    }
}
